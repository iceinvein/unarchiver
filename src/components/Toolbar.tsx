import { Button } from "@heroui/button";
import { Tooltip } from "@heroui/tooltip";
import { useStore } from "@nanostores/react";
import { PackageOpen, Settings as SettingsIcon } from "lucide-react";
import { queueMap, selectedArchiveAtom } from "../lib/store";

interface ToolbarProps {
	onExtract: () => void;
	onOpenSettings: () => void;
}

export default function Toolbar({ onExtract, onOpenSettings }: ToolbarProps) {
	const selectedArchive = useStore(selectedArchiveAtom);
	const queue = useStore(queueMap);

	// Check if any extraction is in progress
	const isExtracting = Object.values(queue).some(
		(item) => item.status === "extracting" || item.status === "pending",
	);

	// Extract button is enabled only when an archive is selected and not currently extracting
	const canExtract = selectedArchive !== null && !isExtracting;

	return (
		<div className="flex items-center justify-between px-4 py-3 border-b border-divider bg-content1">
			<div className="flex items-center gap-2">
				<Tooltip
					content={
						canExtract ? "Extract archive (⌘E)" : "Select an archive to extract"
					}
					placement="bottom"
				>
					<Button
						color="primary"
						startContent={<PackageOpen className="w-4 h-4" />}
						onPress={onExtract}
						isDisabled={!canExtract}
						isLoading={isExtracting}
						aria-label="Extract archive"
						aria-keyshortcuts="Meta+E"
					>
						{isExtracting ? "Extracting..." : "Extract"}
					</Button>
				</Tooltip>
			</div>

			<div className="flex items-center gap-2">
				<Tooltip content="Settings (⌘,)" placement="bottom">
					<Button
						variant="light"
						isIconOnly
						onPress={onOpenSettings}
						aria-label="Open settings"
						aria-keyshortcuts="Meta+Comma"
					>
						<SettingsIcon className="w-5 h-5" />
					</Button>
				</Tooltip>
			</div>
		</div>
	);
}
