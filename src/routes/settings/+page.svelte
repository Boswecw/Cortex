<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import type { IndexStatus } from '$lib/types/api';

  // Settings State
  interface Settings {
    indexedDirectories: string[];
    excludedExtensions: string[];
    maxFileSizeMB: number;
    autoIndex: boolean;
    indexHiddenFiles: boolean;
    followSymlinks: boolean;
  }

  let settings = $state<Settings>({
    indexedDirectories: [],
    excludedExtensions: ['exe', 'dll', 'so', 'dylib', 'app', 'dmg', 'iso', 'zip', 'tar', 'gz'],
    maxFileSizeMB: 100,
    autoIndex: false,
    indexHiddenFiles: false,
    followSymlinks: false,
  });

  let newDirectory = $state('');
  let newExtension = $state('');
  let isSaving = $state(false);
  let saveMessage = $state<{ type: 'success' | 'error'; text: string } | null>(null);
  let indexStatus = $state<IndexStatus | null>(null);
  let isIndexing = $derived(indexStatus?.is_indexing ?? false);

  // Load settings
  onMount(async () => {
    loadSettings();
    await refreshIndexStatus();
  });

  function loadSettings() {
    const saved = localStorage.getItem('cortex-settings');
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        settings = { ...settings, ...parsed };
      } catch (e) {
        console.error('Failed to load settings:', e);
      }
    }
  }

  async function saveSettings() {
    isSaving = true;
    saveMessage = null;

    try {
      localStorage.setItem('cortex-settings', JSON.stringify(settings));
      saveMessage = { type: 'success', text: 'Settings saved successfully!' };

      setTimeout(() => {
        saveMessage = null;
      }, 3000);
    } catch (error) {
      saveMessage = { type: 'error', text: `Failed to save settings: ${error}` };
    } finally {
      isSaving = false;
    }
  }

  async function pickDirectory() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Directory to Index',
      });

      if (selected && typeof selected === 'string') {
        addDirectory(selected);
      }
    } catch (error) {
      console.error('Failed to pick directory:', error);
    }
  }

  function addDirectory(path: string) {
    if (path && !settings.indexedDirectories.includes(path)) {
      settings.indexedDirectories = [...settings.indexedDirectories, path];
      newDirectory = '';
      saveSettings();
    }
  }

  function removeDirectory(path: string) {
    settings.indexedDirectories = settings.indexedDirectories.filter(d => d !== path);
    saveSettings();
  }

  function addExtension() {
    const ext = newExtension.trim().toLowerCase().replace(/^\./, '');
    if (ext && !settings.excludedExtensions.includes(ext)) {
      settings.excludedExtensions = [...settings.excludedExtensions, ext];
      newExtension = '';
      saveSettings();
    }
  }

  function removeExtension(ext: string) {
    settings.excludedExtensions = settings.excludedExtensions.filter(e => e !== ext);
    saveSettings();
  }

  async function startIndexing() {
    if (settings.indexedDirectories.length === 0) {
      saveMessage = { type: 'error', text: 'Please add at least one directory to index' };
      return;
    }

    try {
      await invoke('start_indexing', { paths: settings.indexedDirectories });
      await refreshIndexStatus();
      saveMessage = { type: 'success', text: 'Indexing started!' };
      setTimeout(() => saveMessage = null, 3000);
    } catch (error) {
      saveMessage = { type: 'error', text: `Failed to start indexing: ${error}` };
    }
  }

  async function stopIndexing() {
    try {
      await invoke('stop_indexing');
      await refreshIndexStatus();
      saveMessage = { type: 'success', text: 'Indexing stopped' };
      setTimeout(() => saveMessage = null, 3000);
    } catch (error) {
      saveMessage = { type: 'error', text: `Failed to stop indexing: ${error}` };
    }
  }

  async function refreshIndexStatus() {
    try {
      indexStatus = await invoke('get_index_status');
    } catch (error) {
      console.error('Failed to get index status:', error);
    }
  }

  function resetToDefaults() {
    if (confirm('Reset all settings to defaults? This will not delete indexed data.')) {
      settings = {
        indexedDirectories: [],
        excludedExtensions: ['exe', 'dll', 'so', 'dylib', 'app', 'dmg', 'iso', 'zip', 'tar', 'gz'],
        maxFileSizeMB: 100,
        autoIndex: false,
        indexHiddenFiles: false,
        followSymlinks: false,
      };
      saveSettings();
    }
  }
</script>

