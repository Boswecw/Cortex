<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { fade } from 'svelte/transition';
  import { formatFileSize, getFileTypeInfo } from '$lib/types/api';
  import hljs from 'highlight.js';
  import { marked } from 'marked';
  import { onMount } from 'svelte';

  // Props
  let {
    fileId = $bindable(),
    filename = $bindable(),
    filepath = $bindable()
  }: {
    fileId: number | null;
    filename: string | null;
    filepath: string | null;
  } = $props();

  // State
  let fileDetail = $state<any>(null);
  let isLoading = $state(false);
  let error = $state<string | null>(null);
  let imageZoom = $state(1);
  let highlightedCode = $state<string>('');
  let renderedMarkdown = $state<string>('');

  // File type detection
  const CODE_EXTENSIONS = ['js', 'ts', 'jsx', 'tsx', 'py', 'rs', 'java', 'cpp', 'c', 'h', 'go', 'rb', 'php', 'swift', 'kt', 'cs', 'html', 'css', 'scss', 'less', 'sql', 'sh', 'bash', 'yaml', 'yml', 'toml', 'json', 'xml'];
  const IMAGE_EXTENSIONS = ['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'bmp', 'ico'];
  const PDF_EXTENSIONS = ['pdf'];
  const MARKDOWN_EXTENSIONS = ['md', 'markdown'];

  // Derived file type category
  let fileTypeCategory = $derived(() => {
    if (!fileDetail) return 'text';
    const ext = fileDetail.file_type.toLowerCase();
    if (CODE_EXTENSIONS.includes(ext)) return 'code';
    if (MARKDOWN_EXTENSIONS.includes(ext)) return 'markdown';
    if (IMAGE_EXTENSIONS.includes(ext)) return 'image';
    if (PDF_EXTENSIONS.includes(ext)) return 'pdf';
    return 'text';
  });

  // Watch fileId changes and fetch details
  $effect(() => {
    if (fileId !== null) {
      loadFileDetail(fileId);
    } else {
      fileDetail = null;
      error = null;
      highlightedCode = '';
      renderedMarkdown = '';
      imageZoom = 1;
    }
  });

  async function loadFileDetail(id: number) {
    isLoading = true;
    error = null;

    try {
      fileDetail = await invoke('get_file_detail', { fileId: id });

      // Process content based on file type
      if (fileDetail && fileDetail.content_preview) {
        const category = fileTypeCategory();

        if (category === 'code') {
          highlightCode(fileDetail.content_preview, fileDetail.file_type);
        } else if (category === 'markdown') {
          renderMarkdown(fileDetail.content_preview);
        }
      }
    } catch (err) {
      error = String(err);
      fileDetail = null;
    } finally {
      isLoading = false;
    }
  }

  function highlightCode(code: string, language: string) {
    try {
      const result = hljs.highlight(code, {
        language: getLanguageMapping(language),
        ignoreIllegals: true
      });
      highlightedCode = result.value;
    } catch (err) {
      // Fallback to auto-detection
      try {
        const result = hljs.highlightAuto(code);
        highlightedCode = result.value;
      } catch {
        highlightedCode = code;
      }
    }
  }

  function renderMarkdown(content: string) {
    try {
      renderedMarkdown = marked(content) as string;
    } catch (err) {
      renderedMarkdown = content;
    }
  }

  function getLanguageMapping(ext: string): string {
    const mapping: Record<string, string> = {
      'js': 'javascript',
      'ts': 'typescript',
      'jsx': 'javascript',
      'tsx': 'typescript',
      'py': 'python',
      'rs': 'rust',
      'rb': 'ruby',
      'cpp': 'cpp',
      'c': 'c',
      'h': 'c',
      'yml': 'yaml',
      'sh': 'bash',
    };
    return mapping[ext] || ext;
  }

  function formatDate(timestamp: string): string {
    return new Date(timestamp).toLocaleString();
  }

  function zoomIn() {
    imageZoom = Math.min(imageZoom + 0.25, 3);
  }

  function zoomOut() {
    imageZoom = Math.max(imageZoom - 0.25, 0.5);
  }

  function resetZoom() {
    imageZoom = 1;
  }

  function getFileIcon(category: string): string {
    switch (category) {
      case 'code': return '💻';
      case 'markdown': return '📝';
      case 'image': return '🖼️';
      case 'pdf': return '📕';
      default: return '📄';
    }
  }

  onMount(() => {
    // Import highlight.js theme
    import('highlight.js/styles/atom-one-dark.css');
  });
</script>

<aside class="w-[400px] bg-slate-byte border-l border-neural-gold/20 flex flex-col h-full overflow-hidden">
  {#if fileId === null}
    <!-- Empty State -->
    <div class="flex flex-col items-center justify-center h-full p-6 text-center">
      <span class="text-6xl mb-4">📄</span>
      <p class="text-lg text-silver-neural/70 mb-2">No file selected</p>
      <p class="text-sm text-silver-neural/50">Select a file from the results to see details</p>
      <div class="mt-6 p-4 bg-cortex-black rounded-lg border border-neural-gold/20 text-left w-full max-w-xs">
        <p class="text-xs text-silver-neural/60 mb-2">Keyboard Shortcuts:</p>
        <div class="space-y-1.5 text-xs">
          <div class="flex justify-between">
            <span class="text-silver-neural/70">Quick Preview</span>
            <kbd class="px-2 py-0.5 bg-slate-byte rounded border border-neural-gold/30 text-neural-gold">Space</kbd>
          </div>
          <div class="flex justify-between">
            <span class="text-silver-neural/70">Open File</span>
            <kbd class="px-2 py-0.5 bg-slate-byte rounded border border-neural-gold/30 text-neural-gold">Enter</kbd>
          </div>
          <div class="flex justify-between">
            <span class="text-silver-neural/70">Search</span>
            <kbd class="px-2 py-0.5 bg-slate-byte rounded border border-neural-gold/30 text-neural-gold">⌘K</kbd>
          </div>
        </div>
      </div>
    </div>
  {:else if isLoading}
    <!-- Loading State -->
    <div class="flex flex-col items-center justify-center h-full p-6">
      <span class="text-4xl animate-spin mb-4">⏳</span>
      <p class="text-sm text-silver-neural/70">Loading file details...</p>
    </div>
  {:else if error}
    <!-- Error State -->
    <div class="flex flex-col items-center justify-center h-full p-6 text-center">
      <span class="text-6xl mb-4">⚠️</span>
      <p class="text-lg text-red-400 mb-2">Failed to load file</p>
      <p class="text-sm text-silver-neural/50">{error}</p>
    </div>
  {:else if fileDetail}
    <!-- File Preview -->
    <div class="flex flex-col h-full" transition:fade={{ duration: 150 }}>
      <!-- Header -->
      <div class="p-4 border-b border-neural-gold/20 bg-cortex-deep">
        <div class="flex items-center gap-3 mb-2">
          <span class="text-3xl">{getFileIcon(fileTypeCategory())}</span>
          <div class="flex-1 min-w-0">
            <h2 class="text-lg font-semibold text-neural-gold truncate" title={filename || ''}>
              {filename}
            </h2>
            <p class="text-xs text-silver-neural/50 truncate mt-1" title={filepath || ''}>
              {filepath}
            </p>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <span class={`text-xs px-2 py-1 rounded ${getFileTypeInfo(fileDetail.file_type).color} bg-cortex-black/50`}>
            {getFileTypeInfo(fileDetail.file_type).label}
          </span>
        </div>
      </div>

      <!-- Metadata -->
      <div class="p-4 border-b border-neural-gold/20 bg-cortex-black/30">
        <div class="grid grid-cols-2 gap-3 text-sm">
          <div class="flex items-start gap-2">
            <span class="text-lg">💾</span>
            <div class="flex-1 min-w-0">
              <p class="text-xs text-silver-neural/60 mb-0.5">Size</p>
              <p class="text-silver-neural font-medium">{formatFileSize(fileDetail.size)}</p>
            </div>
          </div>
          <div class="flex items-start gap-2">
            <span class="text-lg">📋</span>
            <div class="flex-1 min-w-0">
              <p class="text-xs text-silver-neural/60 mb-0.5">Type</p>
              <p class="text-silver-neural font-medium">{fileDetail.file_type}</p>
            </div>
          </div>
          <div class="flex items-start gap-2">
            <span class="text-lg">✏️</span>
            <div class="flex-1 min-w-0">
              <p class="text-xs text-silver-neural/60 mb-0.5">Modified</p>
              <p class="text-silver-neural font-medium text-xs">{formatDate(fileDetail.modified_at)}</p>
            </div>
          </div>
          <div class="flex items-start gap-2">
            <span class="text-lg">📅</span>
            <div class="flex-1 min-w-0">
              <p class="text-xs text-silver-neural/60 mb-0.5">Created</p>
              <p class="text-silver-neural font-medium text-xs">{formatDate(fileDetail.created_at)}</p>
            </div>
          </div>
        </div>

        {#if fileDetail.word_count}
          <div class="flex items-start gap-2 mt-3">
            <span class="text-lg">📊</span>
            <div class="flex-1 min-w-0">
              <p class="text-xs text-silver-neural/60 mb-0.5">Word Count</p>
              <p class="text-silver-neural font-medium">{fileDetail.word_count.toLocaleString()} words</p>
            </div>
          </div>
        {/if}

        {#if fileDetail.summary}
          <div class="flex items-start gap-2 mt-3">
            <span class="text-lg">🧠</span>
            <div class="flex-1 min-w-0">
              <p class="text-xs text-silver-neural/60 mb-1">AI Summary</p>
              <p class="text-sm text-silver-neural/80 leading-relaxed">{fileDetail.summary}</p>
            </div>
          </div>
        {/if}
      </div>

      <!-- Content Preview -->
      {#if fileDetail.content_preview}
        <div class="flex-1 overflow-y-auto p-4">
          <div class="mb-3 flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="text-lg">👁️</span>
              <p class="text-xs font-semibold text-neural-gold">Preview</p>
            </div>
            <span class="text-xs text-silver-neural/50">{fileDetail.content_preview.length} chars</span>
          </div>

          {#if fileTypeCategory() === 'code'}
            <!-- Code Preview with Syntax Highlighting -->
            <div class="bg-cortex-black rounded-lg overflow-hidden border border-neural-gold/20">
              <div class="bg-cortex-deep px-4 py-2 border-b border-neural-gold/10">
                <span class="text-xs text-neural-gold font-mono">{fileDetail.file_type}</span>
              </div>
              <div class="overflow-x-auto">
                <pre class="p-4 text-sm"><code class="hljs language-{getLanguageMapping(fileDetail.file_type)}">{@html highlightedCode}</code></pre>
              </div>
            </div>

          {:else if fileTypeCategory() === 'markdown'}
            <!-- Markdown Rendered Preview -->
            <div class="bg-cortex-black rounded-lg p-6 border border-neural-gold/20 prose prose-invert prose-sm max-w-none">
              <div class="markdown-content">
                {@html renderedMarkdown}
              </div>
            </div>

          {:else if fileTypeCategory() === 'image'}
            <!-- Image Preview with Zoom -->
            <div class="bg-cortex-black rounded-lg p-4 border border-neural-gold/20">
              <div class="flex items-center justify-center gap-2 mb-3">
                <button
                  onclick={zoomOut}
                  class="px-3 py-1 bg-neural-gold/10 text-neural-gold rounded hover:bg-neural-gold/20 transition-colors text-sm"
                  disabled={imageZoom <= 0.5}
                >
                  🔍−
                </button>
                <span class="text-xs text-silver-neural/70">{Math.round(imageZoom * 100)}%</span>
                <button
                  onclick={zoomIn}
                  class="px-3 py-1 bg-neural-gold/10 text-neural-gold rounded hover:bg-neural-gold/20 transition-colors text-sm"
                  disabled={imageZoom >= 3}
                >
                  🔍+
                </button>
                <button
                  onclick={resetZoom}
                  class="px-3 py-1 bg-neural-gold/10 text-neural-gold rounded hover:bg-neural-gold/20 transition-colors text-sm"
                >
                  Reset
                </button>
              </div>
              <div class="overflow-auto max-h-[500px] flex items-center justify-center">
                <img
                  src="asset://localhost/{encodeURIComponent(fileDetail.path)}"
                  alt={filename || 'Preview'}
                  class="transition-transform duration-200"
                  style="transform: scale({imageZoom}); transform-origin: center;"
                  onerror={() => error = 'Failed to load image'}
                />
              </div>
            </div>

          {:else if fileTypeCategory() === 'pdf'}
            <!-- PDF Preview Placeholder -->
            <div class="bg-cortex-black rounded-lg p-8 border border-neural-gold/20 text-center">
              <span class="text-6xl mb-4 block">📕</span>
              <p class="text-sm text-silver-neural/70 mb-3">PDF Preview</p>
              <p class="text-xs text-silver-neural/50">PDF rendering coming soon...</p>
              <p class="text-xs text-silver-neural/50 mt-2">File: {filename}</p>
            </div>

          {:else}
            <!-- Plain Text Preview -->
            <div class="bg-cortex-black rounded-lg p-4 border border-neural-gold/20">
              <pre class="text-xs text-silver-neural/90 whitespace-pre-wrap break-words font-mono leading-relaxed">{fileDetail.content_preview}</pre>
            </div>
          {/if}
        </div>
      {:else}
        <div class="flex-1 flex items-center justify-center p-6 text-center">
          <div>
            <span class="text-4xl mb-3 block">📝</span>
            <p class="text-sm text-silver-neural/60">No content preview available</p>
          </div>
        </div>
      {/if}

      <!-- Actions -->
      <div class="p-4 border-t border-neural-gold/20 bg-cortex-deep">
        <button
          class="w-full px-4 py-2 bg-neural-gold text-cortex-black font-semibold rounded-lg hover:bg-ember-gold transition-colors text-sm"
        >
          Open in Default App
        </button>
      </div>
    </div>
  {/if}
</aside>

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

  /* Markdown content styling */
  :global(.markdown-content h1) {
    @apply text-2xl font-bold text-neural-gold mb-4;
  }

  :global(.markdown-content h2) {
    @apply text-xl font-bold text-neural-gold mb-3;
  }

  :global(.markdown-content h3) {
    @apply text-lg font-semibold text-neural-gold mb-2;
  }

  :global(.markdown-content p) {
    @apply text-silver-neural/90 mb-3 leading-relaxed;
  }

  :global(.markdown-content ul, .markdown-content ol) {
    @apply ml-6 mb-3 text-silver-neural/90;
  }

  :global(.markdown-content li) {
    @apply mb-1;
  }

  :global(.markdown-content code) {
    @apply bg-cortex-deep px-1.5 py-0.5 rounded text-neural-gold font-mono text-xs;
  }

  :global(.markdown-content pre) {
    @apply bg-cortex-deep p-4 rounded-lg overflow-x-auto mb-3;
  }

  :global(.markdown-content pre code) {
    @apply bg-transparent p-0;
  }

  :global(.markdown-content a) {
    @apply text-neural-gold hover:text-ember-gold underline;
  }

  :global(.markdown-content blockquote) {
    @apply border-l-4 border-neural-gold/30 pl-4 italic text-silver-neural/70 mb-3;
  }

  :global(.markdown-content hr) {
    @apply border-neural-gold/20 my-4;
  }

  :global(.markdown-content table) {
    @apply w-full mb-3 text-sm;
  }

  :global(.markdown-content th) {
    @apply bg-cortex-deep text-neural-gold font-semibold p-2 border border-neural-gold/20;
  }

  :global(.markdown-content td) {
    @apply p-2 border border-neural-gold/20 text-silver-neural/80;
  }

  /* Code highlighting adjustments */
  :global(.hljs) {
    @apply bg-transparent;
  }
</style>
