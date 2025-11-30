<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { fade, scale } from 'svelte/transition';
  import type { IndexProgressEvent, IndexCompleteEvent } from '$lib/types/api';
  import { formatDuration } from '$lib/types/api';

  // Props
  let { onComplete }: { onComplete: () => void } = $props();

  // State
  let step = $state(1);
  let directories = $state<string[]>([]);
  let isIndexing = $state(false);
  let indexingProgress = $state({ current: 0, total: 0, percentage: 0, currentFile: '' });
  let indexingComplete = $state(false);
  let indexingError = $state<string | null>(null);
  let indexingStats = $state({ filesIndexed: 0, duration: 0 });

  // Event listeners
  let progressUnlisten: UnlistenFn | null = null;
  let completeUnlisten: UnlistenFn | null = null;

  const TOTAL_STEPS = 3;

  async function pickDirectory() {
    try {
      const selected = await open({
        directory: true,
        multiple: true,
        title: 'Select Directories to Index',
      });

      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        directories = [...directories, ...paths.filter(p => !directories.includes(p))];
      }
    } catch (error) {
      console.error('Failed to pick directory:', error);
    }
  }

  function removeDirectory(path: string) {
    directories = directories.filter(d => d !== path);
  }

  async function nextStep() {
    if (step === 2 && directories.length > 0) {
      // Save directories to settings
      const settings = {
        indexedDirectories: directories,
        excludedExtensions: ['exe', 'dll', 'so', 'dylib', 'app', 'dmg', 'iso', 'zip', 'tar', 'gz'],
        maxFileSizeMB: 100,
        autoIndex: false,
        indexHiddenFiles: false,
        followSymlinks: false,
      };
      localStorage.setItem('cortex-settings', JSON.stringify(settings));

      // Start indexing
      await startIndexing();
    }

    if (step < TOTAL_STEPS) {
      step++;
    }
  }

  function prevStep() {
    if (step > 1) {
      step--;
    }
  }

  async function startIndexing() {
    isIndexing = true;
    indexingError = null;

    // Setup event listeners
    progressUnlisten = await listen<IndexProgressEvent>('indexing:progress', (event) => {
      indexingProgress = {
        current: event.payload.indexed_files,
        total: event.payload.total_files,
        percentage: event.payload.progress_percentage,
        currentFile: event.payload.current_file,
      };
    });

    completeUnlisten = await listen<IndexCompleteEvent>('indexing:complete', (event) => {
      isIndexing = false;
      indexingComplete = true;
      indexingStats = {
        filesIndexed: event.payload.indexed_files,
        duration: event.payload.duration_seconds,
      };

      // Cleanup listeners
      progressUnlisten?.();
      completeUnlisten?.();
    });

    try {
      await invoke('start_indexing', { paths: directories });
    } catch (error) {
      indexingError = String(error);
      isIndexing = false;
    }
  }

  function finishOnboarding() {
    // Mark onboarding as complete
    localStorage.setItem('cortex-onboarding-complete', 'true');
    onComplete();
  }

  function skipOnboarding() {
    localStorage.setItem('cortex-onboarding-complete', 'true');
    onComplete();
  }
</script>

