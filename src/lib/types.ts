// Type definitions for the application state

export type JobStatus =
	| "pending"
	| "extracting"
	| "completed"
	| "failed"
	| "cancelled";

export type OverwriteMode = "replace" | "skip" | "rename";

export type Theme = "light" | "dark" | "system";

export interface Progress {
	currentFile: string;
	bytesWritten: number;
	totalBytes?: number;
	filesExtracted: number;
}

export interface QueueItem {
	id: string;
	archivePath: string;
	outputDir: string;
	status: JobStatus;
	progress?: Progress;
	error?: string;
	stats?: import("./bindings/ExtractStats").ExtractStats;
}

export interface Settings {
	overwriteMode: OverwriteMode;
	sizeLimitGB: number;
	stripComponents: number;
	allowSymlinks: boolean;
	allowHardlinks: boolean;
}

export interface AppStore {
	queue: Map<string, QueueItem>;
	settings: Settings;
	theme: Theme;
}
