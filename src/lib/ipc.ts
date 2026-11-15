/**
 * The typed IPC surface. This is the ONLY module that calls `invoke`.
 *
 * Types mirror the Rust side — `adbkit::SetupReport` / `adbkit::Device` and `AppError` — by hand until
 * generated bindings exist (`docs/windows-port/12-ipc-contract.md` §6).
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

/** Mirrors `adbkit::sdk::SdkSource` (`rename_all = "camelCase"`). */
export type SdkSource =
  | 'custom'
  | 'androidHome'
  | 'androidSdkRoot'
  | 'localAppDataDefault'
  | 'programFilesLegacy';

/** Mirrors `adbkit::sdk::SdkLocation`. */
export interface SdkLocation {
  root: string;
  adb: string;
  source: SdkSource;
}

/** Mirrors `adbkit::setup::SetupReport`. */
export interface SetupReport {
  studio: string | null;
  sdk: SdkLocation | null;
  adbVersion: string | null;
  emulator: string | null;
  avdCount: number;
  envHome: string | null;
  customRoot: string | null;
  warning: string | null;
}

/** Mirrors `src-tauri/src/error.rs::AppError`. */
export interface AppError {
  code: string;
  message: string;
}

/** The four pages `open_docs` accepts (the allowlist lives in Rust). */
export const DOCS = {
  studio: 'https://developer.android.com/studio',
  adb: 'https://developer.android.com/tools/adb',
  avds: 'https://developer.android.com/studio/run/managing-avds',
  usb: 'https://developer.android.com/studio/debug/dev-options',
} as const;

export type DocUrl = (typeof DOCS)[keyof typeof DOCS];

export function isAppError(value: unknown): value is AppError {
  if (typeof value !== 'object' || value === null) return false;
  const candidate = value as Record<string, unknown>;
  return typeof candidate.code === 'string' && typeof candidate.message === 'string';
}

/** Read-only detection. The one process it may run is `adb version`. */
export function setupReport(): Promise<SetupReport> {
  return invoke<SetupReport>('setup_report');
}

/** Native folder picker; resolves to a fresh report. Cancelling returns the unchanged report. */
export function chooseSdkFolder(): Promise<SetupReport> {
  return invoke<SetupReport>('choose_sdk_folder');
}

/** `adb devices -l`, parsed in Rust. */
export function listDevices(): Promise<Device[]> {
  return invoke<Device[]>('list_devices');
}

/** `adb start-server`. */
export function adbStart(): Promise<void> {
  return invoke<void>('adb_start');
}

/** `adb kill-server`. Destructive: the caller must confirm first (override O2). */
export function adbStop(): Promise<void> {
  return invoke<void>('adb_stop');
}

/** Open one of `DOCS` in the system browser. Anything else is rejected in Rust. */
export function openDocs(url: DocUrl): Promise<void> {
  return invoke<void>('open_docs', { url });
}

export function messageOf(error: unknown): string {
  if (isAppError(error)) return error.message;
  if (error instanceof Error) return error.message;
  return String(error);
}

export function codeOf(error: unknown): string {
  return isAppError(error) ? error.code : 'UNKNOWN';
}

/** Human label for where the SDK was found. */
export function sdkSourceLabel(source: SdkSource): string {
  switch (source) {
    case 'custom':
      return 'chosen folder';
    case 'androidHome':
      return 'ANDROID_HOME';
    case 'androidSdkRoot':
      return 'ANDROID_SDK_ROOT';
    case 'localAppDataDefault':
      return 'Android Studio default folder';
    case 'programFilesLegacy':
      return 'legacy installer folder';
  }
}