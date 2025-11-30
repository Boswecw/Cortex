<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { toastStore } from '$lib/stores/toastStore';
  import type { SemanticSearchResult, SemanticSearchFilters } from '$lib/types/api';

  // Props
  let { onFileSelect }: { onFileSelect: (fileId: number, filename: string, filepath: string) => void } = $props();

  // State
  let query = $state('');
  let results = $state<SemanticSearchResult[]>([]);
  let isSearching = $state(false);
  let searchTime = $state(0);
  let threshold = $state(0.7);
  let limit = $state(50);

  // Perform semantic search
  async function performSearch() {
    if (!query.trim()) {
      toastStore.warning('Please enter a search query');
      return;
    }

    isSearching = true;
    const startTime = performance.now();

    try {
      const filters: SemanticSearchFilters = {
        query: query.trim(),
        limit,
        threshold,
      };

      results = await invoke<SemanticSearchResult[]>('semantic_search', filters);
      searchTime = performance.now() - startTime;

      if (results.length === 0) {
        toastStore.info('No results found. Try lowering the similarity threshold.', 5000);
      }
    } catch (error) {
      console.error('Semantic search error:', error);
      toastStore.error(`Search failed: ${error}`, 7000);
      results = [];
    } finally {
      isSearching = false;
    }
  }

  // Handle Enter key
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      performSearch();
    }
  }

  // Format similarity score as percentage
  function formatScore(score: number): string {
    return `${(score * 100).toFixed(1)}%`;
  }

  // Get color for score
  function getScoreColor(score: number): string {
    if (score >= 0.9) return 'text-green-400';
    if (score >= 0.8) return 'text-neural-gold';
    if (score >= 0.7) return 'text-yellow-500';
    return 'text-gray-400';
  }
</script>

<div class="flex flex-col h-full">
  <!-- Search Header -->
  <div class="p-4 border-b border-neural-gold/20">
    <div class="flex items-center gap-2 mb-3">
      <span class="text-2xl">🧠</span>
      <h2 class="text-xl font-semibold text-neural-gold">Semantic Search</h2>
    </div>

    <!-- Search Input -->
    <div class="flex gap-2 mb-3">
      <input
        type="text"
        bind:value={query}
        onkeydown={handleKeydown}
        placeholder="Search by meaning... (e.g., 'authentication code')"
        class="flex-1 px-4 py-2 bg-slate-byte border border-neural-gold/30 rounded-lg
               text-silver-neural placeholder-gray-500
               focus:outline-none focus:border-neural-gold
               transition-colors"
      />
      <button
        onclick={performSearch}
        disabled={isSearching || !query.trim()}
        class="px-6 py-2 bg-neural-gold hover:bg-ember-gold text-cortex-black
               rounded-lg font-medium transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isSearching ? 'Searching...' : 'Search'}
      </button>
    </div>

    <!-- Advanced Options -->
    <div class="flex gap-4 text-sm">
      <div class="flex items-center gap-2">
        <label for="threshold" class="text-gray-400">Similarity:</label>
        <input
          id="threshold"
          type="range"
          bind:value={threshold}
          min="0.5"
          max="0.95"
          step="0.05"
          class="w-24"
        />
        <span class="text-neural-gold font-medium">{formatScore(threshold)}</span>
      </div>
      <div class="flex items-center gap-2">
        <label for="limit" class="text-gray-400">Results:</label>
        <select
          id="limit"
          bind:value={limit}
          class="px-2 py-1 bg-slate-byte border border-neural-gold/30 rounded
                 text-silver-neural text-sm"
        >
          <option value={10}>10</option>
          <option value={25}>25</option>
          <option value={50}>50</option>
          <option value={100}>100</option>
        </select>
      </div>
    </div>
  </div>

  <!-- Search Info -->
  {#if results.length > 0}
    <div class="px-4 py-2 bg-slate-byte/50 border-b border-neural-gold/10 text-sm text-gray-400">
      Found <span class="text-neural-gold font-medium">{results.length}</span> similar files
      in <span class="text-neural-gold font-medium">{searchTime.toFixed(0)}ms</span>
    </div>
  {/if}

  <!-- Results List -->
  <div class="flex-1 overflow-y-auto p-4">
    {#if isSearching}
      <div class="flex items-center justify-center py-12">
        <div class="text-center">
          <div class="animate-spin text-4xl mb-2">🧠</div>
          <p class="text-gray-400">Analyzing semantic similarity...</p>
        </div>
      </div>
    {:else if results.length === 0 && query}
      <div class="text-center py-12 text-gray-400">
        <p class="text-4xl mb-2">🔍</p>
        <p class="text-lg mb-2">No semantically similar files found</p>
        <p class="text-sm">Try:</p>
        <ul class="text-sm mt-2 space-y-1">
          <li>• Lowering the similarity threshold</li>
          <li>• Using different search terms</li>
          <li>• Generating embeddings for more files</li>
        </ul>
      </div>
    {:else if results.length > 0}
      <div class="space-y-2">
        {#each results as result, index}
          <button
            onclick={() => onFileSelect(result.file_id, result.filename, result.path)}
            class="w-full text-left p-4 bg-slate-byte border border-neural-gold/20
                   rounded-lg hover:border-neural-gold transition-all
                   hover:shadow-lg hover:shadow-neural-gold/20"
          >
            <!-- Header -->
            <div class="flex items-start justify-between mb-2">
              <div class="flex items-center gap-2 flex-1 min-w-0">
                <span class="text-xl">📄</span>
                <h3 class="font-medium text-silver-neural truncate">
                  {result.filename}
                </h3>
              </div>
              <div class="flex items-center gap-2 ml-2">
                <!-- Similarity Score -->
                <div class="flex items-center gap-1">
                  <span class="text-xs text-gray-400">Similarity:</span>
                  <span class={`font-bold text-sm ${getScoreColor(result.similarity_score)}`}>
                    {formatScore(result.similarity_score)}
                  </span>
                </div>
              </div>
            </div>

            <!-- File Path -->
            <p class="text-xs text-gray-500 truncate mb-2">
              {result.path}
            </p>

            <!-- Metadata -->
            <div class="flex items-center gap-3 text-xs text-gray-400">
              <span class="px-2 py-1 bg-neural-gold/20 text-neural-gold rounded">
                {result.file_type}
              </span>
              <span>Rank: #{index + 1}</span>
            </div>
          </button>
        {/each}
      </div>
    {:else}
      <div class="text-center py-12 text-gray-400">
        <p class="text-4xl mb-2">🧠</p>
        <p class="text-lg mb-2">Semantic Search</p>
        <p class="text-sm">Search by meaning, not just keywords</p>
        <div class="mt-4 text-sm space-y-1">
          <p><strong>Example queries:</strong></p>
          <p>"authentication code"</p>
          <p>"database connection"</p>
          <p>"error handling"</p>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  /* Custom range slider styling */
  input[type="range"] {
    -webkit-appearance: none;
    appearance: none;
    background: transparent;
  }

  input[type="range"]::-webkit-slider-track {
    background: #C9A46C33;
    height: 4px;
    border-radius: 2px;
  }

  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #C9A46C;
    cursor: pointer;
    margin-top: -6px;
  }

  input[type="range"]::-moz-range-track {
    background: #C9A46C33;
    height: 4px;
    border-radius: 2px;
  }

  input[type="range"]::-moz-range-thumb {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #C9A46C;
    cursor: pointer;
    border: none;
  }
</style>
