<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { page } from '$app/stores';
  import type { SearchStats, IndexStatus, EmbeddingStatus } from '$lib/types/api';
  import { formatFileSize } from '$lib/types/api';
  import { onMount } from 'svelte';

  // Props
  let {
    stats = $bindable(),
    indexStatus = $bindable()
  }: {
    stats: SearchStats | null;
    indexStatus: IndexStatus | null;
  } = $props();

  // Computed
  let isIndexing = $derived(indexStatus?.is_indexing ?? false);
  let indexProgress = $derived(indexStatus?.progress?.percentage ?? 0);

  // AI embedding status
  let embeddingStatus = $state<EmbeddingStatus | null>(null);

  // Fetch embedding status
  async function fetchEmbeddingStatus() {
    try {
      embeddingStatus = await invoke('get_embedding_status');
    } catch (error) {
      console.error('Failed to fetch embedding status:', error);
    }
  }

  onMount(() => {
    fetchEmbeddingStatus();
    // Refresh every 10 seconds
    const interval = setInterval(fetchEmbeddingStatus, 10000);
    return () => clearInterval(interval);
  });

  // Functions
  async function stopIndexing() {
    try {
      await invoke('stop_indexing');
    } catch (error) {
      console.error('Failed to stop indexing:', error);
    }
  }

  // Navigation items
  const navItems = [
    { id: 'all', label: 'All Files', icon: '📁', href: '/' },
    { id: 'recent', label: 'Recent', icon: '🕐', href: '/recent' },
    { id: 'starred', label: 'Starred', icon: '⭐', href: '/starred' },
    { id: 'settings', label: 'Settings', icon: '⚙️', href: '/settings' }
  ] as const;

  // Derived current path
  let currentPath = $derived($page.url.pathname);

  function isActive(href: string): boolean {
    if (href === '/') return currentPath === '/';
    return currentPath.startsWith(href);
  }
</script>

