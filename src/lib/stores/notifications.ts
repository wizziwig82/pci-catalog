import { writable } from 'svelte/store';

interface Notification {
  id: number;
  type: 'error' | 'success' | 'info';
  message: string;
  timeout?: number; // Optional timeout in ms
}

const { subscribe, update } = writable<Notification[]>([]);

let nextId = 0;

function addNotification(type: Notification['type'], message: string, timeout = 5000) {
  const id = nextId++;
  update((notifications) => [...notifications, { id, type, message, timeout }]);

  if (timeout) {
    setTimeout(() => {
      removeNotification(id);
    }, timeout);
  }
}

function removeNotification(id: number) {
  update((notifications) => notifications.filter((n) => n.id !== id));
}

export const notifications = {
  subscribe,
  showErrorToast: (message: string, timeout?: number) => addNotification('error', message, timeout),
  showSuccessToast: (message: string, timeout?: number) => addNotification('success', message, timeout),
  showInfoToast: (message: string, timeout?: number) => addNotification('info', message, timeout),
  remove: removeNotification,
};

// Export the specific functions for easier import elsewhere
export const showErrorToast = notifications.showErrorToast;
export const showSuccessToast = notifications.showSuccessToast;
export const showInfoToast = notifications.showInfoToast;