import { useEffect, useState } from 'react';
import { useStore } from '@nanostores/react';
import { Navbar, NavbarBrand, NavbarContent, NavbarItem } from '@heroui/navbar';
import { Button } from '@heroui/button';
import { Spacer } from '@heroui/spacer';
import { Sun, Moon, Monitor, Archive } from 'lucide-react';
import { themeAtom, setTheme, updateQueueItem } from './lib/store';
import { onProgress, onCompletion, onPasswordRequired, onFilesOpened } from './lib/api';
import type { ProgressEvent, CompletionEvent, PasswordRequiredEvent } from './lib/api';
import DropZone from './components/DropZone';
import QueueList from './components/QueueList';
import Settings from './components/Settings';
import PasswordPrompt from './components/PasswordPrompt';

function App() {
  const theme = useStore(themeAtom);
  const [passwordPrompt, setPasswordPrompt] = useState<{
    isOpen: boolean;
    jobId: string;
    archivePath: string;
  }>({
    isOpen: false,
    jobId: '',
    archivePath: '',
  });

  // Set up event listeners
  useEffect(() => {
    let unlistenProgress: (() => void) | undefined;
    let unlistenCompletion: (() => void) | undefined;
    let unlistenPassword: (() => void) | undefined;
    let unlistenFilesOpened: (() => void) | undefined;

    const setupListeners = async () => {
      // Progress events
      unlistenProgress = await onProgress((event: ProgressEvent) => {
        console.log('Progress event received:', event);
        updateQueueItem(event.jobId, {
          status: 'extracting',
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
        console.log('Completion event received:', event);
        
        const status = event.status.toLowerCase() === 'success' 
          ? 'completed' 
          : event.status.toLowerCase() === 'cancelled' 
          ? 'cancelled' 
          : 'failed';

        console.log('Mapped status:', status);

        updateQueueItem(event.jobId, {
          status,
          stats: event.stats,
          error: event.error || undefined,
          progress: undefined,
        });
      });

      // Password required events
      unlistenPassword = await onPasswordRequired((event: PasswordRequiredEvent) => {
        setPasswordPrompt({
          isOpen: true,
          jobId: event.jobId,
          archivePath: event.archivePath,
        });
      });

      // Files opened from Finder (double-click or drag to app icon)
      unlistenFilesOpened = await onFilesOpened(async (paths: string[]) => {
        // Import dynamically to avoid circular dependencies
        const { open } = await import('@tauri-apps/plugin-dialog');
        const { probeArchive, extractArchives } = await import('./lib/api');
        const { settingsAtom, addToQueue } = await import('./lib/store');
        
        // Get output directory from user
        const outputDir = await open({
          directory: true,
          multiple: false,
          title: 'Select Output Directory',
        });

        if (!outputDir || typeof outputDir !== 'string') {
          return;
        }

        // Get current settings
        const settings = settingsAtom.get();

        // Process each archive
        for (const path of paths) {
          try {
            // Probe the archive to get metadata
            await probeArchive(path);
            
            // Create queue item
            const queueItem = {
              id: crypto.randomUUID(),
              archivePath: path,
              outputDir,
              status: 'pending' as const,
            };

            // Add to queue
            addToQueue(queueItem);

            // Start extraction
            const jobId = await extractArchives([path], outputDir, settings);
            
            // Update queue item with actual job ID
            addToQueue({ ...queueItem, id: jobId });
          } catch (error) {
            console.error(`Failed to process ${path}:`, error);
            
            // Add failed item to queue
            addToQueue({
              id: crypto.randomUUID(),
              archivePath: path,
              outputDir,
              status: 'failed' as const,
              error: error instanceof Error ? error.message : 'Unknown error',
            });
          }
        }
      });
    };

    setupListeners();

    return () => {
      unlistenProgress?.();
      unlistenCompletion?.();
      unlistenPassword?.();
      unlistenFilesOpened?.();
    };
  }, []);

  // Apply theme to document
  useEffect(() => {
    const root = document.documentElement;
    
    if (theme === 'system') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      root.classList.toggle('dark', prefersDark);
    } else {
      root.classList.toggle('dark', theme === 'dark');
    }
  }, [theme]);

  const handleThemeToggle = () => {
    const themes: Array<'light' | 'dark' | 'system'> = ['light', 'dark', 'system'];
    const currentIndex = themes.indexOf(theme);
    const nextTheme = themes[(currentIndex + 1) % themes.length];
    setTheme(nextTheme);
  };

  const getThemeIcon = () => {
    switch (theme) {
      case 'light':
        return <Sun className="w-5 h-5" />;
      case 'dark':
        return <Moon className="w-5 h-5" />;
      case 'system':
        return <Monitor className="w-5 h-5" />;
    }
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Navbar */}
      <Navbar isBordered maxWidth="full">
        <NavbarBrand>
          <Archive className="w-6 h-6 mr-2 text-primary" />
          <p className="font-bold text-xl">Unarchive</p>
        </NavbarBrand>
        <NavbarContent justify="end">
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

      {/* Main Content */}
      <main className="container mx-auto px-4 py-8 max-w-7xl">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Left Column - Drop Zone and Queue */}
          <div className="lg:col-span-2 space-y-6">
            <DropZone />
            <QueueList />
          </div>

          {/* Right Column - Settings */}
          <div className="lg:col-span-1">
            <Settings />
          </div>
        </div>
      </main>

      {/* Password Prompt Modal */}
      <PasswordPrompt
        isOpen={passwordPrompt.isOpen}
        onClose={() => setPasswordPrompt({ ...passwordPrompt, isOpen: false })}
        jobId={passwordPrompt.jobId}
        archivePath={passwordPrompt.archivePath}
      />

      <Spacer y={4} />
    </div>
  );
}

export default App;
