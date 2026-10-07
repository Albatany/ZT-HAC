import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface ProcInfo { pid: number; name: string; memory_kb: number }
export interface Restriction { name: string; unlocked: boolean }
export interface VaultReport { processed: number; skipped: number; errors: string[] }
export interface GuardStatus { armed: boolean; paths: string[] }
export interface PortRule { port: number; protocol: string }

/** Typed bridge to the Rust commands registered in src-tauri/src/lib.rs. */
export const api = {
  // Dynamic App Restrictor
  listProcesses: () => invoke<ProcInfo[]>("list_processes"),
  listRestrictions: () => invoke<Restriction[]>("list_restrictions"),
  restrictApp: (name: string) => invoke<void>("restrict_app", { name }),
  unlockApp: (name: string) => invoke<void>("unlock_app", { name }),
  relockApp: (name: string) => invoke<void>("relock_app", { name }),
  removeRestriction: (name: string) => invoke<void>("remove_restriction", { name }),

  // One-Click Folder Vault
  encryptVault: (path: string, password: string) => invoke<VaultReport>("encrypt_vault", { path, password }),
  decryptVault: (path: string, password: string) => invoke<VaultReport>("decrypt_vault", { path, password }),

  // Automated Environment Guard
  guardArm: (paths: string[], password: string) => invoke<GuardStatus>("guard_arm", { paths, password }),
  guardDisarm: () => invoke<GuardStatus>("guard_disarm"),
  guardStatus: () => invoke<GuardStatus>("guard_status"),

  // Network Port Stopper
  listPortRules: () => invoke<PortRule[]>("list_port_rules"),
  blockPort: (port: number, protocol: string) => invoke<PortRule[]>("block_port", { port, protocol }),
  unblockPort: (port: number, protocol: string) => invoke<PortRule[]>("unblock_port", { port, protocol }),

  // Events emitted by the backend
  on: (event: "app-blocked" | "guard-event" | "guard-locked", cb: (payload: string) => void): Promise<UnlistenFn> =>
    listen<string>(event, (e) => cb(e.payload)),
};
