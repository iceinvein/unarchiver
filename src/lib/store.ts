import { atom, map } from "nanostores";
import type { ArchiveInfo } from "./bindings/ArchiveInfo";
import type { QueueItem, Settings, Theme } from "./types";

// Default settings
const defaultSettings: Settings = {
	overwriteMode: "rename",
	sizeLimitGB: 20,
	stripComponents: 0,
	allowSymlinks: false,
	allowHardlinks: false,
	hasSeenPermissionDialog: false,
};

// Theme atom - stores the current theme preference
export const themeAtom = atom<Theme>("system");

// Settings atom - stores user preferences
export const settingsAtom = atom<Settings>(defaultSettings);

// Queue map - stores extraction jobs keyed by job_id
export const queueMap = map<Record<string, QueueItem>>({});

// Current directory in file explorer
export const currentDirectoryAtom = atom<string>("");

// Selected archive path
export const selectedArchiveAtom = atom<string | null>(null);

// Settings modal visibility
export const settingsModalAtom = atom<boolean>(false);

// Archive preview data and loading state
export const archivePreviewAtom = atom<{
	isLoading: boolean;
	contents: ArchiveInfo | null;
	error: string | null;
}>({
	isLoading: false,
	contents: null,
	error: null,
});

// Helper functions for queue management
export const addToQueue = (item: QueueItem) => {
	console.log("addToQueue called:", item);
	queueMap.setKey(item.id, item);
};

export const updateQueueItem = (id: string, updates: Partial<QueueItem>) => {
	const current = queueMap.get()[id];
	console.log("updateQueueItem called:", { id, current, updates });
	if (current) {
		const updated = { ...current, ...updates };
		console.log("Updating queue item:", updated);
		queueMap.setKey(id, updated);
	} else {
		console.warn(
			"Queue item not found:",
			id,
			"Available items:",
			Object.keys(queueMap.get()),
		);
	}
};

export const removeFromQueue = (id: string) => {
	const current = queueMap.get();
	const { [id]: _removed, ...rest } = current;
	queueMap.set(rest);
};

export const clearQueue = () => {
	queueMap.set({});
};

// Helper functions for settings
export const updateSettings = (updates: Partial<Settings>) => {
	settingsAtom.set({ ...settingsAtom.get(), ...updates });
};

export const resetSettings = () => {
	settingsAtom.set(defaultSettings);
};

// Helper function for theme
export const setTheme = (theme: Theme) => {
	themeAtom.set(theme);
};

// Helper functions for file explorer state
export const setCurrentDirectory = (path: string) => {
	currentDirectoryAtom.set(path);
};

export const setSelectedArchive = (path: string | null) => {
	selectedArchiveAtom.set(path);
};

// Helper functions for archive preview state
export const setArchivePreviewLoading = (isLoading: boolean) => {
	archivePreviewAtom.set({
		...archivePreviewAtom.get(),
		isLoading,
	});
};

export const setArchivePreviewContents = (contents: ArchiveInfo | null) => {
	archivePreviewAtom.set({
		isLoading: false,
		contents,
		error: null,
	});
};

export const setArchivePreviewError = (error: string) => {
	archivePreviewAtom.set({
		isLoading: false,
		contents: null,
		error,
	});
};

export const clearArchivePreview = () => {
	archivePreviewAtom.set({
		isLoading: false,
		contents: null,
		error: null,
	});
};

// Helper functions for settings modal
export const openSettingsModal = () => {
	settingsModalAtom.set(true);
};

export const closeSettingsModal = () => {
	settingsModalAtom.set(false);
};
