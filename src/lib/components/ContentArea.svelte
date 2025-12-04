<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { fade, fly } from 'svelte/transition';
  import type { SearchResults, SearchFilters } from '$lib/types/api';
  import { getFileTypeInfo } from '$lib/types/api';
  import SemanticSearch from './SemanticSearch.svelte';

  // Props
  let {
    onFileSelect,
    stats = $bindable()
  }: {
    onFileSelect: (fileId: number, filename: string, path: string) => void;
    stats?: { total_files: number; indexed_files: number; total_size_bytes: number } | null;
  } = $props();

  // Search Mode
  type SearchMode = 'keyword' | 'semantic';
  let searchMode = $state<SearchMode>('keyword');

  // State
  let query = $state('');
  let searchResults = $state<SearchResults | null>(null);
  let isSearching = $state(false);
  let searchError = $state<string | null>(null);
  let showFilters = $state(false);
  let selectedFileId = $state<number | null>(null);
  let keyboardFocusedIndex = $state<number>(-1);
  let showShortcutsModal = $state(false);

  // Filter presets
  interface FilterPreset {
    name: string;
    filters: SearchFilters;
  }

  let filterPresets = $state<FilterPreset[]>([]);
  let presetName = $state('');
  let showPresetSave = $state(false);

  // Filters
  let filters = $state<SearchFilters>({
    file_type: undefined,
    min_size: undefined,
    max_size: undefined,
    date_from: undefined,
    date_to: undefined,
  });

  // Filter preset options
  const fileTypeOptions = [
    { value: '', label: 'All Types' },
    { value: 'txt', label: 'Text (.txt)' },
    { value: 'md', label: 'Markdown (.md)' },
    { value: 'pdf', label: 'PDF (.pdf)' },
    { value: 'docx', label: 'Word (.docx)' },
    { value: 'js', label: 'JavaScript (.js)' },
    { value: 'ts', label: 'TypeScript (.ts)' },
    { value: 'jsx', label: 'React (.jsx)' },
    { value: 'tsx', label: 'React TS (.tsx)' },
    { value: 'py', label: 'Python (.py)' },
    { value: 'rs', label: 'Rust (.rs)' },
    { value: 'java', label: 'Java (.java)' },
    { value: 'cpp', label: 'C++ (.cpp)' },
    { value: 'json', label: 'JSON (.json)' },
    { value: 'yaml', label: 'YAML (.yaml)' },
    { value: 'xml', label: 'XML (.xml)' },
  ];

  const sizePresetOptions = [
    { value: 'any', label: 'Any Size', min: undefined, max: undefined },
    { value: 'tiny', label: '< 100 KB', min: undefined, max: 100 * 1024 },
    { value: 'small', label: '< 1 MB', min: undefined, max: 1024 * 1024 },
    { value: 'medium', label: '1-10 MB', min: 1024 * 1024, max: 10 * 1024 * 1024 },
    { value: 'large', label: '10-100 MB', min: 10 * 1024 * 1024, max: 100 * 1024 * 1024 },
    { value: 'huge', label: '> 100 MB', min: 100 * 1024 * 1024, max: undefined },
  ];

  const datePresetOptions = [
    { value: 'any', label: 'Any Time', days: null },
    { value: 'today', label: 'Today', days: 0 },
    { value: 'week', label: 'Last 7 Days', days: 7 },
    { value: 'month', label: 'Last 30 Days', days: 30 },
    { value: '3months', label: 'Last 3 Months', days: 90 },
    { value: 'year', label: 'Last Year', days: 365 },
  ];

  let selectedSizePreset = $state('any');
  let selectedDatePreset = $state('any');

  // Computed
  let hasResults = $derived(searchResults !== null && searchResults.total > 0);
  let activeFiltersCount = $derived(() => {
    let count = 0;
    if (filters.file_type) count++;
    if (filters.min_size !== undefined || filters.max_size !== undefined) count++;
    if (filters.date_from) count++;
    return count;
  });

  let activeFilterChips = $derived(() => {
    const chips: Array<{ key: string; label: string; value: string }> = [];

    if (filters.file_type) {
      const typeOption = fileTypeOptions.find(opt => opt.value === filters.file_type);
      chips.push({
        key: 'file_type',
        label: 'Type',
        value: typeOption?.label || filters.file_type
      });
    }

    if (filters.min_size !== undefined || filters.max_size !== undefined) {
      const sizePreset = sizePresetOptions.find(opt =>
        opt.min === filters.min_size && opt.max === filters.max_size
      );
      chips.push({
        key: 'size',
        label: 'Size',
        value: sizePreset?.label || 'Custom'
      });
    }

    if (filters.date_from) {
      chips.push({
        key: 'date',
        label: 'Modified',
        value: formatDateChip(filters.date_from)
      });
    }

    return chips;
  });

  // Animation helper
  let resultsKey = $state(0);

  // Search function with debounce
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;

  function handleSearchInput() {
    if (searchTimeout) clearTimeout(searchTimeout);

    searchTimeout = setTimeout(() => {
      if (query.trim()) {
        handleSearch();
      } else {
        searchResults = null;
      }
    }, 150); // 150ms debounce
  }

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
      if (filters.min_size !== undefined) cleanFilters.min_size = filters.min_size;
      if (filters.max_size !== undefined) cleanFilters.max_size = filters.max_size;
      if (filters.date_from) cleanFilters.date_from = filters.date_from;
      if (filters.date_to) cleanFilters.date_to = filters.date_to;

      searchResults = await invoke('search_files', {
        query: query.trim(),
        filters: Object.keys(cleanFilters).length > 0 ? cleanFilters : null,
        limit: 50,
        offset: 0,
      });

      // Reset keyboard focus when new results arrive
      resetKeyboardFocus();
      // Trigger results re-render for animation
      resultsKey++;
    } catch (error) {
      searchError = String(error);
      searchResults = null;
    } finally {
      isSearching = false;
    }
  }

  function clearFilters() {
    filters = {
      file_type: undefined,
      min_size: undefined,
      max_size: undefined,
      date_from: undefined,
      date_to: undefined,
    };
    selectedSizePreset = 'any';
    selectedDatePreset = 'any';
    if (query.trim()) {
      handleSearch();
    }
  }

  function removeFilter(key: string) {
    if (key === 'file_type') {
      filters.file_type = undefined;
    } else if (key === 'size') {
      filters.min_size = undefined;
      filters.max_size = undefined;
      selectedSizePreset = 'any';
    } else if (key === 'date') {
      filters.date_from = undefined;
      filters.date_to = undefined;
      selectedDatePreset = 'any';
    }

    if (query.trim()) {
      handleSearch();
    }
  }

  function handleFileTypeChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    filters.file_type = target.value || undefined;
    if (query.trim()) {
      handleSearch();
    }
  }

  function handleSizePresetChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    selectedSizePreset = target.value;
    const preset = sizePresetOptions.find(opt => opt.value === target.value);
    if (preset) {
      filters.min_size = preset.min;
      filters.max_size = preset.max;
      if (query.trim()) {
        handleSearch();
      }
    }
  }

  function handleDatePresetChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    selectedDatePreset = target.value;
    const preset = datePresetOptions.find(opt => opt.value === target.value);
    if (preset) {
      if (preset.days === null) {
        filters.date_from = undefined;
        filters.date_to = undefined;
      } else {
        const date = new Date();
        date.setDate(date.getDate() - preset.days);
        filters.date_from = date.toISOString().split('T')[0];
        filters.date_to = undefined;
      }
      if (query.trim()) {
        handleSearch();
      }
    }
  }

  function formatDateChip(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays <= 7) return `Last ${diffDays} days`;
    if (diffDays <= 30) return 'Last month';
    if (diffDays <= 90) return 'Last 3 months';
    return `Since ${date.toLocaleDateString()}`;
  }

  function savePreset() {
    if (!presetName.trim()) return;
    if (typeof window === 'undefined') return;

    const preset: FilterPreset = {
      name: presetName.trim(),
      filters: { ...filters }
    };

    filterPresets = [...filterPresets, preset];
    localStorage.setItem('cortex-filter-presets', JSON.stringify(filterPresets));

    presetName = '';
    showPresetSave = false;
  }

  function loadPreset(preset: FilterPreset) {
    filters = { ...preset.filters };

    // Update UI selects
    if (filters.file_type) {
      // File type is already set
    }

    // Find matching size preset
    const sizePreset = sizePresetOptions.find(opt =>
      opt.min === filters.min_size && opt.max === filters.max_size
    );
    selectedSizePreset = sizePreset?.value || 'any';

    // Reset date preset to any (custom dates from preset)
    selectedDatePreset = filters.date_from ? 'any' : 'any';

    if (query.trim()) {
      handleSearch();
    }
  }

  function deletePreset(index: number) {
    if (typeof window === 'undefined') return;
    filterPresets = filterPresets.filter((_, i) => i !== index);
    localStorage.setItem('cortex-filter-presets', JSON.stringify(filterPresets));
  }

  function selectFile(fileId: number, filename: string, path: string) {
    selectedFileId = fileId;
    onFileSelect(fileId, filename, path);
  }

  // Keyboard Navigation
  function handleKeyboardNavigation(event: KeyboardEvent) {
    if (!searchResults || searchResults.results.length === 0) return;

    const results = searchResults.results;

    // Cmd/Ctrl + ? - Show shortcuts modal
    if ((event.metaKey || event.ctrlKey) && event.key === '?') {
      event.preventDefault();
      showShortcutsModal = true;
      return;
    }

    // Arrow Down - Move to next result
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      if (keyboardFocusedIndex < results.length - 1) {
        keyboardFocusedIndex++;
        scrollToFocusedResult();
      }
      return;
    }

    // Arrow Up - Move to previous result
    if (event.key === 'ArrowUp') {
      event.preventDefault();
      if (keyboardFocusedIndex > 0) {
        keyboardFocusedIndex--;
        scrollToFocusedResult();
      } else if (keyboardFocusedIndex === -1 && results.length > 0) {
        keyboardFocusedIndex = results.length - 1;
        scrollToFocusedResult();
      }
      return;
    }

    // Enter or Space - Select focused result
    if ((event.key === 'Enter' || event.key === ' ') && keyboardFocusedIndex >= 0) {
      event.preventDefault();
      const result = results[keyboardFocusedIndex];
      selectFile(result.file_id, result.filename, result.path);
      return;
    }

    // Escape - Clear keyboard focus
    if (event.key === 'Escape') {
      keyboardFocusedIndex = -1;
      if (showShortcutsModal) {
        showShortcutsModal = false;
      }
      return;
    }
  }

  function scrollToFocusedResult() {
    // Scroll the focused result into view
    setTimeout(() => {
      const element = document.querySelector(`[data-result-index="${keyboardFocusedIndex}"]`);
      if (element) {
        element.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      }
    }, 0);
  }

  function resetKeyboardFocus() {
    keyboardFocusedIndex = -1;
  }

  // Load presets on mount
  function loadPresets() {
    // Only run in browser environment (not during SSR)
    if (typeof window === 'undefined') return;

    try {
      const saved = localStorage.getItem('cortex-filter-presets');
      if (saved) {
        filterPresets = JSON.parse(saved);
      }
    } catch (error) {
      console.error('Failed to load filter presets:', error);
    }
  }

  // Initialize (will run on client-side only)
  loadPresets();
