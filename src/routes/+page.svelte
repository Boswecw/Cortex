<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import type {
    SearchResults,
    SearchFilters,
    IndexStatus,
    IndexProgressEvent,
    IndexCompleteEvent,
    SearchStats
  } from '$lib/types/api';
  import { formatFileSize, formatDuration, getFileTypeInfo } from '$lib/types/api';

  // State
  let query = $state('');
  let searchResults = $state<SearchResults | null>(null);
  let isSearching = $state(false);
  let searchError = $state<string | null>(null);

  let indexStatus = $state<IndexStatus | null>(null);
  let stats = $state<SearchStats | null>(null);
  let indexPaths = $state<string[]>([]);
  let newPath = $state('');

  let selectedFileId = $state<number | null>(null);
  let showFilters = $state(false);

  // Filters
  let filters = $state<SearchFilters>({
    file_type: undefined,
    min_size: undefined,
    max_size: undefined,
    date_from: undefined,
    date_to: undefined,
  });

  // Event listeners
  let progressUnlisten: UnlistenFn | null = null;
  let completeUnlisten: UnlistenFn | null = null;

  // Computed
  let hasResults = $derived(searchResults !== null && searchResults.total > 0);
  let isIndexing = $derived(indexStatus?.is_indexing ?? false);
  let indexProgress = $derived(indexStatus?.progress?.percentage ?? 0);

  // Search function
  async function handleSearch() {
    if (!query.trim()) {
      searchError = 'Please enter a search query';
      return;
    }

    isSearching = true;
    searchError = null;

    try {
      const cleanFilters: SearchFilters = {};
      if (filters.file_type) cleanFilters.file_type = filters.file_type;
      if (filters.min_size) cleanFilters.min_size = filters.min_size;
      if (filters.max_size) cleanFilters.max_size = filters.max_size;
      if (filters.date_from) cleanFilters.date_from = filters.date_from;
      if (filters.date_to) cleanFilters.date_to = filters.date_to;

      searchResults = await invoke('search_files', {
        query: query.trim(),
        filters: Object.keys(cleanFilters).length > 0 ? cleanFilters : null,
        limit: 50,
        offset: 0,
      });
    } catch (error) {
      searchError = String(error);
      searchResults = null;
    } finally {
      isSearching = false;
    }
  }

  // Indexing functions
  async function startIndexing() {
    if (indexPaths.length === 0) {
      alert('Please add at least one directory to index');
      return;
    }

    try {
      await invoke('start_indexing', { paths: indexPaths });
      await refreshIndexStatus();
    } catch (error) {
      alert(`Failed to start indexing: ${error}`);
    }
  }

  async function stopIndexing() {
    try {
      await invoke('stop_indexing');
      await refreshIndexStatus();
    } catch (error) {
      alert(`Failed to stop indexing: ${error}`);
    }
  }

  async function refreshIndexStatus() {
    try {
      indexStatus = await invoke('get_index_status');
    } catch (error) {
      console.error('Failed to get index status:', error);
    }
  }

  async function refreshStats() {
    try {
      stats = await invoke('get_search_stats');
    } catch (error) {
      console.error('Failed to get stats:', error);
    }
  }

  function addIndexPath() {
    if (newPath.trim() && !indexPaths.includes(newPath.trim())) {
      indexPaths = [...indexPaths, newPath.trim()];
      newPath = '';
    }
  }

  function removePath(path: string) {
    indexPaths = indexPaths.filter(p => p !== path);
  }

  function clearFilters() {
    filters = {
      file_type: undefined,
      min_size: undefined,
      max_size: undefined,
      date_from: undefined,
      date_to: undefined,
    };
  }

  // Lifecycle
  onMount(async () => {
    // Setup event listeners
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
      alert(`Indexing complete! ${event.payload.indexed_files} files indexed in ${duration}`);
    });

    // Initial data load
    await refreshIndexStatus();
    await refreshStats();

    // Poll for status updates every 2 seconds
    const interval = setInterval(async () => {
      await refreshIndexStatus();
      await refreshStats();
    }, 2000);

    return () => {
      progressUnlisten?.();
      completeUnlisten?.();
      clearInterval(interval);
    };
  });
</script>

