import { Button } from "@heroui/button";
import { Card, CardBody, CardHeader } from "@heroui/card";
import { Chip } from "@heroui/chip";
import { Spinner } from "@heroui/spinner";
import {
	AlertCircle,
	ChevronDown,
	ChevronRight,
	File,
	FileArchive,
	Folder,
	Lock,
	RefreshCw,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import type { ArchiveEntry, ArchiveInfo } from "../lib/api";
import { probeArchive } from "../lib/api";
import { showError, showWarning } from "../lib/toast";
import SplitExtractButton from "./SplitExtractButton";

interface ArchivePreviewProps {
	archivePath: string | null;
	onExtract: (customOutputDir?: string) => void;
}

interface TreeNode {
	name: string;
	path: string;
	isDirectory: boolean;
	size: number;
	compressedSize?: number;
	children: TreeNode[];
	isExpanded: boolean;
}

export default function ArchivePreview({
	archivePath,
	onExtract,
}: ArchivePreviewProps) {
	const [isLoading, setIsLoading] = useState(false);
	const [archiveInfo, setArchiveInfo] = useState<ArchiveInfo | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [tree, setTree] = useState<TreeNode[]>([]);
	const [_retryCount, setRetryCount] = useState(0);

	// Build tree structure from flat entry list
	const buildTree = useCallback((entries: ArchiveEntry[]): TreeNode[] => {
		const root: TreeNode[] = [];
		const nodeMap = new Map<string, TreeNode>();

		// Sort entries to ensure parents come before children
		const sortedEntries = [...entries].sort((a, b) =>
			a.path.localeCompare(b.path),
		);

		sortedEntries.forEach((entry) => {
			const parts = entry.path.split("/").filter(Boolean);
			const name = parts[parts.length - 1] || entry.path;

			const node: TreeNode = {
				name,
				path: entry.path,
				isDirectory: entry.is_directory,
				size: entry.size,
				compressedSize: entry.compressed_size,
				children: [],
				isExpanded: false,
			};

			nodeMap.set(entry.path, node);

			// Find parent
			if (parts.length > 1) {
				const parentPath = `${parts.slice(0, -1).join("/")}/`;
				const parent = nodeMap.get(parentPath);
				if (parent) {
					parent.children.push(node);
				} else {
					// Parent not found, add to root
					root.push(node);
				}
			} else {
				root.push(node);
			}
		});

		return root;
	}, []);

	// Load archive contents when path changes
	useEffect(() => {
		if (!archivePath) {
			setArchiveInfo(null);
			setTree([]);
			setError(null);
			setRetryCount(0);
			return;
		}

		const loadArchive = async () => {
			setIsLoading(true);
			setError(null);
			try {
				const info = await probeArchive(archivePath);
				setArchiveInfo(info);

				// Build tree from entries
				const treeNodes = buildTree(info.entry_list);
				setTree(treeNodes);
				setRetryCount(0); // Reset retry count on success

				// Show warning if archive is encrypted
				if (info.encrypted) {
					showWarning(
						"This archive is password-protected. You will be prompted for a password during extraction.",
					);
				}
			} catch (err) {
				const errorMsg =
					err instanceof Error ? err.message : "Failed to load archive";
				setError(errorMsg);
				setArchiveInfo(null);
				setTree([]);

				// Show user-friendly error toast
				if (errorMsg.includes("corrupted") || errorMsg.includes("Corrupted")) {
					showError("Archive appears to be corrupted or damaged");
				} else if (
					errorMsg.includes("Unsupported") ||
					errorMsg.includes("format")
				) {
					showError("Unsupported archive format");
				} else if (
					errorMsg.includes("not found") ||
					errorMsg.includes("does not exist")
				) {
					showError("Archive file not found");
				} else if (
					errorMsg.includes("Permission denied") ||
					errorMsg.includes("permission")
				) {
					showError("Permission denied: Cannot read archive file");
				} else {
					showError(`Failed to load archive: ${errorMsg}`);
				}
			} finally {
				setIsLoading(false);
			}
		};

		loadArchive();
	}, [archivePath, buildTree]);

	const toggleFolder = useCallback((nodePath: number[]) => {
		setTree((prevTree) => {
			// Helper function to deeply clone and update a node
			const updateNodeAtPath = (
				nodes: TreeNode[],
				path: number[],
				depth: number,
			): TreeNode[] => {
				return nodes.map((node, index) => {
					if (depth === path.length - 1 && index === path[depth]) {
						// This is the node to toggle
						return {
							...node,
							isExpanded: !node.isExpanded,
						};
					} else if (depth < path.length - 1 && index === path[depth]) {
						// This is a parent node in the path, recurse into children
						return {
							...node,
							children: updateNodeAtPath(node.children, path, depth + 1),
						};
					}
					return node;
				});
			};

			return updateNodeAtPath(prevTree, nodePath, 0);
		});
	}, []);

	const formatFileSize = (bytes: number): string => {
		if (bytes === 0) return "0 B";
		const k = 1024;
		const sizes = ["B", "KB", "MB", "GB"];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${(bytes / k ** i).toFixed(1)} ${sizes[i]}`;
	};

	const countFiles = (nodes: TreeNode[]): number => {
		let count = 0;
		nodes.forEach((node) => {
			if (!node.isDirectory) {
				count++;
			}
			if (node.children.length > 0) {
				count += countFiles(node.children);
			}
		});
		return count;
	};

	const countFolders = (nodes: TreeNode[]): number => {
		let count = 0;
		nodes.forEach((node) => {
			if (node.isDirectory) {
				count++;
			}
			if (node.children.length > 0) {
				count += countFolders(node.children);
			}
		});
		return count;
	};

	const renderTreeNode = (
		node: TreeNode,
		depth: number,
		nodePath: number[],
	) => {
		const paddingLeft = `${depth * 20 + 8}px`;

		const ariaProps: Record<string, string | undefined> = {
			"aria-label": `${node.isDirectory ? "Folder" : "File"}: ${node.name}`,
		};

		if (node.isDirectory) {
			ariaProps["aria-expanded"] = node.isExpanded ? "true" : "false";
		}

		return (
			<div key={node.path}>
				<button
					type="button"
					className="w-full flex items-center gap-2 px-2 py-1.5 cursor-pointer border-0 bg-transparent text-left hover:bg-default-100 dark:hover:bg-default-50 focus:ring-2 focus:ring-primary focus:ring-inset"
					style={{ paddingLeft }}
					onClick={() => node.isDirectory && toggleFolder(nodePath)}
					{...ariaProps}
					disabled={!node.isDirectory}
				>
					{/* Expand/collapse icon for directories */}
					{node.isDirectory ? (
						<div className="w-4 h-4 flex-shrink-0">
							{node.isExpanded ? (
								<ChevronDown className="w-4 h-4 text-default-500" />
							) : (
								<ChevronRight className="w-4 h-4 text-default-500" />
							)}
						</div>
					) : (
						<div className="w-4" />
					)}

					{/* File/folder icon */}
					{node.isDirectory ? (
						<Folder className="w-4 h-4 flex-shrink-0 text-default-500" />
					) : (
						<File className="w-4 h-4 flex-shrink-0 text-default-400" />
					)}

					{/* File name */}
					<span className="text-sm truncate text-default-700">{node.name}</span>

					{/* File size */}
					{!node.isDirectory && (
						<span className="text-xs text-default-400 ml-auto flex-shrink-0">
							{formatFileSize(node.size)}
						</span>
					)}
				</button>

				{/* Render children if expanded */}
				{node.isExpanded && node.children.length > 0 && (
					<div>
						{node.children.map((child, childIndex) => {
							const childPath = [...nodePath, childIndex];
							return renderTreeNode(child, depth + 1, childPath);
						})}
					</div>
				)}
			</div>
		);
	};

	const renderMetadata = () => {
		if (!archiveInfo) return null;

		const fileCount = countFiles(tree);
		const folderCount = countFolders(tree);

		return (
			<div className="flex flex-wrap gap-2">
				<Chip size="sm" variant="flat" color="default">
					{archiveInfo.format}
				</Chip>
				{archiveInfo.encrypted && (
					<Chip
						size="sm"
						variant="flat"
						color="warning"
						startContent={<Lock className="w-3 h-3" />}
					>
						Password Protected
					</Chip>
				)}
				<Chip size="sm" variant="flat" color="default">
					{fileCount} {fileCount === 1 ? "file" : "files"}
				</Chip>
				<Chip size="sm" variant="flat" color="default">
					{folderCount} {folderCount === 1 ? "folder" : "folders"}
				</Chip>
				{archiveInfo.uncompressed_estimate !== undefined && (
					<Chip size="sm" variant="flat" color="default">
						{formatFileSize(archiveInfo.uncompressed_estimate)}
					</Chip>
				)}
			</div>
		);
	};

	const renderContent = () => {
		if (!archivePath) {
			return (
				<div className="flex flex-col items-center justify-center h-full text-default-400">
					<FileArchive className="w-16 h-16 mb-4 opacity-50" />
					<p className="text-sm">Select an archive to preview its contents</p>
				</div>
			);
		}

		if (isLoading) {
			return (
				<div className="flex flex-col items-center justify-center h-full">
					<Spinner size="lg" />
					<p className="text-sm text-default-500 mt-4">
						Loading archive contents...
					</p>
				</div>
			);
		}

		if (error) {
			return (
				<div className="flex flex-col items-center justify-center h-full text-danger p-4">
					<AlertCircle className="w-16 h-16 mb-4 opacity-50" />
					<p className="text-sm font-medium mb-2">Failed to load archive</p>
					<p className="text-xs text-default-500 text-center mb-4">{error}</p>
					<Button
						size="sm"
						variant="flat"
						color="danger"
						startContent={<RefreshCw className="w-4 h-4" />}
						onPress={() => setRetryCount((prev) => prev + 1)}
					>
						Retry
					</Button>
				</div>
			);
		}

		if (!archiveInfo || tree.length === 0) {
			return (
				<div className="flex flex-col items-center justify-center h-full text-default-400">
					<FileArchive className="w-16 h-16 mb-4 opacity-50" />
					<p className="text-sm">Archive is empty</p>
				</div>
			);
		}

		return (
			<div aria-label="Archive contents tree">
				{tree.map((node, index) => renderTreeNode(node, 0, [index]))}
			</div>
		);
	};

	const getArchiveName = () => {
		if (!archivePath) return "Archive Preview";
		const parts = archivePath.split("/");
		return parts[parts.length - 1] || "Archive Preview";
	};

	return (
		<Card className="h-full flex flex-col overflow-hidden">
			<CardHeader className="pb-2 flex-shrink-0">
				<div className="flex items-center justify-between w-full">
					<h3 className="text-lg font-semibold truncate flex-1">
						{archivePath ? getArchiveName() : "Archive Preview"}
					</h3>
					{archivePath && archiveInfo && (
						<div className="ml-2 flex-shrink-0">
							<SplitExtractButton onExtract={onExtract} />
						</div>
					)}
				</div>
			</CardHeader>

			{/* Fixed Metadata Chips */}
			{archivePath && archiveInfo && (
				<div className="px-4 py-2 border-b border-divider flex-shrink-0">
					{renderMetadata()}
				</div>
			)}

			<CardBody className="flex-1 overflow-y-auto p-4">
				{renderContent()}
			</CardBody>
		</Card>
	);
}
