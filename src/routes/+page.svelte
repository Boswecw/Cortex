<script lang="ts">
  import { onMount } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import ContentArea from '$lib/components/ContentArea.svelte';
  import PreviewPanel from '$lib/components/PreviewPanel.svelte';
  import Onboarding from '$lib/components/Onboarding.svelte';
  import ToastContainer from '$lib/components/ToastContainer.svelte';
  import { toastStore } from '$lib/stores/toastStore';
  import type {
    SearchStats,
    IndexStatus,
    IndexProgressEvent,
    IndexCompleteEvent
  } from '$lib/types/api';
  import { formatDuration } from '$lib/types/api';

  // State
  let stats = $state<SearchStats | null>(null);
  let indexStatus = $state<IndexStatus | null>(null);
  let showOnboarding = $state(false);

  // Preview state
  let selectedFileId = $state<number | null>(null);
  let selectedFilename = $state<string | null>(null);
  let selectedFilepath = $state<string | null>(null);

  // Event listeners
  let progressUnlisten: UnlistenFn | null = null;
  let completeUnlisten: UnlistenFn | null = null;
  let errorUnlisten: UnlistenFn | null = null;

  // Functions
  async function refreshStats() {
    try {
      stats = await invoke('get_search_stats');
    } catch (error) {
      console.error('Failed to get stats:', error);
    }
  }

  async function refreshIndexStatus() {
    try {
      indexStatus = await invoke('get_index_status');
    } catch (error) {
      console.error('Failed to get index status:', error);
    }
  }

  function handleFileSelect(fileId: number, filename: string, filepath: string) {
    selectedFileId = fileId;
    selectedFilename = filename;
    selectedFilepath = filepath;
  }

  // Global keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    // Cmd/Ctrl + K: Focus search
    if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
      event.preventDefault();
      const searchInput = document.querySelector('input[type="text"]') as HTMLInputElement;
      searchInput?.focus();
    }

    // Escape: Clear selection
    if (event.key === 'Escape') {
      selectedFileId = null;
      selectedFilename = null;
      selectedFilepath = null;
    }
  }

  function handleOnboardingComplete() {
    showOnboarding = false;
    // Refresh stats after onboarding
    refreshStats();
    refreshIndexStatus();
  }

  // Lifecycle
  onMount(() => {
    // Check if onboarding is needed
    const onboardingComplete = localStorage.getItem('cortex-onboarding-complete');
    if (!onboardingComplete) {
      showOnboarding = true;
    }

    // Setup event listeners for indexing progress
    const setupListeners = async () => {
      progressUnlisten = await listen<IndexProgressEvent>('indexing:progress', (event) => {
        if (indexStatus) {
          indexStatus.progress = {
            total_files: event.payload.total_files,
            files_indexed: event.payload.indexed_files,
            current_file: event.payload.current_file,
            percentage: event.payload.progress_percentage,
          };
        }
      });

      completeUnlisten = await listen<IndexCompleteEvent>('indexing:complete', async (event) => {
        await refreshIndexStatus();
        await refreshStats();

        const duration = formatDuration(event.payload.duration_seconds);
        toastStore.success(
          `Indexing complete! ${event.payload.indexed_files.toLocaleString()} files indexed in ${duration}`,
          7000
        );
      });

      errorUnlisten = await listen<{ error: string }>('indexing:error', (event) => {
        console.error('Indexing error:', event.payload.error);
        toastStore.error(`Indexing error: ${event.payload.error}`, 10000);
      });

      // Initial data load
      await refreshStats();
      await refreshIndexStatus();
    };

    setupListeners();

    // Poll for status updates every 3 seconds
    const interval = setInterval(async () => {
      await refreshStats();
      if (indexStatus?.is_indexing) {
        await refreshIndexStatus();
      }
    }, 3000);

    // Add keyboard listener
    window.addEventListener('keydown', handleKeydown);

    return () => {
      progressUnlisten?.();
      completeUnlisten?.();
      errorUnlisten?.();
      clearInterval(interval);
      window.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

<div class="h-screen flex overflow-hidden">
  <!-- Sidebar (250px) -->
  <Sidebar bind:stats={stats} bind:indexStatus={indexStatus} />

  <!-- Content Area (flex-1) -->
  <ContentArea onFileSelect={handleFileSelect} bind:stats={stats} />

  <!-- Preview Panel (400px) -->
  <PreviewPanel
    bind:fileId={selectedFileId}
    bind:filename={selectedFilename}
    bind:filepath={selectedFilepath}
  />

  <!-- Onboarding Modal -->
  {#if showOnboarding}
    <Onboarding onComplete={handleOnboardingComplete} />
  {/if}

  <!-- Toast Notifications -->
  <ToastContainer />
</div>
