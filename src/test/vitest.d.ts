import type { TestingLibraryMatchers } from "@testing-library/jest-dom/matchers";
import "vitest";

declare module "vitest" {
	// biome-ignore lint/suspicious/noExplicitAny: Required for vitest type definitions
	interface Assertion<T = any> extends TestingLibraryMatchers<T, void> {}
	interface AsymmetricMatchersContaining extends TestingLibraryMatchers {}
}