<aside class="w-[250px] bg-slate-byte border-r border-neural-gold/20 flex flex-col h-full">
  <!-- Header -->
  <div class="p-6 border-b border-neural-gold/20">
    <h1 class="text-2xl font-bold text-neural-gold flex items-center gap-2">
      <span class="text-3xl">🧠</span>
      Cortex
    </h1>
    <p class="text-xs text-silver-neural/60 mt-1">Local File Intelligence</p>
  </div>

  <!-- Navigation -->
  <nav class="p-4 space-y-2">
    {#each navItems as item}
      <a
        href={item.href}
        class={`w-full text-left px-4 py-3 rounded-lg transition-all flex items-center gap-3 ${
          isActive(item.href)
            ? 'bg-neural-gold/20 text-neural-gold border border-neural-gold/30'
            : 'text-silver-neural/80 hover:bg-cortex-deep hover:text-silver-neural border border-transparent'
        }`}
      >
        <span class="text-xl">{item.icon}</span>
        <span class="font-medium text-sm">{item.label}</span>
      </a>
    {/each}
  </nav>

  <!-- AI Embedding Status -->
  {#if embeddingStatus}
    <div class="px-4 pb-4 border-b border-neural-gold/20">
      <div class="bg-gradient-to-br from-purple-900/20 to-neural-gold/10 rounded-lg border border-purple-400/20 p-3">
        <div class="flex items-center gap-2 mb-2">
          <span class="text-lg">🧠</span>
          <span class="text-xs font-semibold text-purple-300">AI Embeddings</span>
        </div>

        <div class="space-y-1.5">
          <div class="flex justify-between items-center">
            <span class="text-xs text-silver-neural/60">Embedded</span>
            <span class="text-xs font-semibold text-purple-300">
              {embeddingStatus.files_with_embeddings.toLocaleString()} / {embeddingStatus.total_files.toLocaleString()}
            </span>
          </div>

          {#if embeddingStatus.total_files > 0}
            <div class="bg-cortex-black rounded-full h-1.5 overflow-hidden">
              <div
                class="bg-gradient-to-r from-purple-500 to-neural-gold h-1.5 rounded-full transition-all duration-300"
                style:width={`${(embeddingStatus.files_with_embeddings / embeddingStatus.total_files) * 100}%`}
              ></div>
            </div>
            <div class="text-xs text-silver-neural/50 text-right">
              {((embeddingStatus.files_with_embeddings / embeddingStatus.total_files) * 100).toFixed(0)}%
            </div>
          {/if}

          <div class="flex items-center gap-1.5 text-xs">
            {#if embeddingStatus.model_downloaded}
              <span class="text-green-400">✓</span>
              <span class="text-silver-neural/60">Model Ready</span>
            {:else}
              <span class="text-yellow-400">⚠</span>
              <span class="text-silver-neural/60">Model Not Downloaded</span>
            {/if}
          </div>
        </div>
      </div>
    </div>
  {/if}

  <div class="flex-1"></div>

  <!-- Indexing Status (Active) -->
  {#if isIndexing && indexStatus?.progress}
    <div class="border-t border-neural-gold/20 bg-gradient-to-b from-neural-gold/5 to-cortex-deep">
      <div class="p-4">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <span class="text-neural-gold animate-pulse">⚡</span>
            <span class="text-sm font-semibold text-neural-gold">Indexing...</span>
          </div>
          <button
            onclick={stopIndexing}
            class="px-2 py-1 text-xs text-red-400 hover:text-red-300 hover:bg-red-900/20 rounded transition-colors"
            title="Stop indexing"
          >
            Stop
          </button>
        </div>

        <!-- Progress Bar -->
        <div class="mb-2">
          <div class="bg-cortex-black rounded-full h-2 overflow-hidden">
            <div
              class="bg-gradient-to-r from-neural-gold to-ember-gold h-2 rounded-full transition-all duration-300"
              style:width={`${indexProgress}%`}
            ></div>
          </div>
        </div>

        <!-- Stats -->
        <div class="flex justify-between text-xs mb-1">
          <span class="text-silver-neural/70">
            {indexStatus.progress.files_indexed.toLocaleString()} / {indexStatus.progress.total_files.toLocaleString()}
          </span>
          <span class="text-neural-gold font-semibold">{indexProgress.toFixed(1)}%</span>
        </div>

        <!-- Current File -->
        {#if indexStatus.progress.current_file}
          <div class="text-xs text-silver-neural/50 truncate" title={indexStatus.progress.current_file}>
            {indexStatus.progress.current_file.split('/').pop() || indexStatus.progress.current_file}
          </div>
        {/if}

        <!-- Errors -->
        {#if indexStatus.errors && indexStatus.errors.length > 0}
          <div class="mt-2 pt-2 border-t border-red-700/30">
            <button
              class="text-xs text-red-400 hover:text-red-300 transition-colors flex items-center gap-1"
              title={`${indexStatus.errors.length} error(s)`}
            >
              <span>⚠️</span>
              <span>{indexStatus.errors.length} error{indexStatus.errors.length > 1 ? 's' : ''}</span>
            </button>
          </div>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Statistics Footer -->
  {#if stats}
    <div class="p-4 border-t border-neural-gold/20 bg-cortex-deep">
      <div class="space-y-2">
        <div class="flex justify-between items-center">
          <span class="text-xs text-silver-neural/60">Total Files</span>
          <span class="text-sm font-semibold text-silver-neural">
            {stats.total_files.toLocaleString()}
          </span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-xs text-silver-neural/60">Indexed</span>
          <span class="text-sm font-semibold text-neural-gold">
            {stats.indexed_files.toLocaleString()}
          </span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-xs text-silver-neural/60">Total Size</span>
          <span class="text-sm font-semibold text-silver-neural">
            {formatFileSize(stats.total_size_bytes)}
          </span>
        </div>

        <!-- Progress Bar (when not actively indexing) -->
        {#if !isIndexing}
          <div class="pt-2">
            <div class="bg-cortex-black rounded-full h-1.5">
              <div
                class="bg-neural-gold h-1.5 rounded-full transition-all duration-300"
                style:width={`${stats.total_files > 0 ? (stats.indexed_files / stats.total_files) * 100 : 0}%`}
              ></div>
            </div>
            <p class="text-xs text-silver-neural/50 mt-1 text-right">
              {stats.total_files > 0 ? ((stats.indexed_files / stats.total_files) * 100).toFixed(0) : 0}% indexed
            </p>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</aside>

<style>
  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .animate-pulse {
    animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }
</style>
