import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../lib/api";
import ArchivePreview from "./ArchivePreview";

// Mock the API module
vi.mock("../lib/api", () => ({
	probeArchive: vi.fn(),
}));

describe("ArchivePreview", () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("should render empty state when no archive is selected", () => {
		render(<ArchivePreview archivePath={null} onExtract={vi.fn()} />);

		expect(
			screen.getByText("Select an archive to preview its contents"),
		).toBeInTheDocument();
	});

	it("should show loading state while probing archive", async () => {
		// Mock a delayed response
		vi.mocked(api.probeArchive).mockImplementation(
			() =>
				new Promise((resolve) =>
					setTimeout(
						() =>
							resolve({
								format: "ZIP",
								entries: 2,
								encrypted: false,
								entry_list: [],
							}),
						100,
					),
				),
		);

		render(
			<ArchivePreview archivePath="/test/archive.zip" onExtract={vi.fn()} />,
		);

		expect(screen.getByText("Loading archive contents...")).toBeInTheDocument();
	});

	it("should display archive metadata after successful probe", async () => {
		const mockArchiveInfo = {
			format: "ZIP",
			entries: 3,
			encrypted: false,
			uncompressed_estimate: 1024000,
			entry_list: [
				{ path: "file1.txt", is_directory: false, size: 512000 },
				{ path: "file2.txt", is_directory: false, size: 512000 },
				{ path: "folder/", is_directory: true, size: 0 },
			],
		};

		vi.mocked(api.probeArchive).mockResolvedValue(mockArchiveInfo);

		render(
			<ArchivePreview archivePath="/test/archive.zip" onExtract={vi.fn()} />,
		);

		await waitFor(() => {
			expect(screen.getByText("ZIP")).toBeInTheDocument();
			expect(screen.getByText(/2 files/)).toBeInTheDocument();
			expect(screen.getByText(/1 folder/)).toBeInTheDocument();
		});
	});

	it("should show password protected indicator for encrypted archives", async () => {
		const mockArchiveInfo = {
			format: "ZIP",
			entries: 1,
			encrypted: true,
			entry_list: [{ path: "secret.txt", is_directory: false, size: 1024 }],
		};

		vi.mocked(api.probeArchive).mockResolvedValue(mockArchiveInfo);

		render(
			<ArchivePreview archivePath="/test/encrypted.zip" onExtract={vi.fn()} />,
		);

		await waitFor(() => {
			expect(screen.getByText("Password Protected")).toBeInTheDocument();
		});
	});

	it("should display error state when probe fails", async () => {
		vi.mocked(api.probeArchive).mockRejectedValue(
			new Error("Corrupted archive"),
		);

		render(
			<ArchivePreview archivePath="/test/corrupted.zip" onExtract={vi.fn()} />,
		);

		await waitFor(() => {
			expect(screen.getByText("Failed to load archive")).toBeInTheDocument();
			expect(screen.getByText("Corrupted archive")).toBeInTheDocument();
			expect(
				screen.getByRole("button", { name: /retry/i }),
			).toBeInTheDocument();
		});
	});

	it("should show empty archive state when entry list is empty", async () => {
		const mockArchiveInfo = {
			format: "ZIP",
			entries: 0,
			encrypted: false,
			entry_list: [],
		};

		vi.mocked(api.probeArchive).mockResolvedValue(mockArchiveInfo);

		render(
			<ArchivePreview archivePath="/test/empty.zip" onExtract={vi.fn()} />,
		);

		await waitFor(() => {
			expect(screen.getByText("Archive is empty")).toBeInTheDocument();
		});
	});
});
