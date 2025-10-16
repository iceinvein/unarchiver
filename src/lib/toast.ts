import { atom } from "nanostores";

export type ToastType = "success" | "error" | "warning" | "info";

export interface Toast {
	id: string;
	message: string;
	type: ToastType;
	duration?: number;
}

// Store for active toasts
export const toastsAtom = atom<Toast[]>([]);

// Track recent toasts to prevent duplicates
const recentToasts = new Map<string, number>();
const DUPLICATE_WINDOW_MS = 1000; // 1 second window for duplicate detection

// Helper function to show a toast
export function showToast(
	message: string,
	type: ToastType = "info",
	duration = 5000,
) {
	// Create a key for duplicate detection
	const toastKey = `${type}:${message}`;
	const now = Date.now();

	// Check if we recently showed this exact toast
	const lastShown = recentToasts.get(toastKey);
	if (lastShown && now - lastShown < DUPLICATE_WINDOW_MS) {
		console.log("Duplicate toast prevented:", message);
		return lastShown.toString(); // Return a fake ID
	}

	// Record this toast
	recentToasts.set(toastKey, now);

	// Clean up old entries from the map
	for (const [key, timestamp] of recentToasts.entries()) {
		if (now - timestamp > DUPLICATE_WINDOW_MS) {
			recentToasts.delete(key);
		}
	}

	const id = crypto.randomUUID();
	const toast: Toast = { id, message, type, duration };

	// Add toast to the list
	toastsAtom.set([...toastsAtom.get(), toast]);

	// Auto-remove after duration
	if (duration > 0) {
		setTimeout(() => {
			removeToast(id);
		}, duration);
	}

	return id;
}

// Helper function to remove a toast
export function removeToast(id: string) {
	toastsAtom.set(toastsAtom.get().filter((toast) => toast.id !== id));
}

// Convenience functions
export function showSuccess(message: string, duration?: number) {
	return showToast(message, "success", duration);
}

export function showError(message: string, duration?: number) {
	return showToast(message, "error", duration);
}

export function showWarning(message: string, duration?: number) {
	return showToast(message, "warning", duration);
}

export function showInfo(message: string, duration?: number) {
	return showToast(message, "info", duration);
}