<div class="min-h-screen bg-cortex-black text-silver-neural">
  <!-- Header -->
  <header class="bg-slate-byte border-b border-neural-gold/20 p-4">
    <div class="container mx-auto">
      <h1 class="text-3xl font-bold text-neural-gold">Cortex</h1>
      <p class="text-silver-neural/70 text-sm">Fast, offline-first file search</p>
    </div>
  </header>

  <div class="container mx-auto p-6">
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">

      <!-- Main Search Area (2/3 width) -->
      <div class="lg:col-span-2 space-y-6">

        <!-- Search Bar -->
        <div class="bg-slate-byte rounded-lg p-6 shadow-lg border border-neural-gold/20">
          <h2 class="text-xl font-semibold mb-4 text-neural-gold">Search Files</h2>

          <form onsubmit={(e) => { e.preventDefault(); handleSearch(); }} class="space-y-4">
            <div>
              <input
                type="text"
                bind:value={query}
                placeholder="Search for files..."
                class="w-full px-4 py-3 bg-cortex-black text-silver-neural rounded-lg border border-neural-gold/20 focus:border-neural-gold focus:outline-none placeholder-silver-neural/50"
                disabled={isSearching}
              />
            </div>

            <!-- Filter Toggle -->
            <div class="flex items-center justify-between">
              <button
                type="button"
                onclick={() => showFilters = !showFilters}
                class="text-sm text-neural-gold hover:text-ember-gold transition-colors"
              >
                {showFilters ? 'Hide Filters' : 'Show Filters'}
              </button>

              {#if Object.values(filters).some(v => v !== undefined)}
                <button
                  type="button"
                  onclick={clearFilters}
                  class="text-sm text-silver-neural/70 hover:text-silver-neural transition-colors"
                >
                  Clear Filters
                </button>
              {/if}
            </div>

            <!-- Filters -->
            {#if showFilters}
              <div class="grid grid-cols-2 gap-4 p-4 bg-cortex-black rounded-lg border border-neural-gold/20">
                <div>
                  <label class="block text-sm text-silver-neural/70 mb-2">File Type</label>
                  <input
                    type="text"
                    bind:value={filters.file_type}
                    placeholder="e.g., txt, md, pdf"
                    class="w-full px-3 py-2 bg-slate-byte text-silver-neural rounded border border-neural-gold/20 focus:border-neural-gold focus:outline-none text-sm placeholder-silver-neural/50"
                  />
                </div>

                <div>
                  <label class="block text-sm text-silver-neural/70 mb-2">Min Size (bytes)</label>
                  <input
                    type="number"
                    bind:value={filters.min_size}
                    placeholder="0"
                    class="w-full px-3 py-2 bg-slate-byte text-silver-neural rounded border border-neural-gold/20 focus:border-neural-gold focus:outline-none text-sm placeholder-silver-neural/50"
                  />
                </div>

                <div>
                  <label class="block text-sm text-silver-neural/70 mb-2">Max Size (bytes)</label>
                  <input
                    type="number"
                    bind:value={filters.max_size}
                    placeholder="Unlimited"
                    class="w-full px-3 py-2 bg-slate-byte text-silver-neural rounded border border-neural-gold/20 focus:border-neural-gold focus:outline-none text-sm placeholder-silver-neural/50"
                  />
                </div>

                <div>
                  <label class="block text-sm text-silver-neural/70 mb-2">Modified After</label>
                  <input
                    type="date"
                    bind:value={filters.date_from}
                    class="w-full px-3 py-2 bg-slate-byte text-silver-neural rounded border border-neural-gold/20 focus:border-neural-gold focus:outline-none text-sm"
                  />
                </div>
              </div>
            {/if}

            <button
              type="submit"
              disabled={isSearching || !query.trim()}
              class="w-full px-6 py-3 bg-neural-gold text-cortex-black font-semibold rounded-lg hover:bg-ember-gold disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {isSearching ? 'Searching...' : 'Search'}
            </button>
          </form>

          {#if searchError}
            <div class="mt-4 p-3 bg-red-900/30 border border-red-700 rounded-lg text-red-300 text-sm">
              {searchError}
            </div>
          {/if}
        </div>

        <!-- Search Results -->
        {#if searchResults}
          <div class="bg-slate-byte rounded-lg p-6 shadow-lg border border-neural-gold/20">
            <div class="flex items-center justify-between mb-4">
              <h2 class="text-xl font-semibold text-neural-gold">Results</h2>
              <span class="text-sm text-silver-neural/70">
                {searchResults.total} results in {searchResults.query_time_ms}ms
              </span>
            </div>

            {#if hasResults}
              <div class="space-y-3">
                {#each searchResults.results as result}
                  <div class="bg-cortex-black rounded-lg p-4 border border-neural-gold/20 hover:border-neural-gold transition-colors cursor-pointer">
                    <div class="flex items-start justify-between">
                      <div class="flex-1">
                        <h3 class="font-semibold text-silver-neural mb-1">{result.filename}</h3>
                        <p class="text-xs text-silver-neural/50 mb-2 truncate" title={result.path}>{result.path}</p>
                        <div class="text-sm text-silver-neural/90 snippet">
                          {@html result.snippet}
                        </div>
                      </div>
                      <div class="ml-4">
                        <span class={`text-xs px-2 py-1 rounded bg-neural-gold/20 ${getFileTypeInfo(result.filename.split('.').pop() || '').color}`}>
                          {getFileTypeInfo(result.filename.split('.').pop() || '').label}
                        </span>
                      </div>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="text-silver-neural/50 text-center py-8">No results found</p>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Sidebar (1/3 width) -->
      <div class="space-y-6">

        <!-- Statistics -->
        {#if stats}
          <div class="bg-slate-byte rounded-lg p-6 shadow-lg border border-neural-gold/20">
            <h2 class="text-xl font-semibold mb-4 text-neural-gold">Statistics</h2>
            <div class="space-y-3">
              <div>
                <p class="text-sm text-silver-neural/70">Total Files</p>
                <p class="text-2xl font-bold text-silver-neural">{stats.total_files.toLocaleString()}</p>
              </div>
              <div>
                <p class="text-sm text-silver-neural/70">Indexed Files</p>
                <p class="text-2xl font-bold text-silver-neural">{stats.indexed_files.toLocaleString()}</p>
              </div>
              <div>
                <p class="text-sm text-silver-neural/70">Total Size</p>
                <p class="text-2xl font-bold text-silver-neural">{formatFileSize(stats.total_size_bytes)}</p>
              </div>
              <div class="pt-3 border-t border-neural-gold/20">
                <p class="text-sm text-silver-neural/70">Index Progress</p>
                <div class="mt-2 bg-cortex-black rounded-full h-2">
                  <div
                    class="bg-neural-gold h-2 rounded-full transition-all"
                    style:width={`${stats.total_files > 0 ? (stats.indexed_files / stats.total_files) * 100 : 0}%`}
                  ></div>
                </div>
                <p class="text-xs text-silver-neural/70 mt-1 text-right">
                  {stats.total_files > 0 ? ((stats.indexed_files / stats.total_files) * 100).toFixed(1) : 0}%
                </p>
              </div>
            </div>
          </div>
        {/if}

        <!-- Indexing Control -->
        <div class="bg-slate-byte rounded-lg p-6 shadow-lg border border-neural-gold/20">
          <h2 class="text-xl font-semibold mb-4 text-neural-gold">Indexing</h2>

          {#if isIndexing && indexStatus?.progress}
            <div class="space-y-3 mb-4">
              <div>
                <div class="flex justify-between text-sm mb-2">
                  <span class="text-silver-neural/70">Progress</span>
                  <span class="text-neural-gold font-semibold">{indexProgress.toFixed(1)}%</span>
                </div>
                <div class="bg-cortex-black rounded-full h-3">
                  <div
                    class="bg-neural-gold h-3 rounded-full transition-all"
                    style:width={`${indexProgress}%`}
                  ></div>
                </div>
              </div>
              <div class="text-sm">
                <p class="text-silver-neural/70">Files:</p>
                <p class="text-silver-neural">{indexStatus.progress.files_indexed} / {indexStatus.progress.total_files}</p>
              </div>
              <div class="text-sm">
                <p class="text-silver-neural/70">Current:</p>
                <p class="text-silver-neural truncate text-xs" title={indexStatus.progress.current_file}>
                  {indexStatus.progress.current_file}
                </p>
              </div>
              <button
                onclick={stopIndexing}
                class="w-full px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
              >
                Stop Indexing
              </button>
            </div>
          {:else}
            <div class="space-y-3">
              <div>
                <label class="block text-sm text-silver-neural/70 mb-2">Directory Path</label>
                <div class="flex gap-2">
                  <input
                    type="text"
                    bind:value={newPath}
                    placeholder="/home/user/Documents"
                    class="flex-1 px-3 py-2 bg-cortex-black text-silver-neural rounded border border-neural-gold/20 focus:border-neural-gold focus:outline-none text-sm placeholder-silver-neural/50"
                    onkeypress={(e) => e.key === 'Enter' && (e.preventDefault(), addIndexPath())}
                  />
                  <button
                    type="button"
                    onclick={addIndexPath}
                    class="px-4 py-2 bg-neural-gold text-cortex-black rounded hover:bg-ember-gold transition-colors text-sm font-semibold"
                  >
                    Add
                  </button>
                </div>
              </div>

              {#if indexPaths.length > 0}
                <div class="space-y-2">
                  <p class="text-sm text-silver-neural/70">Paths to index:</p>
                  {#each indexPaths as path}
                    <div class="flex items-center justify-between bg-cortex-black rounded p-2 border border-neural-gold/10">
                      <span class="text-sm truncate flex-1 text-silver-neural" title={path}>{path}</span>
                      <button
                        onclick={() => removePath(path)}
                        class="ml-2 text-red-400 hover:text-red-300 text-xs"
                      >
                        Remove
                      </button>
                    </div>
                  {/each}
                </div>
              {/if}

              <button
                onclick={startIndexing}
                disabled={indexPaths.length === 0}
                class="w-full px-4 py-2 bg-neural-gold text-cortex-black font-semibold rounded-lg hover:bg-ember-gold disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                Start Indexing
              </button>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  :global(.snippet mark) {
    background-color: #D4AF37;
    color: #1A1A1A;
    padding: 0 2px;
    border-radius: 2px;
    font-weight: 600;
  }
</style>
