import { atom, map } from 'nanostores';
import type { QueueItem, Settings, Theme } from './types';

// Default settings
const defaultSettings: Settings = {
  overwriteMode: 'rename',
  sizeLimitGB: 20,
  stripComponents: 0,
  allowSymlinks: false,
  allowHardlinks: false,
};

// Theme atom - stores the current theme preference
export const themeAtom = atom<Theme>('system');

// Settings atom - stores user preferences
export const settingsAtom = atom<Settings>(defaultSettings);

// Queue map - stores extraction jobs keyed by job_id
export const queueMap = map<Record<string, QueueItem>>({});

// Helper functions for queue management
export const addToQueue = (item: QueueItem) => {
  console.log('addToQueue called:', item);
  queueMap.setKey(item.id, item);
};

export const updateQueueItem = (id: string, updates: Partial<QueueItem>) => {
  const current = queueMap.get()[id];
  console.log('updateQueueItem called:', { id, current, updates });
  if (current) {
    const updated = { ...current, ...updates };
    console.log('Updating queue item:', updated);
    queueMap.setKey(id, updated);
  } else {
    console.warn('Queue item not found:', id, 'Available items:', Object.keys(queueMap.get()));
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
