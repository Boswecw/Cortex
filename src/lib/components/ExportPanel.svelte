<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';
  import { onMount } from 'svelte';
  import type { ExportResult, ExportPreview, ExportStatsInfo } from '$lib/types/export';
  import { formatFileSize } from '$lib/types/api';

  // State
  let exportType = $state<'vscode' | 'rake'>('vscode');
  let includeEmbeddings = $state(false);
  let includePrompts = $state(true);
  let projectName = $state('');
  let customContext = $state('');
  let tenantId = $state('');
  let exportMode = $state<'full' | 'incremental' | 'collection'>('full');

  let preview = $state<ExportPreview | null>(null);
  let stats = $state<ExportStatsInfo | null>(null);
  let exporting = $state(false);
  let exportSuccess = $state(false);
  let exportError = $state<string | null>(null);
  let exportResult = $state<ExportResult | null>(null);

  // Load preview and stats on mount
  onMount(async () => {
    await Promise.all([
      loadPreview(),
      loadStats()
    ]);
  });

  // Load export preview
  async function loadPreview() {
    try {
      preview = await invoke('get_export_preview', {
        collectionId: null,
        includeEmbeddings: includeEmbeddings
      });
    } catch (error) {
      console.error('Failed to load preview:', error);
    }
  }

  // Load export stats
  async function loadStats() {
    try {
      stats = await invoke('get_export_stats');
    } catch (error) {
      console.error('Failed to load stats:', error);
    }
  }

  // Reload preview when settings change
  $effect(() => {
    // Dependencies that should trigger reload
    includeEmbeddings;
    loadPreview();
  });

  // Handle export
  async function handleExport() {
    exporting = true;
    exportSuccess = false;
    exportError = null;
    exportResult = null;

    try {
      if (exportType === 'vscode') {
        await exportVSCodeContext();
      } else {
        await exportRakePackage();
      }
      exportSuccess = true;
    } catch (error) {
      exportError = error instanceof Error ? error.message : String(error);
      console.error('Export failed:', error);
    } finally {
      exporting = false;
    }
  }

  // Export VS Code context
  async function exportVSCodeContext() {
    const outputPath = await save({
      title: 'Export VS Code Context',
      defaultPath: '.cortex-export',
      filters: []
    });

    if (!outputPath) {
      throw new Error('Export cancelled');
    }

    exportResult = await invoke('export_vscode_context', {
      collectionId: null,
      includeEmbeddings: includeEmbeddings,
      includePrompts: includePrompts,
      outputPath: outputPath,
      projectName: projectName || null,
      customContext: customContext || null
    });
  }

  // Export Rake package
  async function exportRakePackage() {
    if (!tenantId.trim()) {
      throw new Error('Tenant ID is required for Rake export');
    }

    const outputPath = await save({
      title: 'Export Rake Package',
      defaultPath: 'cortex-rake-export.json',
      filters: [{
        name: 'JSON',
        extensions: ['json']
      }]
    });

    if (!outputPath) {
      throw new Error('Export cancelled');
    }

    const filePath = await invoke<string>('export_rake_package', {
      collectionId: null,
      tenantId: tenantId,
      outputPath: outputPath,
      includeEmbeddings: includeEmbeddings,
      exportMode: exportMode
    });

    exportResult = {
      context_file: filePath,
      starter_prompt_file: '',
      prompt_files: [],
      stats: {
        total_files: preview?.file_count || 0,
        total_chunks: preview?.chunk_count || 0,
        total_size_bytes: preview?.estimated_size || 0,
        files_with_embeddings: preview?.embedded_file_count || 0,
        prompts_generated: 0
      },
      exported_at: new Date().toISOString()
    };
  }
</script>

