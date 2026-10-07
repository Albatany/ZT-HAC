//! Folder Vault: AES-256-GCM file sealing with an Argon2id password-derived key.
//! File layout: MAGIC(6) | salt(16) | nonce(12) | ciphertext+tag.
//! The plaintext is `name_len(u16) | original_name | data`, so the original name
//! (and extension) is only stored encrypted; sealed files get the `.ztv` signature.

use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Key, Nonce};
use argon2::Argon2;
use serde::Serialize;
use std::{collections::HashMap, ffi::OsStr, fs, path::{Path, PathBuf}};

const MAGIC: &[u8; 6] = b"ZTHAC1";
const EXT: &str = "ztv";
const MIN_PASSWORD: usize = 8;

#[derive(Serialize, Default)]
pub struct VaultReport {
    pub processed: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}

fn random<const N: usize>() -> Result<[u8; N], String> {
    let mut buf = [0u8; N];
    getrandom::getrandom(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

fn derive_key(password: &str, salt: &[u8; 16]) -> Result<[u8; 32], String> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| e.to_string())?;
    Ok(key)
}

/// Recursively collects regular files; symlinks are never followed.
fn collect(path: &Path, out: &mut Vec<PathBuf>) {
    let Ok(meta) = fs::symlink_metadata(path) else { return };
    if meta.file_type().is_symlink() {
        return;
    }
    if meta.is_dir() {
        if let Ok(rd) = fs::read_dir(path) {
            for entry in rd.flatten() {
                collect(&entry.path(), out);
            }
        }
    } else if meta.is_file() {
        out.push(path.to_path_buf());
    }
}

fn seal(src: &Path, key: &[u8; 32], salt: &[u8; 16]) -> Result<(), String> {
    let name = src.file_name().and_then(|n| n.to_str()).ok_or("unsupported file name")?;
    let data = fs::read(src).map_err(|e| e.to_string())?;
    let mut plain = Vec::with_capacity(data.len() + name.len() + 2);
    plain.extend_from_slice(&(name.len() as u16).to_le_bytes());
    plain.extend_from_slice(name.as_bytes());
    plain.extend_from_slice(&data);

    let nonce = random::<12>()?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let ct = cipher.encrypt(Nonce::from_slice(&nonce), plain.as_ref()).map_err(|_| "encryption failed")?;

    let mut dest = src.with_extension(EXT);
    if dest.exists() {
        dest = PathBuf::from(format!("{}.{}", src.display(), EXT));
    }
    let mut blob = Vec::with_capacity(ct.len() + 34);
    blob.extend_from_slice(MAGIC);
    blob.extend_from_slice(salt);
    blob.extend_from_slice(&nonce);
    blob.extend_from_slice(&ct);
    fs::write(&dest, blob).map_err(|e| e.to_string())?;
    fs::remove_file(src).map_err(|e| e.to_string())
}

fn open(src: &Path, password: &str, cache: &mut HashMap<[u8; 16], [u8; 32]>) -> Result<(), String> {
    let blob = fs::read(src).map_err(|e| e.to_string())?;
    if blob.len() < 6 + 16 + 12 + 16 || &blob[..6] != MAGIC {
        return Err("not a ZT-HAC vault file".into());
    }
    let salt: [u8; 16] = blob[6..22].try_into().map_err(|_| "corrupt header")?;
    let key = match cache.get(&salt) {
        Some(k) => *k,
        None => {
            let k = derive_key(password, &salt)?;
            cache.insert(salt, k);
            k
        }
    };
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let plain = cipher
        .decrypt(Nonce::from_slice(&blob[22..34]), &blob[34..])
        .map_err(|_| "wrong password or corrupted file")?;
    if plain.len() < 2 {
        return Err("corrupt payload".into());
    }
    let n = u16::from_le_bytes([plain[0], plain[1]]) as usize;
    if plain.len() < 2 + n {
        return Err("corrupt payload".into());
    }
    let name = std::str::from_utf8(&plain[2..2 + n]).map_err(|_| "corrupt file name")?;
    // Guard against path traversal: the stored name must be a bare file name.
    if Path::new(name).file_name() != Some(OsStr::new(name)) {
        return Err("unsafe file name in vault".into());
    }
    let dest = src.with_file_name(name);
    if dest.exists() {
        return Err(format!("{} already exists", dest.display()));
    }
    fs::write(&dest, &plain[2 + n..]).map_err(|e| e.to_string())?;
    fs::remove_file(src).map_err(|e| e.to_string())
}

fn check_password(password: &str) -> Result<(), String> {
    if password.chars().count() < MIN_PASSWORD {
        return Err(format!("password must be at least {MIN_PASSWORD} characters"));
    }
    Ok(())
}

pub fn encrypt_path(path: &str, password: &str) -> Result<VaultReport, String> {
    check_password(password)?;
    let mut files = Vec::new();
    collect(Path::new(path), &mut files);
    if files.is_empty() {
        return Err("no files found at that path".into());
    }
    let salt = random::<16>()?;
    let key = derive_key(password, &salt)?; // derived once per operation
    let mut rep = VaultReport::default();
    for f in files {
        if f.extension().and_then(|e| e.to_str()) == Some(EXT) {
            rep.skipped += 1;
            continue;
        }
        match seal(&f, &key, &salt) {
            Ok(()) => rep.processed += 1,
            Err(e) => rep.errors.push(format!("{}: {e}", f.display())),
        }
    }
    Ok(rep)
}

pub fn decrypt_path(path: &str, password: &str) -> Result<VaultReport, String> {
    check_password(password)?;
    let mut files = Vec::new();
    collect(Path::new(path), &mut files);
    let mut cache = HashMap::new();
    let mut rep = VaultReport::default();
    for f in files {
        if f.extension().and_then(|e| e.to_str()) != Some(EXT) {
            rep.skipped += 1;
            continue;
        }
        match open(&f, password, &mut cache) {
            Ok(()) => rep.processed += 1,
            Err(e) => rep.errors.push(format!("{}: {e}", f.display())),
        }
    }
    Ok(rep)
}
