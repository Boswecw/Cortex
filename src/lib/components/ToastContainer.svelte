<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import { toastStore, type Toast } from '$lib/stores/toastStore';

  let toasts = $state<Toast[]>([]);

  // Subscribe to toast store
  toastStore.subscribe(state => {
    toasts = state.toasts;
  });

  function getToastIcon(type: Toast['type']): string {
    switch (type) {
      case 'success': return '✅';
      case 'error': return '❌';
      case 'warning': return '⚠️';
      case 'info': return 'ℹ️';
      default: return 'ℹ️';
    }
  }

  function getToastStyles(type: Toast['type']): string {
    switch (type) {
      case 'success':
        return 'bg-green-900/90 border-green-600 text-green-100';
      case 'error':
        return 'bg-red-900/90 border-red-600 text-red-100';
      case 'warning':
        return 'bg-yellow-900/90 border-yellow-600 text-yellow-100';
      case 'info':
        return 'bg-blue-900/90 border-blue-600 text-blue-100';
      default:
        return 'bg-slate-byte border-neural-gold/30 text-silver-neural';
    }
  }
</script>

<!-- Toast Container - Fixed position at top-right -->
<div class="fixed top-4 right-4 z-[100] flex flex-col gap-2 pointer-events-none">
  {#each toasts as toast (toast.id)}
    <div
      class={`pointer-events-auto flex items-center gap-3 px-4 py-3 rounded-lg border-2 shadow-2xl backdrop-blur-sm min-w-[300px] max-w-md ${getToastStyles(toast.type)}`}
      in:fly={{ x: 300, duration: 300 }}
      out:fade={{ duration: 200 }}
    >
      <span class="text-2xl flex-shrink-0">{getToastIcon(toast.type)}</span>
      <p class="flex-1 text-sm font-medium">{toast.message}</p>
      <button
        onclick={() => toastStore.dismiss(toast.id)}
        class="text-xl hover:opacity-70 transition-opacity flex-shrink-0"
        title="Dismiss"
      >
        ×
      </button>
    </div>
  {/each}
</div>

<style>
  /* Ensure toasts appear above everything */
  div {
    position: relative;
  }
</style>