<div class="export-panel h-full overflow-y-auto bg-cortex-black">
  <div class="max-w-4xl mx-auto p-8">
    <!-- Header -->
    <div class="mb-8">
      <h2 class="text-3xl font-bold text-neural-gold mb-2">Export for AI Development</h2>
      <p class="text-silver-neural/70">
        Export your indexed project for VS Code Claude or Rake pipeline integration
      </p>
    </div>

    <!-- Export Type Selector -->
    <div class="mb-8">
      <label class="block text-sm font-semibold text-silver-neural mb-3">Export Type</label>
      <div class="flex gap-4">
        <button
          class={`flex-1 p-6 rounded-xl border-2 transition-all ${
            exportType === 'vscode'
              ? 'bg-neural-gold/10 border-neural-gold text-neural-gold'
              : 'bg-cortex-deep border-silver-neural/20 text-silver-neural/80 hover:border-neural-gold/50'
          }`}
          onclick={() => exportType = 'vscode'}
        >
          <div class="text-4xl mb-3">💻</div>
          <div class="font-bold text-lg mb-2">VS Code Claude</div>
          <div class="text-xs opacity-70">
            Generate CONTEXT.md, prompts, and development templates
          </div>
        </button>

        <button
          class={`flex-1 p-6 rounded-xl border-2 transition-all ${
            exportType === 'rake'
              ? 'bg-neural-gold/10 border-neural-gold text-neural-gold'
              : 'bg-cortex-deep border-silver-neural/20 text-silver-neural/80 hover:border-neural-gold/50'
          }`}
          onclick={() => exportType = 'rake'}
        >
          <div class="text-4xl mb-3">🔥</div>
          <div class="font-bold text-lg mb-2">Rake / Forge</div>
          <div class="text-xs opacity-70">
            Export JSON package for Rake pipeline ingestion
          </div>
        </button>
      </div>
    </div>

    <!-- Preview Statistics -->
    {#if preview || stats}
      <div class="mb-8 p-6 bg-gradient-to-br from-neural-gold/5 to-purple-900/5 rounded-xl border border-neural-gold/20">
        <h3 class="text-lg font-bold text-neural-gold mb-4">Export Preview</h3>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          {#if stats}
            <div class="text-center">
              <div class="text-3xl font-bold text-neural-gold">{stats.total_files}</div>
              <div class="text-xs text-silver-neural/60 mt-1">Total Files</div>
            </div>
            <div class="text-center">
              <div class="text-3xl font-bold text-neural-gold">{stats.indexed_files}</div>
              <div class="text-xs text-silver-neural/60 mt-1">Indexed</div>
            </div>
            <div class="text-center">
              <div class="text-3xl font-bold text-purple-400">{stats.embedded_files}</div>
              <div class="text-xs text-silver-neural/60 mt-1">With Embeddings</div>
            </div>
            <div class="text-center">
              <div class="text-2xl font-bold text-neural-gold">{stats.total_size_human}</div>
              <div class="text-xs text-silver-neural/60 mt-1">Total Size</div>
            </div>
          {/if}
        </div>
        {#if preview}
          <div class="mt-4 pt-4 border-t border-neural-gold/20">
            <div class="grid grid-cols-2 gap-4 text-sm">
              <div class="flex justify-between">
                <span class="text-silver-neural/60">Estimated Chunks:</span>
                <span class="font-semibold text-silver-neural">{preview.chunk_count}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-silver-neural/60">Export Size:</span>
                <span class="font-semibold text-silver-neural">{preview.estimated_size_human}</span>
              </div>
            </div>
          </div>
        {/if}
      </div>
    {/if}

    <!-- VS Code Options -->
    {#if exportType === 'vscode'}
      <div class="space-y-6 mb-8">
        <div>
          <label class="block text-sm font-semibold text-silver-neural mb-2">
            Project Name (Optional)
          </label>
          <input
            type="text"
            bind:value={projectName}
            placeholder="Auto-detected if empty"
            class="w-full px-4 py-3 bg-cortex-deep border border-silver-neural/20 rounded-lg text-silver-neural placeholder-silver-neural/40 focus:border-neural-gold focus:outline-none transition-colors"
          />
        </div>

        <div>
          <label class="block text-sm font-semibold text-silver-neural mb-2">
            Custom Context (Optional)
          </label>
          <textarea
            bind:value={customContext}
            placeholder="Add specific context about what you're working on..."
            rows="3"
            class="w-full px-4 py-3 bg-cortex-deep border border-silver-neural/20 rounded-lg text-silver-neural placeholder-silver-neural/40 focus:border-neural-gold focus:outline-none transition-colors resize-none"
          ></textarea>
        </div>

        <div class="space-y-3">
          <label class="flex items-center gap-3 cursor-pointer group">
            <input
              type="checkbox"
              bind:checked={includePrompts}
              class="w-5 h-5 bg-cortex-deep border-2 border-silver-neural/30 rounded checked:bg-neural-gold checked:border-neural-gold transition-colors"
            />
            <div>
              <div class="text-sm font-medium text-silver-neural group-hover:text-neural-gold transition-colors">
                Include Prompt Templates
              </div>
              <div class="text-xs text-silver-neural/60">
                ADD_FEATURE, FIX_BUG, REFACTOR, ADD_TESTS, DOCUMENTATION
              </div>
            </div>
          </label>

          <label class="flex items-center gap-3 cursor-pointer group">
            <input
              type="checkbox"
              bind:checked={includeEmbeddings}
              class="w-5 h-5 bg-cortex-deep border-2 border-silver-neural/30 rounded checked:bg-neural-gold checked:border-neural-gold transition-colors"
            />
            <div>
              <div class="text-sm font-medium text-silver-neural group-hover:text-neural-gold transition-colors">
                Include Embeddings
              </div>
              <div class="text-xs text-silver-neural/60">
                {preview?.embedded_file_count || 0} files have embeddings
              </div>
            </div>
          </label>
        </div>

        <div class="p-4 bg-neural-gold/5 rounded-lg border border-neural-gold/20">
          <div class="text-sm text-silver-neural/80">
            <strong class="text-neural-gold">Output:</strong> Creates a <code>.cortex-export/</code> directory with:
            <ul class="list-disc list-inside ml-4 mt-2 space-y-1 text-xs text-silver-neural/70">
              <li>CONTEXT.md - Full project context</li>
              <li>STARTER_PROMPT.md - Initial development prompt</li>
              <li>prompts/ - Feature-specific templates</li>
              <li>.claude/ - VS Code Claude configuration</li>
            </ul>
          </div>
        </div>
      </div>
    {/if}

    <!-- Rake Options -->
    {#if exportType === 'rake'}
      <div class="space-y-6 mb-8">
        <div>
          <label class="block text-sm font-semibold text-silver-neural mb-2">
            Tenant ID <span class="text-red-400">*</span>
          </label>
          <input
            type="text"
            bind:value={tenantId}
            placeholder="your-tenant-id"
            required
            class="w-full px-4 py-3 bg-cortex-deep border border-silver-neural/20 rounded-lg text-silver-neural placeholder-silver-neural/40 focus:border-neural-gold focus:outline-none transition-colors"
          />
          <p class="text-xs text-silver-neural/60 mt-1">
            Multi-tenant identifier for Forge ecosystem
          </p>
        </div>

        <div>
          <label class="block text-sm font-semibold text-silver-neural mb-2">
            Export Mode
          </label>
          <select
            bind:value={exportMode}
            class="w-full px-4 py-3 bg-cortex-deep border border-silver-neural/20 rounded-lg text-silver-neural focus:border-neural-gold focus:outline-none transition-colors"
          >
            <option value="full">Full Export</option>
            <option value="incremental">Incremental (changes only)</option>
            <option value="collection">Collection Export</option>
          </select>
        </div>

        <label class="flex items-center gap-3 cursor-pointer group">
          <input
            type="checkbox"
            bind:checked={includeEmbeddings}
            class="w-5 h-5 bg-cortex-deep border-2 border-silver-neural/30 rounded checked:bg-neural-gold checked:border-neural-gold transition-colors"
          />
          <div>
            <div class="text-sm font-medium text-silver-neural group-hover:text-neural-gold transition-colors">
              Include Pre-computed Embeddings
            </div>
            <div class="text-xs text-silver-neural/60">
              Skip embedding generation in Rake pipeline
            </div>
          </div>
        </label>

        <div class="p-4 bg-neural-gold/5 rounded-lg border border-neural-gold/20">
          <div class="text-sm text-silver-neural/80">
            <strong class="text-neural-gold">Output:</strong> Generates a JSON package compatible with Rake V1 pipeline format with pre-chunked content (~500 tokens per chunk).
          </div>
        </div>
      </div>
    {/if}

    <!-- Export Button -->
    <button
      onclick={handleExport}
      disabled={exporting || (exportType === 'rake' && !tenantId.trim())}
      class="w-full py-4 bg-gradient-to-r from-neural-gold to-ember-gold text-cortex-black font-bold rounded-xl hover:shadow-lg hover:shadow-neural-gold/50 transition-all disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:shadow-none"
    >
      {#if exporting}
        <span class="flex items-center justify-center gap-2">
          <span class="animate-spin">⚡</span>
          Exporting...
        </span>
      {:else}
        Export {exportType === 'vscode' ? 'Context Bundle' : 'Rake Package'}
      {/if}
    </button>

    <!-- Success Message -->
    {#if exportSuccess && exportResult}
      <div class="mt-6 p-6 bg-green-900/20 border border-green-500/30 rounded-xl">
        <div class="flex items-start gap-3">
          <span class="text-3xl">✅</span>
          <div class="flex-1">
            <h4 class="text-lg font-bold text-green-400 mb-2">Export Successful!</h4>
            {#if exportType === 'vscode'}
              <div class="space-y-2 text-sm text-silver-neural/80">
                <p><strong class="text-green-300">Context File:</strong> {exportResult.context_file}</p>
                <p><strong class="text-green-300">Starter Prompt:</strong> {exportResult.starter_prompt_file}</p>
                <p><strong class="text-green-300">Templates:</strong> {exportResult.prompt_files.length} prompt files</p>
                <div class="mt-3 pt-3 border-t border-green-500/20">
                  <p class="text-xs text-silver-neural/60">
                    <strong>Next:</strong> Load CONTEXT.md and STARTER_PROMPT.md in VS Code Claude to begin development.
                  </p>
                </div>
              </div>
            {:else}
              <div class="space-y-2 text-sm text-silver-neural/80">
                <p><strong class="text-green-300">Export File:</strong> {exportResult.context_file}</p>
                <p><strong class="text-green-300">Files:</strong> {exportResult.stats.total_files}</p>
                <p><strong class="text-green-300">Size:</strong> {formatFileSize(exportResult.stats.total_size_bytes)}</p>
              </div>
            {/if}
          </div>
        </div>
      </div>
    {/if}

    <!-- Error Message -->
    {#if exportError}
      <div class="mt-6 p-6 bg-red-900/20 border border-red-500/30 rounded-xl">
        <div class="flex items-start gap-3">
          <span class="text-3xl">❌</span>
          <div class="flex-1">
            <h4 class="text-lg font-bold text-red-400 mb-2">Export Failed</h4>
            <p class="text-sm text-silver-neural/80">{exportError}</p>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .animate-spin {
    animation: spin 1s linear infinite;
  }
</style>
