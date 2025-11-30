import { writable } from 'svelte/store';

export type ToastType = 'success' | 'error' | 'info' | 'warning';

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration?: number; // milliseconds, undefined = no auto-dismiss
}

interface ToastStore {
  toasts: Toast[];
}

function createToastStore() {
  const { subscribe, update } = writable<ToastStore>({ toasts: [] });

  return {
    subscribe,

    // Add a new toast
    add: (type: ToastType, message: string, duration: number = 5000) => {
      const id = Math.random().toString(36).substring(2, 9);
      const toast: Toast = { id, type, message, duration };

      update(state => ({
        toasts: [...state.toasts, toast]
      }));

      // Auto-dismiss if duration is set
      if (duration > 0) {
        setTimeout(() => {
          toastStore.dismiss(id);
        }, duration);
      }

      return id;
    },

    // Dismiss a specific toast
    dismiss: (id: string) => {
      update(state => ({
        toasts: state.toasts.filter(t => t.id !== id)
      }));
    },

    // Clear all toasts
    clear: () => {
      update(() => ({ toasts: [] }));
    },

    // Convenience methods
    success: (message: string, duration?: number) => {
      return toastStore.add('success', message, duration);
    },

    error: (message: string, duration?: number) => {
      return toastStore.add('error', message, duration);
    },

    info: (message: string, duration?: number) => {
      return toastStore.add('info', message, duration);
    },

    warning: (message: string, duration?: number) => {
      return toastStore.add('warning', message, duration);
    }
  };
}

export const toastStore = createToastStore();
