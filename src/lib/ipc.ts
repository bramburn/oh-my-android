/**
 * The typed IPC surface. This is the ONLY module that calls `invoke`.
 *
 * Types mirror the Rust side (`crates/adbkit/src/parser.rs` for `Device`,
 * `src-tauri/src/error.rs` for `AppError`) — keep them in sync by hand until the generated bindings
 * exist (`docs/windows-port/12-ipc-contract.md` §6).
 */
import { invoke } from '@tauri-apps/api/core';

/** Mirrors `adbkit::parser::DeviceState` (`rename_all = "lowercase"`). */
export type DeviceState = 'device' | 'unauthorized' | 'offline' | 'other';

/** Mirrors `adbkit::parser::Device` (`rename_all = "camelCase"`). */
export interface Device {
  serial: string;
  model: string;
  state: DeviceState;
}

/** Mirrors `src-tauri/src/error.rs::AppError`. */
export interface AppError {
  code: string;
  message: string;
}

export function isAppError(value: unknown): value is AppError {
  if (typeof value !== 'object' || value === null) return false;
  const candidate = value as Record<string, unknown>;
  return typeof candidate.code === 'string' && typeof candidate.message === 'string';
}

/** `adb devices -l`, parsed in Rust. */
export function listDevices(): Promise<Device[]> {
  return invoke<Device[]>('list_devices');
}

/** `adb start-server`. All stdio null on the Rust side. */
export function adbStart(): Promise<void> {
  return invoke<void>('adb_start');
}

/** `adb kill-server`. Destructive: the caller must confirm first (override O2). */
export function adbStop(): Promise<void> {
  return invoke<void>('adb_stop');
}

/** Human-readable message for anything thrown across the boundary. */
export function messageOf(error: unknown): string {
  if (isAppError(error)) return error.message;
  if (error instanceof Error) return error.message;
  return String(error);
}

/** Machine-readable code, or `UNKNOWN`. */
export function codeOf(error: unknown): string {
  return isAppError(error) ? error.code : 'UNKNOWN';
}