import { useState } from 'react';
import {
  Modal,
  ModalContent,
  ModalHeader,
  ModalBody,
  ModalFooter,
} from '@heroui/modal';
import { Input } from '@heroui/input';
import { Button } from '@heroui/button';
import { Lock, AlertCircle } from 'lucide-react';
import { providePassword } from '../lib/api';

interface PasswordPromptProps {
  isOpen: boolean;
  onClose: () => void;
  jobId: string;
  archivePath: string;
}

export default function PasswordPrompt({
  isOpen,
  onClose,
  jobId,
  archivePath,
}: PasswordPromptProps) {
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const archiveName = archivePath.split('/').pop() || archivePath;

  const handleSubmit = async () => {
    if (!password.trim()) {
      setError('Password cannot be empty');
      return;
    }

    setIsSubmitting(true);
    setError(null);

    try {
      await providePassword(jobId, password);
      // Reset state and close modal on success
      setPassword('');
      setError(null);
      onClose();
    } catch (err) {
      // If password is incorrect, the backend will emit another password_required event
      // or return an error. Show error and allow retry.
      setError('Failed to provide password. Please try again.');
      console.error('Password submission error:', err);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleCancel = () => {
    setPassword('');
    setError(null);
    onClose();
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !isSubmitting) {
      handleSubmit();
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleCancel}
      placement="center"
      backdrop="blur"
    >
      <ModalContent>
        <ModalHeader className="flex gap-2 items-center">
          <Lock className="w-5 h-5" />
          Password Required
        </ModalHeader>
        <ModalBody>
          <p className="text-sm text-default-600 mb-4">
            The archive <span className="font-semibold">{archiveName}</span> is password-protected.
            Please enter the password to continue extraction.
          </p>
          <Input
            type="password"
            label="Password"
            placeholder="Enter archive password"
            value={password}
            onValueChange={setPassword}
            onKeyPress={handleKeyPress}
            autoFocus
            isDisabled={isSubmitting}
            errorMessage={error}
            isInvalid={!!error}
          />
          {error && (
            <div className="flex items-start gap-2 mt-2 p-3 bg-danger-50 rounded-lg">
              <AlertCircle className="w-4 h-4 text-danger-600 mt-0.5 flex-shrink-0" />
              <p className="text-sm text-danger-600">{error}</p>
            </div>
          )}
        </ModalBody>
        <ModalFooter>
          <Button
            color="default"
            variant="light"
            onPress={handleCancel}
            isDisabled={isSubmitting}
          >
            Cancel
          </Button>
          <Button
            color="primary"
            onPress={handleSubmit}
            isLoading={isSubmitting}
          >
            Submit
          </Button>
        </ModalFooter>
      </ModalContent>
    </Modal>
  );
}