<div class="fixed inset-0 bg-cortex-black/95 backdrop-blur-sm z-50 flex items-center justify-center p-6" transition:fade={{ duration: 200 }}>
  <div class="bg-slate-byte border border-neural-gold/30 rounded-2xl shadow-2xl max-w-3xl w-full max-h-[90vh] overflow-hidden" transition:scale={{ start: 0.95, duration: 300 }}>

    <!-- Header -->
    <div class="bg-gradient-to-r from-cortex-deep to-slate-byte border-b border-neural-gold/20 p-8">
      <div class="flex items-center gap-4 mb-4">
        <span class="text-6xl">🧠</span>
        <div>
          <h1 class="text-3xl font-bold text-neural-gold">Welcome to Cortex</h1>
          <p class="text-silver-neural/70 text-sm mt-1">Local-first AI file intelligence</p>
        </div>
      </div>

      <!-- Progress Steps -->
      <div class="flex items-center gap-2 mt-6">
        {#each Array(TOTAL_STEPS) as _, i}
          <div class={`flex-1 h-1.5 rounded-full transition-all ${
            i + 1 <= step ? 'bg-neural-gold' : 'bg-cortex-black'
          }`}></div>
        {/each}
      </div>
      <div class="flex justify-between text-xs text-silver-neural/60 mt-2">
        <span>Welcome</span>
        <span>Setup</span>
        <span>Complete</span>
      </div>
    </div>

    <!-- Content -->
    <div class="p-8 overflow-y-auto" style="max-height: calc(90vh - 300px);">

      <!-- Step 1: Welcome -->
      {#if step === 1}
        <div class="space-y-6 text-center max-w-2xl mx-auto">
          <div>
            <h2 class="text-2xl font-bold text-neural-gold mb-3">Find Any File, Instantly</h2>
            <p class="text-silver-neural/80 text-lg leading-relaxed">
              Cortex indexes your files and makes them searchable in milliseconds.
              Think Spotlight or Everything, but smarter and completely private.
            </p>
          </div>

          <div class="grid grid-cols-3 gap-4 mt-8">
            <div class="bg-cortex-black rounded-lg p-4 border border-neural-gold/20">
              <span class="text-3xl mb-2 block">⚡</span>
              <h3 class="text-sm font-semibold text-neural-gold mb-1">Lightning Fast</h3>
              <p class="text-xs text-silver-neural/70">Search results in under 100ms</p>
            </div>
            <div class="bg-cortex-black rounded-lg p-4 border border-neural-gold/20">
              <span class="text-3xl mb-2 block">🔒</span>
              <h3 class="text-sm font-semibold text-neural-gold mb-1">100% Private</h3>
              <p class="text-xs text-silver-neural/70">All data stays on your machine</p>
            </div>
            <div class="bg-cortex-black rounded-lg p-4 border border-neural-gold/20">
              <span class="text-3xl mb-2 block">🧠</span>
              <h3 class="text-sm font-semibold text-neural-gold mb-1">AI-Powered</h3>
              <p class="text-xs text-silver-neural/70">Smart search with context</p>
            </div>
          </div>

          <div class="bg-neural-gold/10 border border-neural-gold/30 rounded-lg p-4 mt-6">
            <p class="text-sm text-silver-neural/80">
              <strong class="text-neural-gold">Quick setup:</strong> Just pick which folders to index,
              and Cortex will do the rest. You'll be searching in minutes.
            </p>
          </div>
        </div>
      {/if}

      <!-- Step 2: Directory Selection -->
      {#if step === 2}
        <div class="space-y-6">
          <div>
            <h2 class="text-2xl font-bold text-neural-gold mb-2">Select Folders to Index</h2>
            <p class="text-silver-neural/70">
              Choose the directories you want to search. Common choices: Documents, Projects, Downloads.
            </p>
          </div>

          <!-- Directory List -->
          {#if directories.length > 0}
            <div class="space-y-2">
              {#each directories as dir}
                <div class="flex items-center justify-between bg-cortex-black rounded-lg p-3 border border-neural-gold/20">
                  <div class="flex items-center gap-3 flex-1 min-w-0">
                    <span class="text-2xl">📂</span>
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
            <div class="bg-cortex-black rounded-lg p-8 border border-neural-gold/20 text-center">
              <span class="text-5xl mb-3 block">📁</span>
              <p class="text-silver-neural/60 mb-4">No folders selected yet</p>
              <p class="text-xs text-silver-neural/50">
                Tip: Start with your Documents or Projects folder
              </p>
            </div>
          {/if}

          <!-- Add Directory Button -->
          <button
            onclick={pickDirectory}
            class="w-full px-6 py-4 bg-neural-gold text-cortex-black rounded-lg hover:bg-ember-gold transition-colors font-semibold flex items-center justify-center gap-3"
          >
            <span class="text-2xl">➕</span>
            <span>Add Folder to Index</span>
          </button>

          <!-- Quick presets -->
          <div class="bg-cortex-black rounded-lg p-4 border border-neural-gold/20">
            <p class="text-xs font-semibold text-neural-gold mb-3">💡 Recommended folders:</p>
            <div class="grid grid-cols-2 gap-2 text-xs text-silver-neural/70">
              <div>• ~/Documents</div>
              <div>• ~/Downloads</div>
              <div>• ~/Projects</div>
              <div>• ~/Desktop</div>
            </div>
          </div>
        </div>
      {/if}

      <!-- Step 3: Indexing Progress / Complete -->
      {#if step === 3}
        <div class="space-y-6">
          {#if isIndexing}
            <!-- Indexing in Progress -->
            <div class="text-center">
              <h2 class="text-2xl font-bold text-neural-gold mb-2">Indexing Your Files...</h2>
              <p class="text-silver-neural/70 mb-6">
                This may take a few minutes depending on how many files you have.
              </p>

              <!-- Progress Bar -->
              <div class="bg-cortex-black rounded-lg p-6 border border-neural-gold/20 mb-4">
                <div class="mb-4">
                  <div class="flex justify-between text-sm mb-2">
                    <span class="text-silver-neural/70">Progress</span>
                    <span class="text-neural-gold font-bold">{indexingProgress.percentage.toFixed(1)}%</span>
                  </div>
                  <div class="bg-cortex-deep rounded-full h-4 overflow-hidden">
                    <div
                      class="bg-gradient-to-r from-neural-gold to-ember-gold h-4 rounded-full transition-all duration-300"
                      style:width={`${indexingProgress.percentage}%`}
                    ></div>
                  </div>
                </div>

                <div class="flex justify-between text-sm text-silver-neural/70 mb-3">
                  <span>{indexingProgress.current.toLocaleString()} / {indexingProgress.total.toLocaleString()} files</span>
                </div>

                {#if indexingProgress.currentFile}
                  <div class="text-xs text-silver-neural/50 truncate" title={indexingProgress.currentFile}>
                    Indexing: {indexingProgress.currentFile}
                  </div>
                {/if}
              </div>

              <!-- Spinner -->
              <div class="flex justify-center">
                <div class="animate-spin text-4xl">⏳</div>
              </div>
            </div>

          {:else if indexingComplete}
            <!-- Indexing Complete -->
            <div class="text-center">
              <div class="text-7xl mb-4">✅</div>
              <h2 class="text-3xl font-bold text-neural-gold mb-2">All Set!</h2>
              <p class="text-silver-neural/70 mb-6">
                Successfully indexed {indexingStats.filesIndexed.toLocaleString()} files
                in {formatDuration(indexingStats.duration)}
              </p>

              <div class="bg-cortex-black rounded-lg p-6 border border-neural-gold/20 mb-6">
                <h3 class="text-lg font-semibold text-neural-gold mb-4">You're ready to search!</h3>
                <div class="space-y-3 text-left text-sm text-silver-neural/80">
                  <div class="flex items-center gap-3">
                    <span class="text-2xl">⌨️</span>
                    <span>Press <kbd class="px-2 py-1 bg-slate-byte rounded border border-neural-gold/30 text-neural-gold">Cmd+K</kbd> to search anywhere</span>
                  </div>
                  <div class="flex items-center gap-3">
                    <span class="text-2xl">🔍</span>
                    <span>Type to find files by name or content</span>
                  </div>
                  <div class="flex items-center gap-3">
                    <span class="text-2xl">⚙️</span>
                    <span>Manage folders in Settings anytime</span>
                  </div>
                </div>
              </div>
            </div>

          {:else if indexingError}
            <!-- Error State -->
            <div class="text-center">
              <div class="text-6xl mb-4">⚠️</div>
              <h2 class="text-2xl font-bold text-red-400 mb-2">Indexing Failed</h2>
              <p class="text-silver-neural/70 mb-4">
                {indexingError}
              </p>
              <button
                onclick={startIndexing}
                class="px-6 py-3 bg-neural-gold text-cortex-black rounded-lg hover:bg-ember-gold transition-colors font-semibold"
              >
                Try Again
              </button>
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <div class="border-t border-neural-gold/20 p-6 bg-cortex-deep flex items-center justify-between">
      <div>
        {#if step === 1}
          <button
            onclick={skipOnboarding}
            class="text-sm text-silver-neural/60 hover:text-silver-neural transition-colors"
          >
            Skip setup
          </button>
        {/if}
      </div>

      <div class="flex gap-3">
        {#if step > 1 && step < 3}
          <button
            onclick={prevStep}
            class="px-6 py-3 bg-cortex-black text-silver-neural border border-neutral-gold/30 rounded-lg hover:bg-slate-byte transition-colors font-semibold"
          >
            Back
          </button>
        {/if}

        {#if step === 1}
          <button
            onclick={nextStep}
            class="px-6 py-3 bg-neural-gold text-cortex-black rounded-lg hover:bg-ember-gold transition-colors font-semibold"
          >
            Get Started →
          </button>
        {:else if step === 2}
          <button
            onclick={nextStep}
            disabled={directories.length === 0}
            class="px-6 py-3 bg-neural-gold text-cortex-black rounded-lg hover:bg-ember-gold disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-semibold"
          >
            Start Indexing →
          </button>
        {:else if step === 3 && indexingComplete}
          <button
            onclick={finishOnboarding}
            class="px-6 py-3 bg-neural-gold text-cortex-black rounded-lg hover:bg-ember-gold transition-colors font-semibold"
          >
            Start Using Cortex →
          </button>
        {/if}
      </div>
    </div>
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
    animation: spin 2s linear infinite;
  }
</style>
