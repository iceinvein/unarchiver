import { Button } from "@heroui/button";
import {
	Modal,
	ModalBody,
	ModalContent,
	ModalFooter,
	ModalHeader,
} from "@heroui/modal";
import { FolderOpen, ShieldAlert } from "lucide-react";
import { requestFolderAccess } from "../lib/api";
import { showError, showSuccess } from "../lib/toast";

interface PermissionDialogProps {
	isOpen: boolean;
	onClose: () => void;
	onAccessGranted: (path: string) => void;
}

export default function PermissionDialog({
	isOpen,
	onClose,
	onAccessGranted,
}: PermissionDialogProps) {
	const handleRequestAccess = async () => {
		try {
			const selectedPath = await requestFolderAccess();
			if (selectedPath) {
				showSuccess("Folder access granted");
				onAccessGranted(selectedPath);
				onClose();
			}
		} catch (_err) {
			showError("Failed to request folder access");
		}
	};

	return (
		<Modal isOpen={isOpen} onClose={onClose} size="md">
			<ModalContent>
				<ModalHeader className="flex gap-2 items-center">
					<ShieldAlert className="w-5 h-5 text-warning" />
					<span>Folder Access Required</span>
				</ModalHeader>
				<ModalBody>
					<p className="text-sm text-default-600 mb-3">
						This app needs permission to access folders on your system to browse
						and extract archives.
					</p>
					<p className="text-sm text-default-600 mb-3">
						Due to macOS sandbox restrictions, you'll need to grant access to
						folders you want to browse.
					</p>
					<p className="text-sm text-default-600">
						Click "Grant Access" to select a folder and give the app permission
						to access it.
					</p>
				</ModalBody>
				<ModalFooter>
					<Button variant="light" onPress={onClose}>
						Skip
					</Button>
					<Button
						color="primary"
						onPress={handleRequestAccess}
						startContent={<FolderOpen className="w-4 h-4" />}
					>
						Grant Access
					</Button>
				</ModalFooter>
			</ModalContent>
		</Modal>
	);
}
