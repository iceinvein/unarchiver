import { Button } from "@heroui/button";
import { Divider } from "@heroui/divider";
import { Input } from "@heroui/input";
import {
	Modal,
	ModalBody,
	ModalContent,
	ModalFooter,
	ModalHeader,
} from "@heroui/modal";
import { Select, SelectItem } from "@heroui/select";
import { Switch } from "@heroui/switch";
import { useStore } from "@nanostores/react";
import { invoke } from "@tauri-apps/api/core";
import { RotateCcw, Settings as SettingsIcon } from "lucide-react";
import { useEffect } from "react";
import {
	resetSettings,
	setTheme,
	settingsAtom,
	settingsModalAtom,
	themeAtom,
	updateSettings,
} from "../lib/store";
import type { OverwriteMode, Theme } from "../lib/types";

export default function Settings() {
	const settings = useStore(settingsAtom);
	const theme = useStore(themeAtom);
	const isOpen = useStore(settingsModalAtom);

	// Load settings on mount
	useEffect(() => {
		const loadSettings = async () => {
			try {
				const loaded = await invoke<Record<string, unknown>>("load_settings");
				if (loaded) {
					updateSettings(loaded);
				}
			} catch (error) {
				console.error("Failed to load settings:", error);
			}
		};
		loadSettings();
	}, []);

	const saveSettings = async () => {
		try {
			await invoke("save_settings", { settings });
		} catch (error) {
			console.error("Failed to save settings:", error);
		}
	};

	const handleOverwriteModeChange = (value: string) => {
		updateSettings({ overwriteMode: value as OverwriteMode });
		saveSettings();
	};

	const handleSizeLimitChange = (value: string) => {
		const numValue = parseFloat(value);
		if (!Number.isNaN(numValue) && numValue >= 0) {
			updateSettings({ sizeLimitGB: numValue });
			saveSettings();
		}
	};

	const handleStripComponentsChange = (value: string) => {
		const numValue = parseInt(value, 10);
		if (!Number.isNaN(numValue) && numValue >= 0) {
			updateSettings({ stripComponents: numValue });
			saveSettings();
		}
	};

	const handleSymlinksChange = (checked: boolean) => {
		updateSettings({ allowSymlinks: checked });
		saveSettings();
	};

	const handleHardlinksChange = (checked: boolean) => {
		updateSettings({ allowHardlinks: checked });
		saveSettings();
	};

	const handleThemeChange = (value: string) => {
		setTheme(value as Theme);
	};

	const handleReset = () => {
		resetSettings();
		saveSettings();
	};

	const handleClose = () => {
		settingsModalAtom.set(false);
	};

	return (
		<Modal
			isOpen={isOpen}
			onClose={handleClose}
			size="2xl"
			scrollBehavior="inside"
		>
			<ModalContent>
				<ModalHeader className="flex items-center gap-2">
					<SettingsIcon className="w-5 h-5" />
					<span>Settings</span>
				</ModalHeader>
				<ModalBody className="space-y-6">
					{/* Extraction Options */}
					<div className="space-y-4">
						<h4 className="text-sm font-semibold text-default-700">
							Extraction Options
						</h4>

						<Select
							label="Overwrite Mode"
							placeholder="Select overwrite behavior"
							selectedKeys={[settings.overwriteMode]}
							onChange={(e) => handleOverwriteModeChange(e.target.value)}
							description="How to handle existing files"
						>
							<SelectItem key="replace">
								Replace - Overwrite existing files
							</SelectItem>
							<SelectItem key="skip">Skip - Keep existing files</SelectItem>
							<SelectItem key="rename">Rename - Add (1), (2), etc.</SelectItem>
						</Select>

						<Input
							type="number"
							label="Size Limit (GB)"
							placeholder="20"
							value={settings.sizeLimitGB.toString()}
							onValueChange={handleSizeLimitChange}
							description="Maximum total extraction size (0 = unlimited)"
							min={0}
							step={1}
						/>

						<Input
							type="number"
							label="Strip Components"
							placeholder="0"
							value={settings.stripComponents.toString()}
							onValueChange={handleStripComponentsChange}
							description="Remove leading path components"
							min={0}
							step={1}
						/>
					</div>

					<Divider />

					{/* Security Options */}
					<div className="space-y-4">
						<h4 className="text-sm font-semibold text-default-700">
							Security Options
						</h4>

						<Switch
							isSelected={settings.allowSymlinks}
							onValueChange={handleSymlinksChange}
						>
							<div className="flex flex-col">
								<span className="text-sm">Allow Symbolic Links</span>
								<span className="text-xs text-default-400">
									Extract symlinks (may pose security risks)
								</span>
							</div>
						</Switch>

						<Switch
							isSelected={settings.allowHardlinks}
							onValueChange={handleHardlinksChange}
						>
							<div className="flex flex-col">
								<span className="text-sm">Allow Hard Links</span>
								<span className="text-xs text-default-400">
									Extract hardlinks (may pose security risks)
								</span>
							</div>
						</Switch>
					</div>

					<Divider />

					{/* Appearance */}
					<div className="space-y-4">
						<h4 className="text-sm font-semibold text-default-700">
							Appearance
						</h4>

						<Select
							label="Theme"
							placeholder="Select theme"
							selectedKeys={[theme]}
							onChange={(e) => handleThemeChange(e.target.value)}
							description="Choose your preferred color scheme"
						>
							<SelectItem key="light">Light</SelectItem>
							<SelectItem key="dark">Dark</SelectItem>
							<SelectItem key="system">System</SelectItem>
						</Select>
					</div>
				</ModalBody>
				<ModalFooter>
					<Button
						variant="light"
						startContent={<RotateCcw className="w-4 h-4" />}
						onPress={handleReset}
					>
						Reset to Defaults
					</Button>
					<Button color="primary" onPress={handleClose}>
						Close
					</Button>
				</ModalFooter>
			</ModalContent>
		</Modal>
	);
}
