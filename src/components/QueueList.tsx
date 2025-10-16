import { useStore } from '@nanostores/react';
import { Card, CardBody, CardHeader } from '@heroui/card';
import { Progress } from '@heroui/progress';
import { Chip } from '@heroui/chip';
import { Button } from '@heroui/button';
import { X, Trash2, CheckCircle, XCircle, Clock, Loader2 } from 'lucide-react';
import { queueMap, removeFromQueue } from '../lib/store';
import { cancelJob } from '../lib/api';
import type { QueueItem, JobStatus } from '../lib/types';

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

function getStatusColor(status: JobStatus): 'default' | 'primary' | 'success' | 'danger' | 'warning' {
  switch (status) {
    case 'pending':
      return 'default';
    case 'extracting':
      return 'primary';
    case 'completed':
      return 'success';
    case 'failed':
      return 'danger';
    case 'cancelled':
      return 'warning';
  }
}

function getStatusIcon(status: JobStatus) {
  switch (status) {
    case 'pending':
      return <Clock className="w-4 h-4" />;
    case 'extracting':
      return <Loader2 className="w-4 h-4 animate-spin" />;
    case 'completed':
      return <CheckCircle className="w-4 h-4" />;
    case 'failed':
      return <XCircle className="w-4 h-4" />;
    case 'cancelled':
      return <XCircle className="w-4 h-4" />;
  }
}

interface QueueItemCardProps {
  item: QueueItem;
}

function QueueItemCard({ item }: QueueItemCardProps) {
  const handleCancel = async () => {
    try {
      await cancelJob(item.id);
      removeFromQueue(item.id);
    } catch (error) {
      console.error('Failed to cancel job:', error);
    }
  };

  const handleRemove = () => {
    removeFromQueue(item.id);
  };

  const archiveName = item.archivePath.split('/').pop() || item.archivePath;
  const isInProgress = item.status === 'extracting';
  const isCompleted = item.status === 'completed' || item.status === 'failed' || item.status === 'cancelled';

  const progressPercentage = item.progress?.totalBytes
    ? Math.round((item.progress.bytesWritten / item.progress.totalBytes) * 100)
    : 0;

  return (
    <Card className="mb-3">
      <CardHeader className="flex justify-between items-start pb-2">
        <div className="flex-1">
          <h4 className="text-lg font-semibold">{archiveName}</h4>
          <p className="text-sm text-default-500">{item.outputDir}</p>
        </div>
        <div className="flex items-center gap-2">
          <Chip
            color={getStatusColor(item.status)}
            variant="flat"
            startContent={getStatusIcon(item.status)}
            size="sm"
          >
            {item.status}
          </Chip>
          {isInProgress && (
            <Button
              isIconOnly
              size="sm"
              color="danger"
              variant="light"
              onPress={handleCancel}
              aria-label="Cancel extraction"
            >
              <X className="w-4 h-4" />
            </Button>
          )}
          {isCompleted && (
            <Button
              isIconOnly
              size="sm"
              color="default"
              variant="light"
              onPress={handleRemove}
              aria-label="Remove from queue"
            >
              <Trash2 className="w-4 h-4" />
            </Button>
          )}
        </div>
      </CardHeader>
      <CardBody className="pt-0">
        {isInProgress && item.progress && (
          <div className="space-y-2">
            <Progress
              value={progressPercentage}
              color="primary"
              size="sm"
              showValueLabel
              className="mb-2"
            />
            <div className="text-sm text-default-600">
              <p className="truncate">
                <span className="font-medium">Current file:</span> {item.progress.currentFile}
              </p>
              <p>
                <span className="font-medium">Files extracted:</span> {item.progress.filesExtracted}
              </p>
              {item.progress.totalBytes && (
                <p>
                  <span className="font-medium">Progress:</span>{' '}
                  {formatBytes(item.progress.bytesWritten)} / {formatBytes(item.progress.totalBytes)}
                </p>
              )}
            </div>
          </div>
        )}
        {item.status === 'completed' && item.stats && (
          <div className="text-sm text-success-600">
            <p>
              <span className="font-medium">Extracted:</span> {item.stats.files_extracted} files (
              {formatBytes(item.stats.bytes_written)})
            </p>
          </div>
        )}
        {item.status === 'failed' && (
          <div className="text-sm text-danger-600">
            <p className="font-medium">Error:</p>
            <p>{item.error || 'Extraction failed with no error message'}</p>
          </div>
        )}
      </CardBody>
    </Card>
  );
}

export default function QueueList() {
  const queue = useStore(queueMap);
  const queueItems = Object.values(queue);

  if (queueItems.length === 0) {
    return (
      <div className="text-center py-12 text-default-400">
        <p>No archives in queue</p>
        <p className="text-sm mt-2">Drag and drop archives or use the file picker to get started</p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <h3 className="text-xl font-semibold mb-4">Extraction Queue</h3>
      {queueItems.map((item) => (
        <QueueItemCard key={item.id} item={item} />
      ))}
    </div>
  );
}
