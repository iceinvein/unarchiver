import { useState } from 'react';
import { useStore } from '@nanostores/react';
import { Card, CardBody } from '@heroui/card';
import { Button } from '@heroui/button';
import { Upload, FolderOpen, FileArchive } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { settingsAtom, addToQueue } from '../lib/store';
import { probeArchive, extractArchives } from '../lib/api';
import type { QueueItem } from '../lib/types';

export default function DropZone() {
  const [isDragOver, setIsDragOver] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const settings = useStore(settingsAtom);

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);

    // Note: In Tauri, file drops are handled via the file-drop event
    // This is a fallback for web-based drag and drop
    // The actual file paths will be handled by Tauri's event system
    console.log('Files dropped:', e.dataTransfer.files);
  };

  const handleFilePicker = async () => {
    try {
      const selected = await open({
        multiple: true,
        title: 'Select Archive Files',
        filters: [
          {
            name: 'Archives',
            extensions: ['zip', '7z', 'rar', 'tar', 'gz', 'bz2', 'xz', 'tgz', 'tbz2', 'txz', 'iso'],
          },
        ],
      });

      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        console.log('Selected archives:', paths);
        await processFiles(paths);
      }
    } catch (error) {
      console.error('File picker error:', error);
    }
  };

  const handleOutputDirPicker = async () => {
    try {
      console.log('Opening directory picker...');
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Output Directory',
      });

      console.log('Selected directory:', selected);
      if (selected && typeof selected === 'string') {
        return selected;
      }
    } catch (error) {
      console.error('Directory picker error:', error);
    }
    return null;
  };

  const processFiles = async (paths: string[]) => {
    console.log('processFiles called with:', paths);
    if (paths.length === 0) return;

    setIsProcessing(true);

    try {
      // Get output directory from user
      const outputDir = await handleOutputDirPicker();
      console.log('Output directory selected:', outputDir);
      if (!outputDir) {
        console.log('No output directory selected, cancelling');
        setIsProcessing(false);
        return;
      }

      // Process each archive
      for (const path of paths) {
        console.log('Processing archive:', path);
        try {
          // Probe the archive to get metadata
          console.log('Probing archive...');
          const info = await probeArchive(path);
          console.log('Probe result:', info);
          
          // Start extraction first to get the job ID
          console.log('Starting extraction with settings:', settings);
          const jobId = await extractArchives([path], outputDir, settings);
          console.log('Extraction started, job ID:', jobId);
          
          // Create and add queue item with the actual job ID
          const queueItem: QueueItem = {
            id: jobId,
            archivePath: path,
            outputDir,
            status: 'pending',
          };
          addToQueue(queueItem);

          console.log(`Started extraction for ${path}`, info);
        } catch (error) {
          console.error(`Failed to process ${path}:`, error);
          
          // Get detailed error message
          let errorMessage = 'Unknown error';
          if (error instanceof Error) {
            errorMessage = error.message;
          } else if (typeof error === 'string') {
            errorMessage = error;
          } else if (error && typeof error === 'object') {
            errorMessage = JSON.stringify(error);
          }
          
          console.error('Detailed error:', errorMessage);
          
          // Add failed item to queue
          const queueItem: QueueItem = {
            id: crypto.randomUUID(),
            archivePath: path,
            outputDir,
            status: 'failed',
            error: errorMessage,
          };
          addToQueue(queueItem);
        }
      }
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <Card
      className={`border-2 border-dashed transition-colors ${
        isDragOver
          ? 'border-primary bg-primary-50 dark:bg-primary-950'
          : 'border-default-300 bg-default-50'
      }`}
    >
      <CardBody
        className="flex flex-col items-center justify-center py-12 px-6 text-center"
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        <div className="mb-4">
          {isDragOver ? (
            <FileArchive className="w-16 h-16 text-primary animate-bounce" />
          ) : (
            <Upload className="w-16 h-16 text-default-400" />
          )}
        </div>
        
        <h3 className="text-xl font-semibold mb-2">
          {isDragOver ? 'Drop archives here' : 'Extract Archives'}
        </h3>
        
        <p className="text-default-500 mb-6 max-w-md">
          Drag and drop archive files here, or use the buttons below to select files
        </p>

        <div className="flex gap-3">
          <Button
            color="primary"
            startContent={<FileArchive className="w-4 h-4" />}
            onPress={handleFilePicker}
            isLoading={isProcessing}
          >
            Select Archives
          </Button>
          
          <Button
            color="default"
            variant="bordered"
            startContent={<FolderOpen className="w-4 h-4" />}
            onPress={handleOutputDirPicker}
            isDisabled={isProcessing}
          >
            Choose Output Directory
          </Button>
        </div>

        <p className="text-xs text-default-400 mt-6">
          Supported formats: ZIP, 7Z, RAR, TAR, GZ, BZ2, XZ, ISO
        </p>
      </CardBody>
    </Card>
  );
}
