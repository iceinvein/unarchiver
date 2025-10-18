import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Settings } from "./types";

export type { ArchiveEntry } from "./bindings/ArchiveEntry";
// Import and re-export generated types from Rust
export type { ArchiveInfo } from "./bindings/ArchiveInfo";
export type { CompletionEvent } from "./bindings/CompletionEvent";
export type { ExtractOptionsDTO } from "./bindings/ExtractOptionsDTO";
export type { ExtractStats } from "./bindings/ExtractStats";
export type { FileSystemEntry } from "./bindings/FileSystemEntry";
export type { JobStatus } from "./bindings/JobStatus";
export type { PasswordRequiredEvent } from "./bindings/PasswordRequiredEvent";
export type { ProgressEvent } from "./bindings/ProgressEvent";

import type { ArchiveInfo } from "./bindings/ArchiveInfo";
import type { CompletionEvent } from "./bindings/CompletionEvent";
import type { ExtractOptionsDTO } from "./bindings/ExtractOptionsDTO";
import type { FileSystemEntry } from "./bindings/FileSystemEntry";
import type { PasswordRequiredEvent } from "./bindings/PasswordRequiredEvent";
import type { ProgressEvent } from "./bindings/ProgressEvent";

// Convert Settings to ExtractOptionsDTO
function settingsToOptions(
	settings: Settings,
	password?: string,
): ExtractOptionsDTO {
	return {
		overwrite: settings.overwriteMode,
		sizeLimitBytes:
			settings.sizeLimitGB > 0
				? settings.sizeLimitGB * 1024 * 1024 * 1024
				: undefined,
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
	password?: string,
): Promise<string> {
	const options = settingsToOptions(settings, password);
	return await invoke<string>("extract", {
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
	return await invoke<ArchiveInfo>("probe", { path });
}

/**
 * Cancel an in-progress extraction job
 * @param jobId - Job ID to cancel
 */
export async function cancelJob(jobId: string): Promise<void> {
	await invoke("cancel_job", { jobId });
}

/**
 * Provide password for a password-protected archive
 * @param jobId - Job ID that requires password
 * @param password - Password to use
 */
export async function providePassword(
	jobId: string,
	password: string,
): Promise<void> {
	await invoke("provide_password", { jobId, password });
}

/**
 * Listen for extraction progress events
 * @param callback - Function to call on progress updates
 * @returns Unlisten function to stop listening
 */
export async function onProgress(
	callback: (event: ProgressEvent) => void,
): Promise<UnlistenFn> {
	return await listen<ProgressEvent>("extract_progress", (event) => {
		callback(event.payload);
	});
}

/**
 * Listen for extraction completion events
 * @param callback - Function to call when extraction completes
 * @returns Unlisten function to stop listening
 */
export async function onCompletion(
	callback: (event: CompletionEvent) => void,
): Promise<UnlistenFn> {
	return await listen<CompletionEvent>("extract_done", (event) => {
		callback(event.payload);
	});
}

/**
 * Listen for password required events
 * @param callback - Function to call when password is needed
 * @returns Unlisten function to stop listening
 */
export async function onPasswordRequired(
	callback: (event: PasswordRequiredEvent) => void,
): Promise<UnlistenFn> {
	return await listen<PasswordRequiredEvent>("password_required", (event) => {
		callback(event.payload);
	});
}

/**
 * Listen for files opened events (when archives are opened from Finder)
 * @param callback - Function to call when files are opened
 * @returns Unlisten function to stop listening
 */
export async function onFilesOpened(
	callback: (paths: string[]) => void,
): Promise<UnlistenFn> {
	return await listen<string[]>("files_opened", (event) => {
		callback(event.payload);
	});
}

/**
 * List directory contents with metadata
 * @param path - Directory path to list
 * @returns Array of file system entries
 */
export async function listDirectory(path: string): Promise<FileSystemEntry[]> {
	return await invoke<FileSystemEntry[]>("list_directory", { path });
}

/**
 * Get user's home directory path
 * @returns Home directory path
 */
export async function getHomeDirectory(): Promise<string> {
	return await invoke<string>("get_home_directory");
}

/**
 * Check if a path exists
 * @param path - Path to check
 * @returns True if path exists, false otherwise
 */
export async function checkPathExists(path: string): Promise<boolean> {
	return await invoke<boolean>("check_path_exists", { path });
}

/**
 * Get a unique output path for extraction with conflict resolution
 * @param archivePath - Archive file path
 * @returns Unique output directory path
 */
export async function getUniqueOutputPath(
	archivePath: string,
): Promise<string> {
	return await invoke<string>("get_unique_output_path", { archivePath });
}

/**
 * Get accessible default directories (works in sandbox)
 * @returns Array of accessible directories
 */
export async function getAccessibleDirectories(): Promise<FileSystemEntry[]> {
	return await invoke<FileSystemEntry[]>("get_accessible_directories");
}

/**
 * Request folder access permission using native file picker
 * @returns Selected folder path or null if cancelled
 */
export async function requestFolderAccess(): Promise<string | null> {
	return await invoke<string | null>("request_folder_access");
}
