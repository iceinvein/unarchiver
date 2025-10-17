import { Badge } from "@heroui/badge";
import { Button } from "@heroui/button";
import {
	Drawer,
	DrawerBody,
	DrawerContent,
	DrawerHeader,
} from "@heroui/drawer";
import { Navbar, NavbarBrand, NavbarContent, NavbarItem } from "@heroui/navbar";
import { useStore } from "@nanostores/react";
import {
	Archive,
	List,
	Monitor,
	Moon,
	Settings as SettingsIcon,
	Sun,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import MainLayout from "./components/MainLayout";
import PasswordPrompt from "./components/PasswordPrompt";
import QueueList from "./components/QueueList";
import Settings from "./components/Settings";
import ToastContainer from "./components/ToastContainer";
import type {
	CompletionEvent,
	PasswordRequiredEvent,
	ProgressEvent,
} from "./lib/api";
import {
	onCompletion,
	onFilesOpened,
	onPasswordRequired,
	onProgress,
} from "./lib/api";
import {
	queueMap,
	setTheme,
	settingsModalAtom,
	themeAtom,
	updateQueueItem,
} from "./lib/store";
import { showError, showSuccess } from "./lib/toast";

function App() {
	const theme = useStore(themeAtom);
	const queue = useStore(queueMap);
	const [passwordPrompt, setPasswordPrompt] = useState<{
		isOpen: boolean;
		jobId: string;
		archivePath: string;
	}>({
		isOpen: false,
		jobId: "",
		archivePath: "",
	});
	const [isQueueDrawerOpen, setIsQueueDrawerOpen] = useState(false);
	const completedJobsRef = useRef<Set<string>>(new Set());

	// Count active queue items (pending or extracting)
	const activeQueueCount = Object.values(queue).filter(
		(item) => item.status === "pending" || item.status === "extracting",
	).length;

	// Set up event listeners
	useEffect(() => {
		let unlistenProgress: (() => void) | undefined;
		let unlistenCompletion: (() => void) | undefined;
		let unlistenPassword: (() => void) | undefined;
		let unlistenFilesOpened: (() => void) | undefined;

		const setupListeners = async () => {
			console.log("Setting up event listeners...");

			// Progress events
			unlistenProgress = await onProgress((event: ProgressEvent) => {
				console.log("Progress event received:", event);
				updateQueueItem(event.jobId, {
					status: "extracting",
					progress: {
						currentFile: event.currentFile,
						bytesWritten: event.bytesWritten,
						totalBytes: event.totalBytes,
						filesExtracted: 0, // Not provided by backend
					},
				});
			});

			// Completion events
			unlistenCompletion = await onCompletion((event: CompletionEvent) => {
				console.log("Completion event received:", event);
				console.log(
					"Event status type:",
					typeof event.status,
					"value:",
					event.status,
				);

				// Check if we've already processed this completion
				if (completedJobsRef.current.has(event.jobId)) {
					console.log(
						"Duplicate completion event ignored for job:",
						event.jobId,
					);
					return;
				}

				// Mark this job as completed
				completedJobsRef.current.add(event.jobId);

				// Handle status - it might be an object with a variant
				let statusStr: string;
				if (typeof event.status === "string") {
					statusStr = event.status.toLowerCase();
				} else if (typeof event.status === "object" && event.status !== null) {
					// Handle enum-like object from Rust
					statusStr = String(event.status).toLowerCase();
				} else {
					statusStr = "failed";
				}

				const status =
					statusStr === "success"
						? "completed"
						: statusStr === "cancelled"
							? "cancelled"
							: "failed";

				console.log("Mapped status:", status);

				updateQueueItem(event.jobId, {
					status,
					stats: event.stats,
					error: event.error || undefined,
					progress: undefined,
				});

				// Show toast notification based on status
				console.log("About to show toast for status:", status);
				if (status === "completed") {
					const archiveName = event.archivePath.split("/").pop() || "Archive";
					console.log("Showing success toast for:", archiveName);
					showSuccess(`Successfully extracted: ${archiveName}`);
				} else if (status === "failed") {
					const archiveName = event.archivePath.split("/").pop() || "Archive";
					const errorMsg = event.error || "Unknown error";
					console.log("Showing error toast for:", archiveName);
					showError(`Extraction failed for ${archiveName}: ${errorMsg}`);
				}
			});

			// Password required events
			unlistenPassword = await onPasswordRequired(
				(event: PasswordRequiredEvent) => {
					setPasswordPrompt({
						isOpen: true,
						jobId: event.jobId,
						archivePath: event.archivePath,
					});
				},
			);

			// Files opened from Finder (drag-and-drop to app window or icon)
			unlistenFilesOpened = await onFilesOpened(async (paths: string[]) => {
				console.log("files_opened event received with paths:", paths);

				// Import dynamically to avoid circular dependencies
				const { currentDirectoryAtom, selectedArchiveAtom } = await import(
					"./lib/store"
				);

				// Process the first archive (if multiple files are opened, just handle the first one)
				if (paths.length > 0) {
					const archivePath = paths[0];
					console.log("Processing archive:", archivePath);

					// Get the directory containing the archive
					const directory = archivePath.substring(
						0,
						archivePath.lastIndexOf("/"),
					);
					console.log("Navigating to directory:", directory);

					// Navigate to the directory
					currentDirectoryAtom.set(directory);

					// Select the archive (this will show it in the preview)
					selectedArchiveAtom.set(archivePath);
					console.log("Archive selected:", archivePath);

					const archiveName = archivePath.split("/").pop() || "Archive";
					showSuccess(`Opened: ${archiveName}`);
				}
			});
		};

		setupListeners();

		return () => {
			console.log("Cleaning up event listeners...");
			unlistenProgress?.();
			unlistenCompletion?.();
			unlistenPassword?.();
			unlistenFilesOpened?.();
		};
	}, []);

	// Apply theme to document
	useEffect(() => {
		const root = document.documentElement;

		if (theme === "system") {
			const prefersDark = window.matchMedia(
				"(prefers-color-scheme: dark)",
			).matches;
			root.classList.toggle("dark", prefersDark);
		} else {
			root.classList.toggle("dark", theme === "dark");
		}
	}, [theme]);

	const handleThemeToggle = () => {
		const themes: Array<"light" | "dark" | "system"> = [
			"light",
			"dark",
			"system",
		];
		const currentIndex = themes.indexOf(theme);
		const nextTheme = themes[(currentIndex + 1) % themes.length];
		setTheme(nextTheme);
	};

	const getThemeIcon = () => {
		switch (theme) {
			case "light":
				return <Sun className="w-5 h-5" />;
			case "dark":
				return <Moon className="w-5 h-5" />;
			case "system":
				return <Monitor className="w-5 h-5" />;
		}
	};

	return (
		<div className="h-screen bg-background flex flex-col overflow-hidden">
			{/* Navbar */}
			<Navbar isBordered maxWidth="full">
				<NavbarBrand>
					<Archive className="w-6 h-6 mr-2 text-primary" />
					<p className="font-bold text-xl">Unarchive</p>
				</NavbarBrand>
				<NavbarContent justify="end">
					<NavbarItem>
						<Badge
							content={activeQueueCount}
							color="primary"
							isInvisible={activeQueueCount === 0}
							shape="circle"
						>
							<Button
								isIconOnly
								variant="light"
								onPress={() => setIsQueueDrawerOpen(true)}
								aria-label="View extraction queue"
							>
								<List className="w-5 h-5" />
							</Button>
						</Badge>
					</NavbarItem>
					<NavbarItem>
						<Button
							isIconOnly
							variant="light"
							onPress={() => settingsModalAtom.set(true)}
							aria-label="Open settings"
						>
							<SettingsIcon className="w-5 h-5" />
						</Button>
					</NavbarItem>
					<NavbarItem>
						<Button
							isIconOnly
							variant="light"
							onPress={handleThemeToggle}
							aria-label="Toggle theme"
						>
							{getThemeIcon()}
						</Button>
					</NavbarItem>
				</NavbarContent>
			</Navbar>

			{/* Main Content - Full height layout */}
			<main className="flex-1 overflow-hidden">
				<MainLayout />
			</main>

			{/* Queue Drawer - Background notification area */}
			<Drawer
				isOpen={isQueueDrawerOpen}
				onClose={() => setIsQueueDrawerOpen(false)}
				placement="right"
				size="md"
			>
				<DrawerContent>
					<DrawerHeader>
						<h2 className="text-xl font-semibold">Extraction Queue</h2>
					</DrawerHeader>
					<DrawerBody>
						<QueueList />
					</DrawerBody>
				</DrawerContent>
			</Drawer>

			{/* Password Prompt Modal */}
			<PasswordPrompt
				isOpen={passwordPrompt.isOpen}
				onClose={() => setPasswordPrompt({ ...passwordPrompt, isOpen: false })}
				jobId={passwordPrompt.jobId}
				archivePath={passwordPrompt.archivePath}
			/>

			{/* Settings Modal */}
			<Settings />

			{/* Toast Notifications */}
			<ToastContainer />
		</div>
	);
}

export default App;
