import { Button } from "@heroui/button";
import {
	Dropdown,
	DropdownItem,
	DropdownMenu,
	DropdownTrigger,
} from "@heroui/dropdown";
import { open } from "@tauri-apps/plugin-dialog";
import { ChevronDown, FolderOpen, PackageOpen } from "lucide-react";
import { useState } from "react";

interface SplitExtractButtonProps {
	onExtract: (customOutputDir?: string) => void;
	disabled?: boolean;
}

export default function SplitExtractButton({
	onExtract,
	disabled = false,
}: SplitExtractButtonProps) {
	const [isSelectingFolder, setIsSelectingFolder] = useState(false);

	const handleExtractToCustomFolder = async () => {
		setIsSelectingFolder(true);
		try {
			const selected = await open({
				directory: true,
				multiple: false,
				title: "Select extraction folder",
			});

			if (selected) {
				onExtract(selected);
			}
		} catch (error) {
			console.error("Failed to select folder:", error);
		} finally {
			setIsSelectingFolder(false);
		}
	};

	return (
		<div className="flex items-center gap-0">
			{/* Main extract button */}
			<Button
				color="primary"
				size="sm"
				onPress={() => onExtract()}
				isDisabled={disabled || isSelectingFolder}
				aria-label="Extract archive to default location"
				className="rounded-r-none"
			>
				<PackageOpen className="w-4 h-4" />
				Extract
			</Button>

			{/* Dropdown trigger button */}
			<Dropdown placement="bottom-end">
				<DropdownTrigger>
					<Button
						color="primary"
						size="sm"
						isIconOnly
						isDisabled={disabled || isSelectingFolder}
						aria-label="Extract options"
						className="rounded-l-none border-l border-primary-600/30 min-w-6 w-6 px-0"
					>
						<ChevronDown className="w-3 h-3" />
					</Button>
				</DropdownTrigger>
				<DropdownMenu aria-label="Extract options">
					<DropdownItem
						key="custom-folder"
						startContent={<FolderOpen className="w-4 h-4" />}
						onPress={handleExtractToCustomFolder}
					>
						Extract to custom folder...
					</DropdownItem>
				</DropdownMenu>
			</Dropdown>
		</div>
	);
}
