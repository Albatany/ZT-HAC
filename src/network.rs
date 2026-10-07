//! Network Port Stopper. Linux: iptables through pkexec (needs a polkit agent).
//! Windows: Windows Firewall through netsh (run the app as Administrator).

use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Clone, PartialEq)]
pub struct PortRule { pub port: u16, pub protocol: String }

fn validate(port: u16, proto: &str) -> Result<(), String> {
    if port == 0 {
        return Err("port must be between 1 and 65535".into());
    }
    if !matches!(proto, "tcp" | "udp") {
        return Err("protocol must be tcp or udp".into());
    }
    Ok(())
}

fn run(cmd: &mut Command) -> Result<(), String> {
    let out = cmd.output().map_err(|e| format!("could not launch command: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

#[cfg(target_os = "linux")]
fn apply(add: bool, port: u16, proto: &str) -> Result<(), String> {
    let op = if add { "-I" } else { "-D" };
    let p = port.to_string();
    for chain in ["INPUT", "OUTPUT"] {
        run(Command::new("pkexec").args([
            "iptables", op, chain, "-p", proto, "--dport", p.as_str(),
            "-m", "comment", "--comment", "ZTHAC", "-j", "DROP",
        ]))?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn apply(add: bool, port: u16, proto: &str) -> Result<(), String> {
    for dir in ["in", "out"] {
        let name = format!("name=ZTHAC-{proto}-{port}-{dir}");
        let args: Vec<String> = if add {
            let key = if dir == "in" { "localport" } else { "remoteport" };
            vec![
                "advfirewall".into(), "firewall".into(), "add".into(), "rule".into(), name,
                format!("dir={dir}"), "action=block".into(),
                format!("protocol={}", proto.to_uppercase()), format!("{key}={port}"),
            ]
        } else {
            vec!["advfirewall".into(), "firewall".into(), "delete".into(), "rule".into(), name]
        };
        run(Command::new("netsh").args(&args))?;
    }
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn apply(_add: bool, _port: u16, _proto: &str) -> Result<(), String> {
    Err("port rules are only supported on Linux and Windows".into())
}

pub fn block(port: u16, proto: &str) -> Result<(), String> {
    validate(port, proto)?;
    apply(true, port, proto)
}

pub fn unblock(port: u16, proto: &str) -> Result<(), String> {
    validate(port, proto)?;
    apply(false, port, proto)
}
