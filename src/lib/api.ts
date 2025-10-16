import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Settings } from './types';

// Import and re-export generated types from Rust
export type { ArchiveInfo } from './bindings/ArchiveInfo';
export type { ExtractStats } from './bindings/ExtractStats';
export type { ExtractOptionsDTO } from './bindings/ExtractOptionsDTO';
export type { ProgressEvent } from './bindings/ProgressEvent';
export type { CompletionEvent } from './bindings/CompletionEvent';
export type { PasswordRequiredEvent } from './bindings/PasswordRequiredEvent';
export type { JobStatus } from './bindings/JobStatus';

import type { ExtractOptionsDTO } from './bindings/ExtractOptionsDTO';
import type { ArchiveInfo } from './bindings/ArchiveInfo';
import type { ProgressEvent } from './bindings/ProgressEvent';
import type { CompletionEvent } from './bindings/CompletionEvent';
import type { PasswordRequiredEvent } from './bindings/PasswordRequiredEvent';

// Convert Settings to ExtractOptionsDTO
function settingsToOptions(settings: Settings, password?: string): ExtractOptionsDTO {
  return {
    overwrite: settings.overwriteMode,
    sizeLimitBytes: settings.sizeLimitGB > 0 ? settings.sizeLimitGB * 1024 * 1024 * 1024 : undefined,
    stripComponents: settings.stripComponents,
    allowSymlinks: settings.allowSymlinks,
    allowHardlinks: settings.allowHardlinks,
    password,
  };
}

/**
 * Extract one or more archives
 * @param paths - Array of archive file paths
 * @param outputDir - Output directory path
 * @param settings - Extraction settings
 * @param password - Optional password for encrypted archives
 * @returns Job ID for tracking progress
 */
export async function extractArchives(
  paths: string[],
  outputDir: string,
  settings: Settings,
  password?: string
): Promise<string> {
  const options = settingsToOptions(settings, password);
  return await invoke<string>('extract', {
    inputPaths: paths,
    outDir: outputDir,
    options,
  });
}

/**
 * Probe archive metadata without extracting
 * @param path - Archive file path
 * @returns Archive information
 */
export async function probeArchive(path: string): Promise<ArchiveInfo> {
  return await invoke<ArchiveInfo>('probe', { path });
}

/**
 * Cancel an in-progress extraction job
 * @param jobId - Job ID to cancel
 */
export async function cancelJob(jobId: string): Promise<void> {
  await invoke('cancel_job', { jobId });
}

/**
 * Provide password for a password-protected archive
 * @param jobId - Job ID that requires password
 * @param password - Password to use
 */
export async function providePassword(jobId: string, password: string): Promise<void> {
  await invoke('provide_password', { jobId, password });
}

/**
 * Listen for extraction progress events
 * @param callback - Function to call on progress updates
 * @returns Unlisten function to stop listening
 */
export async function onProgress(
  callback: (event: ProgressEvent) => void
): Promise<UnlistenFn> {
  return await listen<ProgressEvent>('extract_progress', (event) => {
    callback(event.payload);
  });
}

/**
 * Listen for extraction completion events
 * @param callback - Function to call when extraction completes
 * @returns Unlisten function to stop listening
 */
export async function onCompletion(
  callback: (event: CompletionEvent) => void
): Promise<UnlistenFn> {
  return await listen<CompletionEvent>('extract_done', (event) => {
    callback(event.payload);
  });
}

/**
 * Listen for password required events
 * @param callback - Function to call when password is needed
 * @returns Unlisten function to stop listening
 */
export async function onPasswordRequired(
  callback: (event: PasswordRequiredEvent) => void
): Promise<UnlistenFn> {
  return await listen<PasswordRequiredEvent>('password_required', (event) => {
    callback(event.payload);
  });
}

/**
 * Listen for files opened events (when archives are opened from Finder)
 * @param callback - Function to call when files are opened
 * @returns Unlisten function to stop listening
 */
export async function onFilesOpened(
  callback: (paths: string[]) => void
): Promise<UnlistenFn> {
  return await listen<string[]>('files_opened', (event) => {
    callback(event.payload);
  });
}
