import { useState, useCallback, useRef, useEffect } from 'react';
import { useStore } from '@nanostores/react';
import { 
  currentDirectoryAtom, 
  selectedArchiveAtom,
  settingsAtom,
} from '../lib/store';
import FileExplorer from './FileExplorer';
import ArchivePreview from './ArchivePreview';
import { extractArchives, getUniqueOutputPath, probeArchive } from '../lib/api';
import { addToQueue } from '../lib/store';
import { showError, showSuccess, showWarning } from '../lib/toast';

const MIN_PANE_WIDTH = 300;

export default function MainLayout() {
  const currentDirectory = useStore(currentDirectoryAtom);
  const selectedArchive = useStore(selectedArchiveAtom);
  const settings = useStore(settingsAtom);
  
  const [leftPaneWidth, setLeftPaneWidth] = useState(40); // percentage
  const [isResizing, setIsResizing] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const handlePathChange = useCallback((path: string) => {
    currentDirectoryAtom.set(path);
  }, []);

  const handleArchiveSelect = useCallback((path: string | null) => {
    selectedArchiveAtom.set(path);
  }, []);

  const handleExtract = useCallback(async () => {
    if (!selectedArchive) return;

    try {
      // First, probe the archive to check size and validity
      let archiveInfo;
      try {
        archiveInfo = await probeArchive(selectedArchive);
      } catch (probeError) {
        const errorMsg = probeError instanceof Error ? probeError.message : 'Failed to read archive';
        showError(`Cannot extract: ${errorMsg}`);
        return;
      }

      // Check if archive is too large (basic check - actual disk space check would need native API)
      const estimatedSize = archiveInfo.uncompressed_estimate || 0;
      const sizeLimitBytes = settings.sizeLimitGB > 0 ? settings.sizeLimitGB * 1024 * 1024 * 1024 : Infinity;
      
      if (estimatedSize > sizeLimitBytes) {
        showError(`Archive size (${formatBytes(estimatedSize)}) exceeds the configured limit (${settings.sizeLimitGB} GB)`);
        return;
      }

      // Warn if archive is very large
      if (estimatedSize > 10 * 1024 * 1024 * 1024) { // > 10 GB
        showWarning(`Large archive detected (${formatBytes(estimatedSize)}). Extraction may take a while.`);
      }

      // Get unique output path with conflict resolution
      const outputDir = await getUniqueOutputPath(selectedArchive);

      // Start extraction with automatic output path
      const jobId = await extractArchives([selectedArchive], outputDir, settings);
      
      // Create queue item with the actual job ID
      const queueItem = {
        id: jobId,
        archivePath: selectedArchive,
        outputDir,
        status: 'pending' as const,
      };

      // Add to queue
      addToQueue(queueItem);
      
      showSuccess(`Extraction started: ${outputDir.split('/').pop()}`);
    } catch (error) {
      console.error('Failed to start extraction:', error);
      const errorMsg = error instanceof Error ? error.message : 'Unknown error';
      
      // Show user-friendly error message
      if (errorMsg.includes('Permission denied') || errorMsg.includes('permission')) {
        showError('Permission denied: Cannot create output directory');
      } else if (errorMsg.includes('disk space') || errorMsg.includes('space')) {
        showError('Insufficient disk space for extraction');
      } else if (errorMsg.includes('not found')) {
        showError('Archive file not found');
      } else {
        showError(`Failed to start extraction: ${errorMsg}`);
      }
      
      // Add failed item to queue
      addToQueue({
        id: crypto.randomUUID(),
        archivePath: selectedArchive,
        outputDir: '',
        status: 'failed' as const,
        error: errorMsg,
      });
    }
  }, [selectedArchive, settings]);

  // Helper function to format bytes
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
  };

  // Global keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Check for Cmd (Mac) or Ctrl (Windows/Linux)
      const isMod = e.metaKey || e.ctrlKey;

      // Cmd+E or Ctrl+E: Extract
      if (isMod && e.key === 'e') {
        e.preventDefault();
        if (selectedArchive) {
          handleExtract();
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [selectedArchive, handleExtract]);

  // Mouse down on resize handle
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  }, []);

  // Mouse move during resize
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isResizing || !containerRef.current) return;

      const containerRect = containerRef.current.getBoundingClientRect();
      const containerWidth = containerRect.width;
      const mouseX = e.clientX - containerRect.left;
      
      // Calculate new percentage
      let newPercentage = (mouseX / containerWidth) * 100;
      
      // Enforce minimum widths
      const minLeftPercentage = (MIN_PANE_WIDTH / containerWidth) * 100;
      const minRightPercentage = (MIN_PANE_WIDTH / containerWidth) * 100;
      const maxLeftPercentage = 100 - minRightPercentage;
      
      newPercentage = Math.max(minLeftPercentage, Math.min(maxLeftPercentage, newPercentage));
      
      setLeftPaneWidth(newPercentage);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
    };

    if (isResizing) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = 'col-resize';
      document.body.style.userSelect = 'none';
    }

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    };
  }, [isResizing]);

  return (
    <div 
      ref={containerRef}
      className="flex h-full overflow-hidden"
    >
      {/* Left pane - File Explorer */}
      <div 
        className="h-full overflow-hidden"
        style={{ width: `${leftPaneWidth}%` }}
      >
        <FileExplorer
          currentPath={currentDirectory}
          onPathChange={handlePathChange}
          selectedArchive={selectedArchive}
          onArchiveSelect={handleArchiveSelect}
          onExtract={handleExtract}
        />
      </div>

      {/* Resize handle */}
      <div
        className="w-1 bg-divider hover:bg-primary cursor-col-resize flex-shrink-0 transition-colors"
        onMouseDown={handleMouseDown}
        role="separator"
        aria-label={`Resize panes (${Math.round(leftPaneWidth)}% width)`}
        aria-orientation="vertical"
        tabIndex={0}
      />

      {/* Right pane - Archive Preview */}
      <div 
        className="h-full overflow-hidden flex-1"
      >
        <ArchivePreview 
          archivePath={selectedArchive}
          onExtract={handleExtract}
        />
      </div>
    </div>
  );
}