<div class="h-full overflow-y-auto bg-cortex-black p-8">
  <div class="max-w-4xl mx-auto">
    <!-- Header -->
    <div class="mb-8">
      <h1 class="text-3xl font-bold text-neural-gold mb-2">Settings</h1>
      <p class="text-silver-neural/70">Configure how Cortex indexes and searches your files</p>
    </div>

    <!-- Save Message -->
    {#if saveMessage}
      <div class={`mb-6 p-4 rounded-lg border ${
        saveMessage.type === 'success'
          ? 'bg-green-900/30 border-green-700 text-green-300'
          : 'bg-red-900/30 border-red-700 text-red-300'
      }`}>
        {saveMessage.text}
      </div>
    {/if}

    <div class="space-y-6">

      <!-- Indexed Directories Section -->
      <section class="bg-slate-byte rounded-lg border border-neural-gold/20 p-6">
        <h2 class="text-xl font-semibold text-neural-gold mb-4 flex items-center gap-2">
          <span>📁</span> Indexed Directories
        </h2>
        <p class="text-sm text-silver-neural/70 mb-4">
          Directories that Cortex will scan and index. Add your Documents, Projects, or any folder you want to search.
        </p>

        <!-- Directory List -->
        {#if settings.indexedDirectories.length > 0}
          <div class="space-y-2 mb-4">
            {#each settings.indexedDirectories as dir}
              <div class="flex items-center justify-between bg-cortex-black rounded-lg p-3 border border-neural-gold/10">
                <div class="flex items-center gap-3 flex-1 min-w-0">
                  <span class="text-neural-gold">📂</span>
                  <span class="text-sm text-silver-neural truncate" title={dir}>{dir}</span>
                </div>
                <button
                  onclick={() => removeDirectory(dir)}
                  class="ml-3 px-3 py-1 text-sm text-red-400 hover:text-red-300 hover:bg-red-900/20 rounded transition-colors"
                >
                  Remove
                </button>
              </div>
            {/each}
          </div>
        {:else}
          <div class="bg-cortex-black rounded-lg p-6 border border-neural-gold/10 text-center mb-4">
            <span class="text-4xl mb-2 block">📂</span>
            <p class="text-silver-neural/60 text-sm">No directories added yet</p>
          </div>
        {/if}

        <!-- Add Directory -->
        <div class="flex gap-2">
          <input
            type="text"
            bind:value={newDirectory}
            placeholder="/home/user/Documents"
            class="flex-1 px-4 py-2 bg-cortex-black text-silver-neural rounded-lg border border-neural-gold/20 focus:border-neural-gold focus:outline-none text-sm placeholder-silver-neural/40"
            onkeypress={(e) => e.key === 'Enter' && newDirectory.trim() && addDirectory(newDirectory.trim())}
          />
          <button
            onclick={pickDirectory}
            class="px-4 py-2 bg-neural-gold/20 text-neural-gold rounded-lg hover:bg-neural-gold/30 transition-colors text-sm font-semibold border border-neural-gold/30"
          >
            Browse
          </button>
          <button
            onclick={() => newDirectory.trim() && addDirectory(newDirectory.trim())}
            disabled={!newDirectory.trim()}
            class="px-4 py-2 bg-neural-gold text-cortex-black rounded-lg hover:bg-ember-gold disabled:opacity-50 disabled:cursor-not-allowed transition-colors text-sm font-semibold"
          >
            Add
          </button>
        </div>

        <!-- Index Action -->
        {#if settings.indexedDirectories.length > 0}
          <div class="mt-4 pt-4 border-t border-neural-gold/20">
            {#if isIndexing}
              <button
                onclick={stopIndexing}
                class="w-full px-4 py-3 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors font-semibold"
              >
                Stop Indexing
              </button>
            {:else}
              <button
                onclick={startIndexing}
                class="w-full px-4 py-3 bg-neural-gold text-cortex-black rounded-lg hover:bg-ember-gold transition-colors font-semibold"
              >
                Start Indexing Now
              </button>
            {/if}
          </div>
        {/if}
      </section>

      <!-- File Type Exclusions Section -->
      <section class="bg-slate-byte rounded-lg border border-neural-gold/20 p-6">
        <h2 class="text-xl font-semibold text-neural-gold mb-4 flex items-center gap-2">
          <span>🚫</span> Excluded File Types
        </h2>
        <p class="text-sm text-silver-neural/70 mb-4">
          File extensions to skip during indexing (e.g., binaries, archives, large media files).
        </p>

        <!-- Extension Tags -->
        <div class="flex flex-wrap gap-2 mb-4">
          {#each settings.excludedExtensions as ext}
            <span class="inline-flex items-center gap-2 px-3 py-1 bg-cortex-black rounded-full border border-neural-gold/20 text-sm">
              <span class="text-silver-neural">.{ext}</span>
              <button
                onclick={() => removeExtension(ext)}
                class="text-red-400 hover:text-red-300 transition-colors"
              >
                ×
              </button>
            </span>
          {/each}
        </div>

        <!-- Add Extension -->
        <div class="flex gap-2">
          <input
            type="text"
            bind:value={newExtension}
            placeholder="exe, dll, zip..."
            class="flex-1 px-4 py-2 bg-cortex-black text-silver-neural rounded-lg border border-neural-gold/20 focus:border-neural-gold focus:outline-none text-sm placeholder-silver-neural/40"
            onkeypress={(e) => e.key === 'Enter' && newExtension.trim() && addExtension()}
          />
          <button
            onclick={addExtension}
            disabled={!newExtension.trim()}
            class="px-4 py-2 bg-neural-gold text-cortex-black rounded-lg hover:bg-ember-gold disabled:opacity-50 disabled:cursor-not-allowed transition-colors text-sm font-semibold"
          >
            Add Extension
          </button>
        </div>
      </section>

      <!-- Indexing Options Section -->
      <section class="bg-slate-byte rounded-lg border border-neural-gold/20 p-6">
        <h2 class="text-xl font-semibold text-neural-gold mb-4 flex items-center gap-2">
          <span>⚙️</span> Indexing Options
        </h2>

        <div class="space-y-4">
          <!-- Max File Size -->
          <div>
            <label class="block text-sm font-medium text-silver-neural mb-2">
              Maximum File Size (MB)
            </label>
            <input
              type="number"
              bind:value={settings.maxFileSizeMB}
              onchange={saveSettings}
              min="1"
              max="1000"
              class="w-full px-4 py-2 bg-cortex-black text-silver-neural rounded-lg border border-neural-gold/20 focus:border-neural-gold focus:outline-none"
            />
            <p class="text-xs text-silver-neural/60 mt-1">Files larger than this will be skipped</p>
          </div>

          <!-- Toggle Options -->
          <div class="space-y-3 pt-3 border-t border-neural-gold/20">
            <label class="flex items-center justify-between cursor-pointer group">
              <div>
                <span class="text-sm font-medium text-silver-neural">Auto-index on startup</span>
                <p class="text-xs text-silver-neural/60">Automatically start indexing when Cortex launches</p>
              </div>
              <input
                type="checkbox"
                bind:checked={settings.autoIndex}
                onchange={saveSettings}
                class="w-5 h-5 rounded border-neural-gold/30 bg-cortex-black checked:bg-neural-gold focus:ring-2 focus:ring-neural-gold/50"
              />
            </label>

            <label class="flex items-center justify-between cursor-pointer group">
              <div>
                <span class="text-sm font-medium text-silver-neural">Index hidden files</span>
                <p class="text-xs text-silver-neural/60">Include files and folders starting with "."</p>
              </div>
              <input
                type="checkbox"
                bind:checked={settings.indexHiddenFiles}
                onchange={saveSettings}
                class="w-5 h-5 rounded border-neural-gold/30 bg-cortex-black checked:bg-neural-gold focus:ring-2 focus:ring-neural-gold/50"
              />
            </label>

            <label class="flex items-center justify-between cursor-pointer group">
              <div>
                <span class="text-sm font-medium text-silver-neural">Follow symbolic links</span>
                <p class="text-xs text-silver-neural/60">Index files and folders linked via symlinks</p>
              </div>
              <input
                type="checkbox"
                bind:checked={settings.followSymlinks}
                onchange={saveSettings}
                class="w-5 h-5 rounded border-neural-gold/30 bg-cortex-black checked:bg-neural-gold focus:ring-2 focus:ring-neural-gold/50"
              />
            </label>
          </div>
        </div>
      </section>

      <!-- Actions -->
      <div class="flex gap-3">
        <button
          onclick={resetToDefaults}
          class="px-6 py-3 bg-cortex-black text-silver-neural rounded-lg border border-neural-gold/30 hover:bg-slate-byte transition-colors font-semibold"
        >
          Reset to Defaults
        </button>
        <button
          onclick={saveSettings}
          disabled={isSaving}
          class="flex-1 px-6 py-3 bg-neural-gold text-cortex-black rounded-lg hover:bg-ember-gold disabled:opacity-50 transition-colors font-semibold"
        >
          {isSaving ? 'Saving...' : 'Save Settings'}
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  /* Custom checkbox styling for better dark theme */
  input[type="checkbox"] {
    appearance: none;
    -webkit-appearance: none;
    cursor: pointer;
  }

  input[type="checkbox"]:checked {
    background-color: #C9A46C;
    border-color: #C9A46C;
  }

  input[type="checkbox"]:checked::after {
    content: '✓';
    display: block;
    text-align: center;
    color: #0A0A0C;
    font-size: 14px;
    line-height: 20px;
  }
</style>