</script>

<svelte:window onkeydown={handleKeyboardNavigation} />

<main class="flex-1 flex flex-col bg-cortex-black h-full overflow-hidden">
  <!-- Search Mode Toggle -->
  <div class="px-6 pt-4 bg-slate-byte border-b border-neural-gold/10">
    <div class="max-w-4xl">
      <div class="flex items-center gap-2 bg-cortex-black rounded-lg p-1 w-fit">
        <button
          onclick={() => searchMode = 'keyword'}
          class={`px-4 py-2 rounded-md text-sm font-medium transition-all ${
            searchMode === 'keyword'
              ? 'bg-neural-gold text-cortex-black'
              : 'text-silver-neural/70 hover:text-silver-neural hover:bg-cortex-deep'
          }`}
        >
          <span class="flex items-center gap-2">
            <span>🔍</span>
            <span>Keyword Search</span>
          </span>
        </button>
        <button
          onclick={() => searchMode = 'semantic'}
          class={`px-4 py-2 rounded-md text-sm font-medium transition-all ${
            searchMode === 'semantic'
              ? 'bg-gradient-to-r from-purple-500 to-neural-gold text-white'
              : 'text-silver-neural/70 hover:text-silver-neural hover:bg-cortex-deep'
          }`}
        >
          <span class="flex items-center gap-2">
            <span>🧠</span>
            <span>Semantic Search</span>
            <span class="text-xs px-1.5 py-0.5 rounded bg-purple-500/20 text-purple-300">AI</span>
          </span>
        </button>
      </div>
    </div>
  </div>

  {#if searchMode === 'keyword'}
  <!-- Keyword Search Interface -->
  <div class="p-6 border-b border-neural-gold/20 bg-slate-byte">
    <div class="max-w-4xl">
      <!-- Search Input -->
      <div class="relative">
        <input
          type="text"
          bind:value={query}
          oninput={handleSearchInput}
          placeholder="Search files... (Cmd+K)"
          class="w-full px-4 py-3 pl-12 bg-cortex-black text-silver-neural rounded-lg border border-neural-gold/30 focus:border-neural-gold focus:outline-none focus:ring-2 focus:ring-neural-gold/20 placeholder-silver-neural/50 transition-all"
          disabled={isSearching}
        />
        <span class="absolute left-4 top-1/2 -translate-y-1/2 text-xl">🔍</span>
        {#if isSearching}
          <span class="absolute right-4 top-1/2 -translate-y-1/2 text-neural-gold animate-spin">⏳</span>
        {/if}
      </div>

      <!-- Filter Chips & Controls -->
      <div class="flex items-center gap-3 mt-3 flex-wrap">
        <button
          type="button"
          onclick={() => showFilters = !showFilters}
          class="text-sm text-neural-gold hover:text-ember-gold transition-colors flex items-center gap-2 px-3 py-1.5 rounded-lg border border-neural-gold/30 hover:border-neural-gold/50 bg-cortex-black/50"
        >
          <span class="text-xs">{showFilters ? '▼' : '▶'}</span>
          <span>Filters</span>
          {#if activeFiltersCount() > 0}
            <span class="px-1.5 py-0.5 bg-neural-gold text-cortex-black rounded text-xs font-semibold">
              {activeFiltersCount()}
            </span>
          {/if}
        </button>

        <!-- Active Filter Chips -->
        {#each activeFilterChips() as chip, index (chip.key)}
          <div class="flex items-center gap-1.5 px-3 py-1.5 bg-neural-gold/10 border border-neural-gold/30 rounded-lg text-sm chip-fade-in" style="animation-delay: {index * 50}ms;" transition:fly={{ x: -10, duration: 200 }}>
            <span class="text-silver-neural/70 text-xs">{chip.label}:</span>
            <span class="text-neural-gold font-medium">{chip.value}</span>
            <button
              onclick={() => removeFilter(chip.key)}
              class="ml-1 text-silver-neural/50 hover:text-red-400 transition-colors"
              title="Remove filter"
            >
              ×
            </button>
          </div>
        {/each}

        {#if activeFiltersCount() > 0}
          <button
            type="button"
            onclick={clearFilters}
            class="text-sm text-silver-neural/70 hover:text-silver-neural transition-colors px-3 py-1.5 rounded-lg hover:bg-cortex-black/50"
          >
            Clear All
          </button>
        {/if}

        {#if searchResults}
          <span class="ml-auto text-sm text-silver-neural/60">
            {searchResults.total} results • {searchResults.query_time_ms}ms
          </span>
        {/if}
      </div>

      <!-- Filters Panel -->
      {#if showFilters}
        <div class="mt-4 p-4 bg-cortex-black rounded-lg border border-neural-gold/20 space-y-4">
          <!-- Filter Grid -->
          <div class="grid grid-cols-3 gap-4">
            <!-- File Type -->
            <div>
              <label for="fileType" class="block text-xs font-semibold text-neural-gold mb-2">
                📋 File Type
              </label>
              <select
                id="fileType"
                value={filters.file_type || ''}
                onchange={handleFileTypeChange}
                class="w-full px-3 py-2 bg-slate-byte text-silver-neural rounded border border-neural-gold/20 focus:border-neural-gold focus:outline-none text-sm cursor-pointer hover:border-neural-gold/40 transition-colors"
              >
                {#each fileTypeOptions as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>

            <!-- File Size -->
            <div>
              <label for="sizePreset" class="block text-xs font-semibold text-neural-gold mb-2">
                💾 File Size
              </label>
              <select
                id="sizePreset"
                bind:value={selectedSizePreset}
                onchange={handleSizePresetChange}
                class="w-full px-3 py-2 bg-slate-byte text-silver-neural rounded border border-neural-gold/20 focus:border-neural-gold focus:outline-none text-sm cursor-pointer hover:border-neural-gold/40 transition-colors"
              >
                {#each sizePresetOptions as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>

            <!-- Date Range -->
            <div>
              <label for="datePreset" class="block text-xs font-semibold text-neural-gold mb-2">
                📅 Modified Date
              </label>
              <select
                id="datePreset"
                bind:value={selectedDatePreset}
                onchange={handleDatePresetChange}
                class="w-full px-3 py-2 bg-slate-byte text-silver-neural rounded border border-neural-gold/20 focus:border-neural-gold focus:outline-none text-sm cursor-pointer hover:border-neural-gold/40 transition-colors"
              >
                {#each datePresetOptions as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>
          </div>

          <!-- Filter Presets -->
          <div class="pt-4 border-t border-neural-gold/10">
            <div class="flex items-center justify-between mb-3">
              <h3 class="text-xs font-semibold text-neural-gold">⭐ Saved Filters</h3>
              <button
                onclick={() => showPresetSave = !showPresetSave}
                class="text-xs text-neural-gold hover:text-ember-gold transition-colors px-2 py-1 rounded border border-neural-gold/30 hover:border-neural-gold/50"
                disabled={activeFiltersCount() === 0}
              >
                {showPresetSave ? 'Cancel' : '+ Save Current'}
              </button>
            </div>

            {#if showPresetSave}
              <div class="flex gap-2 mb-3">
                <input
                  type="text"
                  bind:value={presetName}
                  placeholder="Preset name..."
                  class="flex-1 px-3 py-2 bg-slate-byte text-silver-neural rounded border border-neural-gold/20 focus:border-neural-gold focus:outline-none text-sm placeholder-silver-neural/40"
                  onkeydown={(e) => e.key === 'Enter' && savePreset()}
                />
                <button
                  onclick={savePreset}
                  class="px-4 py-2 bg-neural-gold text-cortex-black rounded hover:bg-ember-gold transition-colors text-sm font-semibold"
                  disabled={!presetName.trim()}
                >
                  Save
                </button>
              </div>
            {/if}

            {#if filterPresets.length > 0}
              <div class="flex flex-wrap gap-2">
                {#each filterPresets as preset, index}
                  <div class="flex items-center gap-2 px-3 py-2 bg-slate-byte rounded-lg border border-neural-gold/20 text-sm group hover:border-neural-gold/40 transition-colors">
                    <button
                      onclick={() => loadPreset(preset)}
                      class="text-silver-neural hover:text-neural-gold transition-colors"
                    >
                      {preset.name}
                    </button>
                    <button
                      onclick={() => deletePreset(index)}
                      class="text-silver-neural/40 hover:text-red-400 transition-colors opacity-0 group-hover:opacity-100"
                      title="Delete preset"
                    >
                      ×
                    </button>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="text-xs text-silver-neural/50 italic">No saved filters yet</p>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Error Message -->
      {#if searchError}
        <div class="mt-3 p-4 bg-red-900/30 border border-red-700 rounded-lg">
          <div class="flex items-start gap-3">
            <span class="text-2xl">⚠️</span>
            <div class="flex-1">
              <p class="text-sm font-semibold text-red-300 mb-1">Search Error</p>
              <p class="text-sm text-red-200/80 mb-3">{searchError}</p>
              <button
                onclick={() => {
                  searchError = null;
                  if (query.trim()) handleSearch();
                }}
                class="px-3 py-1.5 bg-red-700/50 hover:bg-red-700/70 text-red-100 rounded text-xs font-medium transition-colors"
              >
                Try Again
              </button>
            </div>
            <button
              onclick={() => searchError = null}
              class="text-red-300/50 hover:text-red-300 transition-colors"
              title="Dismiss"
            >
              ×
            </button>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Results Area (Scrollable) -->
  <div class="flex-1 overflow-y-auto p-6">
    {#if isSearching}
      <!-- Loading Skeleton -->
      <div class="max-w-4xl space-y-2">
        {#each Array(5) as _, i}
          <div class="w-full p-4 rounded-lg border border-neural-gold/10 bg-slate-byte animate-pulse">
            <div class="flex items-start justify-between gap-4">
              <div class="flex-1 space-y-3">
                <div class="h-5 bg-neural-gold/10 rounded w-1/3"></div>
                <div class="h-3 bg-neural-gold/5 rounded w-2/3"></div>
                <div class="h-3 bg-neural-gold/5 rounded w-1/2"></div>
              </div>
              <div class="h-4 bg-neural-gold/10 rounded w-16"></div>
            </div>
          </div>
        {/each}
      </div>
    {:else if searchResults}
      {#if hasResults}
        <div class="max-w-4xl space-y-2">
          {#each searchResults.results as result, index (resultsKey + '-' + result.file_id)}
            <button
              data-result-index={index}
              onclick={() => {
                selectFile(result.file_id, result.filename, result.path);
                keyboardFocusedIndex = index;
              }}
              onmouseenter={() => keyboardFocusedIndex = index}
              class={`w-full text-left p-4 rounded-lg border transition-all result-fade-in ${
                selectedFileId === result.file_id
                  ? 'bg-neural-gold/10 border-neural-gold'
                  : keyboardFocusedIndex === index
                  ? 'bg-neural-gold/5 border-neural-gold/60 ring-2 ring-neural-gold/30'
                  : 'bg-slate-byte border-neural-gold/20 hover:border-neural-gold/50 hover:bg-slate-byte/80'
              }`}
              style="animation-delay: {index * 30}ms;"
            >
              <div class="flex items-start justify-between gap-4">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1">
                    <h3 class="font-semibold text-silver-neural truncate">{result.filename}</h3>
                    <span class={`text-xs px-2 py-0.5 rounded ${getFileTypeInfo(result.filename.split('.').pop() || '').color}`}>
                      {getFileTypeInfo(result.filename.split('.').pop() || '').label}
                    </span>
                  </div>
                  <p class="text-xs text-silver-neural/50 truncate mb-2" title={result.path}>
                    {result.path}
                  </p>
                  {#if result.snippet}
                    <div class="text-sm text-silver-neural/90 line-clamp-2 snippet">
                      {@html result.snippet}
                    </div>
                  {/if}
                </div>
                <div class="text-xs text-silver-neural/50 flex-shrink-0">
                  <span>Score: {result.score.toFixed(2)}</span>
                </div>
              </div>
            </button>
          {/each}
        </div>
      {:else}
        <!-- No Results Empty State -->
        <div class="flex flex-col items-center justify-center h-full text-center px-4" transition:fade={{ duration: 200 }}>
          <span class="text-6xl mb-4">🔍</span>
          <p class="text-xl font-semibold text-silver-neural/80 mb-2">No results found</p>
          <p class="text-sm text-silver-neural/60 mb-6 max-w-md">We couldn't find any files matching "<span class="text-neural-gold font-medium">{query}</span>"</p>

          <!-- Suggestions -->
          <div class="bg-slate-byte rounded-lg border border-neural-gold/20 p-6 max-w-lg mb-6">
            <p class="text-sm font-semibold text-neural-gold mb-3">Try these suggestions:</p>
            <ul class="text-left text-sm text-silver-neural/70 space-y-2">
              <li class="flex items-start gap-2">
                <span class="text-neural-gold">•</span>
                <span>Check your spelling or try different keywords</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-neural-gold">•</span>
                <span>Use fewer or more general search terms</span>
              </li>
              {#if activeFiltersCount() > 0}
                <li class="flex items-start gap-2">
                  <span class="text-neural-gold">•</span>
                  <span>Remove some filters to broaden your search</span>
                </li>
              {/if}
              {#if stats && stats.indexed_files === 0}
                <li class="flex items-start gap-2">
                  <span class="text-neural-gold">•</span>
                  <span>Index more directories in <a href="/settings" class="text-neural-gold hover:text-ember-gold underline">Settings</a></span>
                </li>
              {/if}
            </ul>
          </div>

          <!-- Actions -->
          <div class="flex gap-3">
            {#if activeFiltersCount() > 0}
              <button
                onclick={clearFilters}
                class="px-4 py-2 bg-neural-gold/10 text-neural-gold rounded-lg border border-neural-gold/30 hover:bg-neural-gold/20 transition-colors text-sm font-medium"
              >
                Clear Filters
              </button>
            {/if}
            <a
              href="/settings"
              class="px-4 py-2 bg-slate-byte text-silver-neural rounded-lg border border-neural-gold/30 hover:border-neural-gold/50 transition-colors text-sm font-medium"
            >
              Manage Indexed Folders
            </a>
          </div>
        </div>
      {/if}
    {:else if stats && stats.indexed_files === 0}
      <!-- No Indexed Files State -->
      <div class="flex flex-col items-center justify-center h-full text-center px-4" transition:fade={{ duration: 200 }}>
        <span class="text-6xl mb-4">📂</span>
        <p class="text-2xl font-semibold text-silver-neural/80 mb-2">No files indexed yet</p>
        <p class="text-sm text-silver-neural/60 mb-8 max-w-md">Start by adding folders to index in Settings. Cortex will scan and index all your files for lightning-fast search.</p>

        <!-- Feature Cards -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 max-w-3xl mb-8">
          <div class="bg-slate-byte rounded-lg border border-neural-gold/20 p-4 text-left">
            <div class="text-2xl mb-2">⚡</div>
            <p class="text-sm font-semibold text-neural-gold mb-1">Lightning Fast</p>
            <p class="text-xs text-silver-neural/60">Full-text search across all your documents in milliseconds</p>
          </div>
          <div class="bg-slate-byte rounded-lg border border-neural-gold/20 p-4 text-left">
            <div class="text-2xl mb-2">🔒</div>
            <p class="text-sm font-semibold text-neural-gold mb-1">Private & Secure</p>
            <p class="text-xs text-silver-neural/60">All data stays on your machine. No cloud, no tracking</p>
          </div>
          <div class="bg-slate-byte rounded-lg border border-neural-gold/20 p-4 text-left">
            <div class="text-2xl mb-2">🎯</div>
            <p class="text-sm font-semibold text-neural-gold mb-1">Smart Filters</p>
            <p class="text-xs text-silver-neural/60">Filter by type, size, date, and save your favorite searches</p>
          </div>
        </div>

        <!-- CTA -->
        <a
          href="/settings"
          class="px-6 py-3 bg-neural-gold text-cortex-black rounded-lg hover:bg-ember-gold transition-colors font-semibold shadow-lg"
        >
          Add Folders to Index
        </a>
      </div>
    {:else}
      <!-- Initial State - Ready to Search -->
      <div class="flex flex-col items-center justify-center h-full text-center px-4" transition:fade={{ duration: 200 }}>
        <span class="text-6xl mb-4">🧠</span>
        <p class="text-xl font-semibold text-silver-neural/80 mb-2">Start searching your files</p>
        <p class="text-sm text-silver-neural/60 mb-6">Type in the search bar above or press <kbd class="px-2 py-1 bg-slate-byte rounded border border-neural-gold/30 text-neural-gold font-mono">Cmd+K</kbd></p>

        {#if filterPresets.length > 0}
          <div class="mt-2">
            <p class="text-xs text-silver-neural/60 mb-3">Quick filters:</p>
            <div class="flex flex-wrap gap-2 justify-center">
              {#each filterPresets.slice(0, 3) as preset}
                <button
                  onclick={() => loadPreset(preset)}
                  class="px-3 py-2 bg-neural-gold/10 text-neural-gold rounded-lg border border-neural-gold/30 hover:bg-neural-gold/20 transition-colors text-sm"
                >
                  {preset.name}
                </button>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Quick Tips -->
        {#if stats && stats.indexed_files > 0}
          <div class="mt-8 bg-slate-byte rounded-lg border border-neural-gold/20 p-4 max-w-md">
            <p class="text-xs font-semibold text-neural-gold mb-2">💡 Pro Tips:</p>
            <ul class="text-left text-xs text-silver-neural/60 space-y-1">
              <li>• Use quotes for exact phrases: "error message"</li>
              <li>• Press ↑/↓ to navigate results with keyboard</li>
              <li>• Press Cmd+? to see all keyboard shortcuts</li>
            </ul>
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Keyboard Shortcuts Help Modal -->
  {#if showShortcutsModal}
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="shortcuts-modal-title"
      tabindex="-1"
      class="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50"
      onclick={() => showShortcutsModal = false}
      onkeydown={(e) => e.key === 'Escape' && (showShortcutsModal = false)}
      transition:fade={{ duration: 150 }}
    >
      <div
        role="document"
        class="bg-slate-byte border-2 border-neural-gold rounded-xl p-8 max-w-2xl w-full mx-4 shadow-2xl"
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => e.stopPropagation()}
        transition:fly={{ y: -20, duration: 200 }}
      >
        <div class="flex items-center justify-between mb-6">
          <h2 id="shortcuts-modal-title" class="text-2xl font-bold text-neural-gold flex items-center gap-3">
            ⌨️ Keyboard Shortcuts
          </h2>
          <button
            onclick={() => showShortcutsModal = false}
            class="text-silver-neural/50 hover:text-silver-neural text-2xl transition-colors"
            title="Close (Esc)"
          >
            ×
          </button>
        </div>

        <div class="space-y-6">
          <!-- Search -->
          <div>
            <h3 class="text-sm font-semibold text-neural-gold/70 mb-3 uppercase tracking-wider">Search</h3>
            <div class="space-y-2">
              <div class="flex justify-between items-center p-3 bg-cortex-black/50 rounded-lg">
                <span class="text-silver-neural">Focus search bar</span>
                <kbd class="px-3 py-1.5 bg-slate-byte border border-neural-gold/30 rounded text-neural-gold font-mono text-sm">Cmd+K</kbd>
              </div>
              <div class="flex justify-between items-center p-3 bg-cortex-black/50 rounded-lg">
                <span class="text-silver-neural">Clear search</span>
                <kbd class="px-3 py-1.5 bg-slate-byte border border-neural-gold/30 rounded text-neural-gold font-mono text-sm">Esc</kbd>
              </div>
            </div>
          </div>

          <!-- Navigation -->
          <div>
            <h3 class="text-sm font-semibold text-neural-gold/70 mb-3 uppercase tracking-wider">Navigation</h3>
            <div class="space-y-2">
              <div class="flex justify-between items-center p-3 bg-cortex-black/50 rounded-lg">
                <span class="text-silver-neural">Move down in results</span>
                <kbd class="px-3 py-1.5 bg-slate-byte border border-neural-gold/30 rounded text-neural-gold font-mono text-sm">↓</kbd>
              </div>
              <div class="flex justify-between items-center p-3 bg-cortex-black/50 rounded-lg">
                <span class="text-silver-neural">Move up in results</span>
                <kbd class="px-3 py-1.5 bg-slate-byte border border-neural-gold/30 rounded text-neural-gold font-mono text-sm">↑</kbd>
              </div>
              <div class="flex justify-between items-center p-3 bg-cortex-black/50 rounded-lg">
                <span class="text-silver-neural">Select focused result</span>
                <div class="flex gap-2">
                  <kbd class="px-3 py-1.5 bg-slate-byte border border-neural-gold/30 rounded text-neural-gold font-mono text-sm">Enter</kbd>
                  <span class="text-silver-neural/50">or</span>
                  <kbd class="px-3 py-1.5 bg-slate-byte border border-neural-gold/30 rounded text-neural-gold font-mono text-sm">Space</kbd>
                </div>
              </div>
            </div>
          </div>

          <!-- General -->
          <div>
            <h3 class="text-sm font-semibold text-neural-gold/70 mb-3 uppercase tracking-wider">General</h3>
            <div class="space-y-2">
              <div class="flex justify-between items-center p-3 bg-cortex-black/50 rounded-lg">
                <span class="text-silver-neural">Show this help</span>
                <kbd class="px-3 py-1.5 bg-slate-byte border border-neural-gold/30 rounded text-neural-gold font-mono text-sm">Cmd+?</kbd>
              </div>
              <div class="flex justify-between items-center p-3 bg-cortex-black/50 rounded-lg">
                <span class="text-silver-neural">Close modal / Clear focus</span>
                <kbd class="px-3 py-1.5 bg-slate-byte border border-neural-gold/30 rounded text-neural-gold font-mono text-sm">Esc</kbd>
              </div>
            </div>
          </div>
        </div>

        <div class="mt-6 pt-6 border-t border-neural-gold/20 text-center">
          <p class="text-sm text-silver-neural/60">
            Press <kbd class="px-2 py-1 bg-slate-byte border border-neural-gold/20 rounded text-neural-gold/70 font-mono text-xs">Esc</kbd> to close
          </p>
        </div>
      </div>
    </div>
  {/if}

  {:else}
  <!-- Semantic Search Interface -->
  <SemanticSearch {onFileSelect} />
  {/if}
</main>

<style>
  :global(.snippet mark) {
    background-color: #C9A46C;
    color: #0A0A0C;
    padding: 0 2px;
    border-radius: 2px;
    font-weight: 600;
  }

  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes fadeInUp {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .animate-spin {
    animation: spin 1s linear infinite;
  }

  .result-fade-in {
    animation: fadeInUp 0.3s ease-out forwards;
    opacity: 0;
  }

  @keyframes chipFadeIn {
    from {
      opacity: 0;
      transform: scale(0.9);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .chip-fade-in {
    animation: chipFadeIn 0.2s ease-out forwards;
    opacity: 0;
  }

  select {
    appearance: none;
    background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3e%3cpath stroke='%23C9A46C' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3e%3c/svg%3e");
    background-position: right 0.5rem center;
    background-repeat: no-repeat;
    background-size: 1.5em 1.5em;
    padding-right: 2.5rem;
  }
</style>
