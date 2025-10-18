import { Button } from "@heroui/button";
import { Card, CardBody, CardHeader } from "@heroui/card";
import { Spinner } from "@heroui/spinner";
import { open } from "@tauri-apps/plugin-dialog";
import {
	AlertCircle,
	ChevronDown,
	ChevronLeft,
	ChevronRight,
	FileArchive,
	Folder,
	FolderOpen,
	Home,
	PackageOpen,
} from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import {
	getHomeDirectory,
	listDirectory,
	requestFolderAccess,
} from "../lib/api";
import type { FileSystemEntry } from "../lib/bindings/FileSystemEntry";
import { showError } from "../lib/toast";

interface FileExplorerProps {
	currentPath: string;
	onPathChange: (path: string) => void;
	selectedArchive: string | null;
	onArchiveSelect: (path: string | null) => void;
	onExtract: (customOutputDir?: string) => void;
}

interface TreeNode {
	entry: FileSystemEntry;
	isExpanded: boolean;
	children: TreeNode[];
	isLoading: boolean;
}

export default function FileExplorer({
	currentPath,
	onPathChange,
	selectedArchive,
	onArchiveSelect,
	onExtract,
}: FileExplorerProps) {
	const [rootPath, setRootPath] = useState<string>("");
	const [tree, setTree] = useState<TreeNode[]>([]);
	const [isLoading, setIsLoading] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const [focusedIndex, setFocusedIndex] = useState<number>(-1);
	const [contextMenu, setContextMenu] = useState<{
		x: number;
		y: number;
		archivePath: string;
	} | null>(null);
	const treeContainerRef = useRef<HTMLDivElement>(null);
	const focusedItemRef = useRef<HTMLButtonElement>(null);

	// Close context menu on click outside
	useEffect(() => {
		const handleClick = () => setContextMenu(null);
		if (contextMenu) {
			document.addEventListener("click", handleClick);
			return () => document.removeEventListener("click", handleClick);
		}
	}, [contextMenu]);

	// Initialize - don't auto-load any directory to avoid permission prompts
	useEffect(() => {
		const initializeHome = async () => {
			try {
				// Try to get home directory path without accessing it
				const homePath = await getHomeDirectory();
				setRootPath(homePath);
				// Don't call onPathChange here - wait for user to select a folder
			} catch (err) {
				const errorMsg =
					err instanceof Error ? err.message : "Failed to get home directory";
				console.error("Failed to get home directory:", errorMsg);
				// Set a default root path
				setRootPath("/");
			}
		};
		initializeHome();
	}, []);

	const navigateUp = useCallback(() => {
		if (!currentPath || currentPath === rootPath) return;

		const parentPath = currentPath.split("/").slice(0, -1).join("/");
		if (parentPath) {
			onPathChange(parentPath);
			onArchiveSelect(null);
		}
	}, [currentPath, rootPath, onPathChange, onArchiveSelect]);

	// Load directory contents when path changes
	useEffect(() => {
		if (!currentPath) return;

		const loadDirectory = async () => {
			setIsLoading(true);
			setError(null);
			try {
				const entries = await listDirectory(currentPath);
				// Filter: hide hidden files and show only directories and archives
				const visibleEntries = entries.filter(
					(entry) =>
						!entry.name.startsWith(".") &&
						(entry.isDirectory || entry.isArchive),
				);
				const nodes: TreeNode[] = visibleEntries.map((entry) => ({
					entry,
					isExpanded: false,
					children: [],
					isLoading: false,
				}));
				setTree(nodes);
			} catch (err) {
				const errorMsg =
					err instanceof Error ? err.message : "Failed to load directory";
				setError(errorMsg);
				setTree([]);

				// Handle specific error cases
				if (
					errorMsg.includes("PERMISSION_DENIED") ||
					errorMsg.includes("Permission denied") ||
					errorMsg.includes("permission") ||
					errorMsg.includes("Access denied")
				) {
					setError("PERMISSION_DENIED");
					showError(
						"Permission denied: You don't have access to this directory. Click 'Request Access' to grant permission.",
					);
				} else if (
					errorMsg.includes("does not exist") ||
					errorMsg.includes("not found")
				) {
					showError("Directory not found. Navigating to parent directory...");
					// Navigate to parent directory after a short delay
					setTimeout(() => {
						navigateUp();
					}, 1500);
				} else {
					showError(`Failed to load directory: ${errorMsg}`);
				}
			} finally {
				setIsLoading(false);
			}
		};

		loadDirectory();

		// Auto-refresh: Poll for changes every 3 seconds
		const refreshInterval = setInterval(async () => {
			try {
				const entries = await listDirectory(currentPath);
				const visibleEntries = entries.filter(
					(entry) =>
						!entry.name.startsWith(".") &&
						(entry.isDirectory || entry.isArchive),
				);

				// Check if the directory contents have changed
				setTree((prevTree) => {
					// Compare entry counts and names
					if (prevTree.length !== visibleEntries.length) {
						// Directory changed, update tree
						return visibleEntries.map((entry) => ({
							entry,
							isExpanded: false,
							children: [],
							isLoading: false,
						}));
					}

					// Check if any entries are different
					const hasChanges = visibleEntries.some((newEntry, index) => {
						const oldEntry = prevTree[index]?.entry;
						return (
							!oldEntry ||
							oldEntry.name !== newEntry.name ||
							oldEntry.modifiedAt !== newEntry.modifiedAt ||
							oldEntry.size !== newEntry.size
						);
					});

					if (hasChanges) {
						// Directory changed, update tree
						return visibleEntries.map((entry) => ({
							entry,
							isExpanded: false,
							children: [],
							isLoading: false,
						}));
					}

					// No changes, keep existing tree
					return prevTree;
				});
			} catch (err) {
				// Silently fail on refresh errors to avoid spamming the user
				console.error("Failed to refresh directory:", err);
			}
		}, 3000); // Refresh every 3 seconds

		return () => {
			clearInterval(refreshInterval);
		};
	}, [currentPath, navigateUp]);

	const loadChildren = useCallback(
		async (node: TreeNode): Promise<TreeNode[]> => {
			if (!node.entry.isDirectory) return [];

			try {
				const entries = await listDirectory(node.entry.path);
				// Filter: hide hidden files and show only directories and archives
				const visibleEntries = entries.filter(
					(entry) =>
						!entry.name.startsWith(".") &&
						(entry.isDirectory || entry.isArchive),
				);
				return visibleEntries.map((entry) => ({
					entry,
					isExpanded: false,
					children: [],
					isLoading: false,
				}));
			} catch (err) {
				const errorMsg =
					err instanceof Error ? err.message : "Failed to load folder";
				console.error("Failed to load children:", err);

				// Show user-friendly error message
				if (
					errorMsg.includes("Permission denied") ||
					errorMsg.includes("permission")
				) {
					showError(`Cannot access "${node.entry.name}": Permission denied`);
				} else {
					showError(`Failed to load folder "${node.entry.name}"`);
				}

				return [];
			}
		},
		[],
	);

	const toggleFolder = useCallback(
		async (nodePath: string[]) => {
			console.log("toggleFolder called with path:", nodePath);
			
			// Helper function to deeply clone and update a node
			const updateNodeAtPath = (
				nodes: TreeNode[],
				path: string[],
				depth: number,
			): TreeNode[] => {
				return nodes.map((node, index) => {
					if (
						depth === path.length - 1 &&
						index === parseInt(path[depth], 10)
					) {
						// This is the node to toggle
						const newNode = {
							...node,
							isExpanded: !node.isExpanded,
							isLoading: !node.isExpanded && node.children.length === 0,
						};

						console.log(
							"Toggling node:",
							node.entry.name,
							"from",
							node.isExpanded,
							"to",
							newNode.isExpanded,
						);

						// Load children if expanding and not loaded yet
						if (newNode.isExpanded && node.children.length === 0) {
							loadChildren(node).then((children) => {
								// Single update with loaded children
								setTree((prevTree) => {
									const updateWithChildren = (
										nodes: TreeNode[],
										path: string[],
										depth: number,
									): TreeNode[] => {
										return nodes.map((n, i) => {
											if (
												depth === path.length - 1 &&
												i === parseInt(path[depth], 10)
											) {
												return { ...n, children, isLoading: false };
											}
											if (
												depth < path.length - 1 &&
												i === parseInt(path[depth], 10)
											) {
												return {
													...n,
													children: updateWithChildren(
														n.children,
														path,
														depth + 1,
													),
												};
											}
											return n;
										});
									};
									return updateWithChildren(prevTree, path, 0);
								});
							});
						}

						return newNode;
					}
					if (
						depth < path.length - 1 &&
						index === parseInt(path[depth], 10)
					) {
						// This is a parent node in the path, recurse into children
						return {
							...node,
							children: updateNodeAtPath(node.children, path, depth + 1),
						};
					}
					return node;
				});
			};

			setTree((prevTree) => {
				console.log("Previous tree:", prevTree);
				const newTree = updateNodeAtPath(prevTree, nodePath, 0);
				console.log("New tree:", newTree);
				return newTree;
			});
		},
		[loadChildren],
	);

	const handleItemClick = useCallback(
		(node: TreeNode, nodePath: string[]) => {
			console.log(
				"Item clicked:",
				node.entry.name,
				"isDirectory:",
				node.entry.isDirectory,
				"isArchive:",
				node.entry.isArchive,
			);
			if (node.entry.isArchive) {
				// Single-click selection for archives
				onArchiveSelect(
					node.entry.path === selectedArchive ? null : node.entry.path,
				);
			} else if (node.entry.isDirectory) {
				// Toggle folder expansion for directories
				console.log("Toggling folder:", nodePath);
				toggleFolder(nodePath);
			}
		},
		[selectedArchive, onArchiveSelect, toggleFolder],
	);

	const handleItemDoubleClick = useCallback(
		(node: TreeNode) => {
			if (node.entry.isDirectory && !node.entry.isArchive) {
				// Double-click navigation for folders
				onPathChange(node.entry.path);
				onArchiveSelect(null);
			}
		},
		[onPathChange, onArchiveSelect],
	);

	const navigateHome = useCallback(() => {
		if (rootPath) {
			onPathChange(rootPath);
			onArchiveSelect(null);
		}
	}, [rootPath, onPathChange, onArchiveSelect]);

	// Flatten tree for keyboard navigation
	const flattenTree = useCallback(
		(
			nodes: TreeNode[],
			path: string[] = [],
		): Array<{ node: TreeNode; path: string[] }> => {
			const result: Array<{ node: TreeNode; path: string[] }> = [];
			nodes.forEach((node, index) => {
				const nodePath = [...path, index.toString()];
				result.push({ node, path: nodePath });
				if (node.isExpanded && node.children.length > 0) {
					result.push(...flattenTree(node.children, nodePath));
				}
			});
			return result;
		},
		[],
	);

	// Keyboard navigation
	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if (!treeContainerRef.current?.contains(document.activeElement)) return;

			const flatTree = flattenTree(tree);

			switch (e.key) {
				case "ArrowDown":
					e.preventDefault();
					setFocusedIndex((prev) => Math.min(prev + 1, flatTree.length - 1));
					break;
				case "ArrowUp":
					e.preventDefault();
					setFocusedIndex((prev) => Math.max(prev - 1, 0));
					break;
				case "ArrowRight":
					e.preventDefault();
					if (focusedIndex >= 0 && focusedIndex < flatTree.length) {
						const { node, path } = flatTree[focusedIndex];
						if (node.entry.isDirectory && !node.isExpanded) {
							toggleFolder(path);
						}
					}
					break;
				case "ArrowLeft":
					e.preventDefault();
					if (focusedIndex >= 0 && focusedIndex < flatTree.length) {
						const { node, path } = flatTree[focusedIndex];
						if (node.entry.isDirectory && node.isExpanded) {
							toggleFolder(path);
						}
					}
					break;
				case "Enter":
				case " ":
					e.preventDefault();
					if (focusedIndex >= 0 && focusedIndex < flatTree.length) {
						const { node } = flatTree[focusedIndex];
						if (node.entry.isDirectory && !node.entry.isArchive) {
							handleItemDoubleClick(node);
						} else if (node.entry.isArchive) {
							onArchiveSelect(
								node.entry.path === selectedArchive ? null : node.entry.path,
							);
						}
					}
					break;
				case "Home":
					e.preventDefault();
					setFocusedIndex(0);
					break;
				case "End":
					e.preventDefault();
					setFocusedIndex(flatTree.length - 1);
					break;
			}
		};

		window.addEventListener("keydown", handleKeyDown);
		return () => window.removeEventListener("keydown", handleKeyDown);
	}, [
		tree,
		focusedIndex,
		flattenTree,
		toggleFolder,
		handleItemDoubleClick,
		selectedArchive,
		onArchiveSelect,
	]);

	// Scroll focused item into view
	useEffect(() => {
		if (focusedItemRef.current) {
			focusedItemRef.current.scrollIntoView({
				block: "nearest",
				behavior: "smooth",
			});
			focusedItemRef.current.focus();
		}
	}, []);

	const renderBreadcrumbs = () => {
		if (!currentPath) return null;

		const parts = currentPath.split("/").filter(Boolean);
		const pathSegments = parts.map((part, idx) => {
			const fullPath = `/${parts.slice(0, idx + 1).join("/")}`;
			return { name: part, path: fullPath };
		});

		return (
			<div className="flex w-full items-start gap-1 text-sm text-default-600 overflow-x-auto">
				<Button
					size="sm"
					variant="light"
					isIconOnly
					onPress={navigateHome}
					aria-label="Go to home directory"
				>
					<Home className="w-4 h-4" />
				</Button>
				{pathSegments.map((segment) => (
					<div key={segment.path} className="flex items-center gap-1">
						<ChevronRight className="w-3 h-3 text-default-400" />
						<Button
							size="sm"
							variant="light"
							onPress={() => onPathChange(segment.path)}
							className="min-w-0"
						>
							{segment.name}
						</Button>
					</div>
				))}
			</div>
		);
	};

	const renderTreeNode = (
		node: TreeNode,
		depth: number,
		nodePath: string[],
		flatIndex: number,
	) => {
		const isSelected =
			node.entry.isArchive && node.entry.path === selectedArchive;
		const isFocused = flatIndex === focusedIndex;
		const paddingLeft = depth * 20 + 8;

		const ariaProps: Record<string, string | undefined> = {
			"aria-label": `${node.entry.isDirectory ? "Folder" : node.entry.isArchive ? "Archive" : "File"}: ${node.entry.name}`,
			"aria-current": isSelected ? "true" : undefined,
		};

		if (node.entry.isDirectory) {
			ariaProps["aria-expanded"] = node.isExpanded ? "true" : "false";
		}

		return (
			<div key={node.entry.path}>
				<button
					ref={isFocused ? focusedItemRef : null}
					type="button"
					className={`
            w-full flex items-center gap-2 px-2 py-1.5 cursor-pointer border-0 bg-transparent text-left select-none
            ${isSelected ? "bg-primary-100 dark:bg-primary-900" : "hover:bg-default-100 dark:hover:bg-default-50"}
            ${isFocused ? "ring-2 ring-primary ring-inset" : ""}
          `}
					style={{ paddingLeft: `${paddingLeft}px`, userSelect: "none" }}
					onClick={() => handleItemClick(node, nodePath)}
					onDoubleClick={() => handleItemDoubleClick(node)}
					onContextMenu={(e) => {
						if (node.entry.isArchive) {
							e.preventDefault();
							e.stopPropagation();
							// Select the archive first
							onArchiveSelect(node.entry.path);
							setContextMenu({
								x: e.clientX,
								y: e.clientY,
								archivePath: node.entry.path,
							});
						}
					}}
					{...ariaProps}
					tabIndex={isFocused ? 0 : -1}
				>
					{/* Expand/collapse icon for directories */}
					{node.entry.isDirectory ? (
						<div className="w-4 h-4 flex-shrink-0">
							{node.isLoading ? (
								<Spinner size="sm" />
							) : node.isExpanded ? (
								<ChevronDown className="w-4 h-4 text-default-500" />
							) : (
								<ChevronRight className="w-4 h-4 text-default-500" />
							)}
						</div>
					) : (
						<div className="w-4 h-4 flex-shrink-0" />
					)}

					{/* File/folder icon */}
					{node.entry.isArchive ? (
						<FileArchive
							className={`w-4 h-4 flex-shrink-0 ${isSelected ? "text-primary" : "text-warning"}`}
						/>
					) : node.entry.isDirectory ? (
						node.isExpanded ? (
							<FolderOpen className="w-4 h-4 flex-shrink-0 text-primary" />
						) : (
							<Folder className="w-4 h-4 flex-shrink-0 text-default-500" />
						)
					) : null}

					{/* File name */}
					<span
						className={`text-sm truncate ${isSelected ? "font-medium text-primary" : "text-default-700"}`}
					>
						{node.entry.name}
					</span>

					{/* File size */}
					{node.entry.size !== undefined && !node.entry.isDirectory && (
						<span className="text-xs text-default-400 ml-auto flex-shrink-0">
							{formatFileSize(node.entry.size)}
						</span>
					)}
				</button>

				{/* Render children if expanded */}
				{node.isExpanded && node.children.length > 0 && (
					<div>
						{node.children.map((child, childIndex) => {
							const childPath = [...nodePath, childIndex.toString()];
							const childFlatIndex = flattenTree(tree).findIndex(
								(item) => item.path.join(",") === childPath.join(","),
							);
							return renderTreeNode(
								child,
								depth + 1,
								childPath,
								childFlatIndex,
							);
						})}
					</div>
				)}
			</div>
		);
	};

	const formatFileSize = (bytes: number): string => {
		if (bytes === 0) return "0 B";
		const k = 1024;
		const sizes = ["B", "KB", "MB", "GB"];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${(bytes / k ** i).toFixed(1)} ${sizes[i]}`;
	};

	const flatTree = flattenTree(tree);

	return (
		<Card className="h-full flex flex-col overflow-hidden">
			<CardHeader className="flex flex-col gap-2 pb-2 flex-shrink-0">
				<div className="flex items-center justify-between w-full">
					<h3 className="text-lg font-semibold">File Explorer</h3>
					<div className="flex gap-1">
						<Button
							size="sm"
							variant="light"
							isIconOnly
							onPress={async () => {
								try {
									const selectedPath = await requestFolderAccess();
									if (selectedPath) {
										onPathChange(selectedPath);
										setError(null);
									}
								} catch (err) {
									showError("Failed to select folder");
								}
							}}
							aria-label="Select folder"
						>
							<FolderOpen className="w-4 h-4" />
						</Button>
						<Button
							size="sm"
							variant="light"
							isIconOnly
							onPress={navigateUp}
							isDisabled={!currentPath || currentPath === rootPath}
							aria-label="Go to parent directory"
						>
							<ChevronLeft className="w-4 h-4" />
						</Button>
					</div>
				</div>
				{renderBreadcrumbs()}
			</CardHeader>

			<CardBody className="flex-1 overflow-y-auto p-0">
				{!currentPath ? (
					<div className="flex flex-col items-center justify-center h-full text-default-400 p-8">
						<FolderOpen className="w-16 h-16 mb-4 opacity-50" />
						<p className="text-sm text-center mb-4">
							Select a folder to browse archives
						</p>
						<Button
							color="primary"
							variant="flat"
							startContent={<FolderOpen className="w-4 h-4" />}
							onPress={async () => {
								try {
									const selectedPath = await requestFolderAccess();
									if (selectedPath) {
										onPathChange(selectedPath);
										setError(null);
									}
								} catch (err) {
									showError("Failed to select folder");
								}
							}}
						>
							Select Folder
						</Button>
					</div>
				) : isLoading ? (
					<div className="flex items-center justify-center h-32">
						<Spinner size="lg" />
					</div>
				) : error ? (
					<div className="flex flex-col items-center justify-center h-32 text-danger p-4">
						<AlertCircle className="w-12 h-12 mb-3 opacity-50" />
						<p className="text-sm text-center mb-3">
							{error === "PERMISSION_DENIED"
								? "Access denied to this directory"
								: error}
						</p>
						<div className="flex gap-2">
							{error === "PERMISSION_DENIED" && (
								<Button
									size="sm"
									variant="flat"
									color="primary"
									onPress={async () => {
										try {
											const selectedPath = await requestFolderAccess();
											if (selectedPath) {
												onPathChange(selectedPath);
												setError(null);
											}
										} catch (err) {
											showError("Failed to request folder access");
										}
									}}
								>
									Select Different Folder
								</Button>
							)}
							<Button
								size="sm"
								variant="flat"
								color="danger"
								onPress={() => onPathChange(currentPath)}
							>
								Retry
							</Button>
							{currentPath !== rootPath && (
								<Button size="sm" variant="light" onPress={navigateUp}>
									Go to Parent
								</Button>
							)}
						</div>
					</div>
				) : tree.length === 0 ? (
					<div className="flex items-center justify-center h-32 text-default-400">
						<p className="text-sm">Empty directory</p>
					</div>
				) : (
					<div
						ref={treeContainerRef}
						aria-label="File system tree"
						tabIndex={0}
						className="select-none"
					>
						{tree.map((node, index) => {
							const nodePath = [index.toString()];
							const flatIndex = flatTree.findIndex(
								(item) => item.path.join(",") === nodePath.join(","),
							);
							return renderTreeNode(node, 0, nodePath, flatIndex);
						})}
					</div>
				)}
			</CardBody>

			{/* Context Menu */}
			{contextMenu && (
				<div
					className="fixed bg-content1 border border-divider rounded-lg shadow-lg py-1 z-50 min-w-[180px]"
					style={{
						left: `${contextMenu.x + 5}px`,
						top: `${contextMenu.y + 5}px`,
					}}
					onClick={(e) => e.stopPropagation()}
				>
					<button
						type="button"
						className="w-full px-4 py-2 text-left text-sm hover:bg-default-100 flex items-center gap-2 border-0 bg-transparent cursor-pointer"
						onClick={() => {
							onArchiveSelect(contextMenu.archivePath);
							onExtract();
							setContextMenu(null);
						}}
					>
						<PackageOpen className="w-4 h-4" />
						Extract Here
					</button>
					<button
						type="button"
						className="w-full px-4 py-2 text-left text-sm hover:bg-default-100 flex items-center gap-2 border-0 bg-transparent cursor-pointer"
						onClick={async () => {
							const archivePath = contextMenu.archivePath;
							setContextMenu(null);

							try {
								const selected = await open({
									directory: true,
									multiple: false,
									title: "Select extraction folder",
								});

								if (selected) {
									onArchiveSelect(archivePath);
									onExtract(selected);
								}
							} catch (error) {
								console.error("Failed to select folder:", error);
							}
						}}
					>
						<FolderOpen className="w-4 h-4" />
						Extract to Folder...
					</button>
				</div>
			)}
		</Card>
	);
}
