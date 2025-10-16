import { type RenderOptions, render } from "@testing-library/react";
import type { ReactElement } from "react";

/**
 * Custom render function that wraps components with necessary providers
 * Add any global providers here (e.g., theme provider, router, etc.)
 */
export function renderWithProviders(
	ui: ReactElement,
	options?: Omit<RenderOptions, "wrapper">,
) {
	return render(ui, { ...options });
}

/**
 * Re-export everything from testing library
 */
export * from "@testing-library/react";
export { default as userEvent } from "@testing-library/user-event";
