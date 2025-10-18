import { Button } from "@heroui/button";
import {
	Modal,
	ModalBody,
	ModalContent,
	ModalFooter,
	ModalHeader,
} from "@heroui/modal";
import { invoke } from "@tauri-apps/api/core";
import { ExternalLink, ShieldAlert } from "lucide-react";
import { showError } from "../lib/toast";

interface PermissionDialogProps {
	isOpen: boolean;
	onClose: () => void;
	onDismiss?: () => void;
}

export default function PermissionDialog({
	isOpen,
	onClose,
	onDismiss,
}: PermissionDialogProps) {
	const handleOpenSystemSettings = async () => {
		try {
			// Open macOS System Settings to Privacy & Security > Full Disk Access
			await invoke("open_system_settings");
			onClose();
		} catch (_err) {
			showError("Failed to open System Settings");
		}
	};

	const handleDismiss = () => {
		if (onDismiss) {
			onDismiss();
		}
		onClose();
	};

	return (
		<Modal isOpen={isOpen} onClose={onClose} size="lg">
			<ModalContent>
				<ModalHeader className="flex gap-2 items-center">
					<ShieldAlert className="w-5 h-5 text-warning" />
					<span>Full Disk Access Recommended</span>
				</ModalHeader>
				<ModalBody>
					<p className="text-sm text-default-600 mb-3">
						To use Unarchive as a file explorer for archives, we recommend
						granting <strong>Full Disk Access</strong> in macOS System Settings.
					</p>
					<p className="text-sm text-default-600 mb-3">
						This allows you to browse and extract archives from anywhere on your
						Mac without repeatedly selecting folders.
					</p>
					<div className="bg-default-100 rounded-lg p-4 mb-3">
						<p className="text-sm font-medium mb-2">How to grant access:</p>
						<ol className="text-sm text-default-600 space-y-1 list-decimal list-inside">
							<li>Click "Open System Settings" below</li>
							<li>Find "Unarchive" in the list</li>
							<li>Toggle the switch to enable Full Disk Access</li>
							<li>Restart the app for changes to take effect</li>
						</ol>
					</div>
					<p className="text-xs text-default-500">
						Note: You can still use the app without Full Disk Access by
						selecting individual folders when needed.
					</p>
				</ModalBody>
				<ModalFooter>
					<Button variant="light" onPress={onClose}>
						Later
					</Button>
					{onDismiss && (
						<Button variant="light" onPress={handleDismiss}>
							Don't Show Again
						</Button>
					)}
					<Button
						color="primary"
						onPress={handleOpenSystemSettings}
						startContent={<ExternalLink className="w-4 h-4" />}
					>
						Open System Settings
					</Button>
				</ModalFooter>
			</ModalContent>
		</Modal>
	);
}
