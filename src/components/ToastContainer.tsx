import { useStore } from '@nanostores/react';
import { Card, CardBody } from '@heroui/card';
import { Button } from '@heroui/button';
import { CheckCircle, XCircle, AlertTriangle, Info, X } from 'lucide-react';
import { toastsAtom, removeToast, type Toast } from '../lib/toast';

export default function ToastContainer() {
  const toasts = useStore(toastsAtom);

  const getIcon = (type: Toast['type']) => {
    switch (type) {
      case 'success':
        return <CheckCircle className="w-5 h-5 text-success" />;
      case 'error':
        return <XCircle className="w-5 h-5 text-danger" />;
      case 'warning':
        return <AlertTriangle className="w-5 h-5 text-warning" />;
      case 'info':
        return <Info className="w-5 h-5 text-primary" />;
    }
  };

  const getColorClass = (type: Toast['type']) => {
    switch (type) {
      case 'success':
        return 'border-l-4 border-success';
      case 'error':
        return 'border-l-4 border-danger';
      case 'warning':
        return 'border-l-4 border-warning';
      case 'info':
        return 'border-l-4 border-primary';
    }
  };

  if (toasts.length === 0) return null;

  return (
    <div 
      className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-md"
      role="region"
      aria-label="Notifications"
    >
      {toasts.map((toast) => (
        <Card 
          key={toast.id}
          className={`${getColorClass(toast.type)} shadow-lg animate-in slide-in-from-right`}
        >
          <CardBody className="flex flex-row items-start gap-3 p-3">
            <div className="flex-shrink-0 mt-0.5">
              {getIcon(toast.type)}
            </div>
            <p className="text-sm flex-1 text-default-700">
              {toast.message}
            </p>
            <Button
              isIconOnly
              size="sm"
              variant="light"
              onPress={() => removeToast(toast.id)}
              aria-label="Dismiss notification"
              className="flex-shrink-0"
            >
              <X className="w-4 h-4" />
            </Button>
          </CardBody>
        </Card>
      ))}
    </div>
  );
}
