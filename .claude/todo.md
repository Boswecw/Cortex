# Cortex Development TODO

**Last Updated:** 2025-11-29 22:15 UTC
**Current Phase:** Phase 1 - Desktop UI (In Progress)
**Phase 0 Status:** ✅ Complete (backend production-ready)
**Phase 1 Progress:** 4/12 tasks complete (33%)

---

## Phase 1: Desktop UI (Current)

### P0 - Critical Path

#### UI-101 [DONE] Main Application Layout
**Area:** Frontend/Layout
**Owner:** -
**Priority:** P0
**Status:** DONE
**Completed:** 2025-11-29
**Files:**
- `src/routes/+layout.svelte`
- `src/lib/components/Sidebar.svelte`
- `src/lib/components/ContentArea.svelte`
- `src/lib/components/PreviewPanel.svelte`

**Deps:** None
**Acceptance:**
- [ ] Three-column layout: Sidebar (250px) | Content (flex) | Preview (400px)
- [ ] Neural Gold dark theme applied
- [ ] Responsive resizing with drag handles
- [ ] 60 FPS rendering

**Notes:**
- Reference design tokens in Cortex_VSCode_Context.md:209-229
- Use Svelte 5 runes ($state, $derived)
- Apply Tailwind with custom theme colors

---

#### UI-102 [DONE] Global Search Bar (Cmd+K)
**Area:** Frontend/Search
**Owner:** -
**Priority:** P0
**Status:** DONE
**Completed:** 2025-11-29
**Files:**
- `src/lib/components/SearchBar.svelte`
- `src/lib/stores/search.ts`

**Deps:** UI-101
**Acceptance:**
- [ ] Global hotkey: Cmd/Ctrl+K opens search
- [ ] Real-time search as you type (debounced 150ms)
- [ ] Calls `search_files` Tauri command
- [ ] Shows loading state while searching
- [ ] Displays results with highlighting

**Notes:**
- Use `@tauri-apps/api` for invoking `search_files(query, filters, type, limit, offset)`
- Performance target: <100ms perceived latency
- Show recent searches when empty

---

#### UI-103 [DONE] File Grid/List View
**Area:** Frontend/Display
**Owner:** -
**Priority:** P0
**Status:** DONE
**Completed:** 2025-11-29
**Files:**
- `src/lib/components/FileGrid.svelte`
- `src/lib/components/FileList.svelte`
- `src/lib/components/FileCard.svelte`

**Deps:** UI-102
**Acceptance:**
- [ ] Toggle between grid and list views
- [ ] Virtual scrolling for 1000+ results
- [ ] File preview thumbnails (images, text preview)
- [ ] Click to select, Enter to open, Space for quick preview
- [ ] Keyboard navigation (arrow keys)

**Notes:**
- Use svelte-virtual for virtualized scrolling
- Cache thumbnails/previews in IndexedDB
- Smooth 120ms transitions

---

#### UI-104 [DONE] Settings Page
**Area:** Frontend/Settings
**Owner:** -
**Priority:** P0
**Status:** DONE
**Completed:** 2025-11-29
**Files:**
- `src/routes/settings/+page.svelte`
- Updated `src/lib/components/Sidebar.svelte` (navigation)
- `src-tauri/Cargo.toml` (dialog plugin)
- `src-tauri/src/main.rs` (dialog plugin init)
- `src-tauri/tauri.conf.json` (dialog permissions)

**Deps:** UI-101
**Acceptance:**
- [x] Indexed directories management (add/remove)
- [x] File type exclusions
- [x] Max file size setting
- [x] Index schedule (auto-index toggle)
- [x] Appearance settings (hidden files, symlinks)
- [x] Persist to localStorage
- [x] Tauri dialog integration for directory picker
- [x] Start/Stop indexing from settings
- [x] Success/error toasts

**Notes:**
- Settings persisted to localStorage (client-side)
- Tauri dialog plugin added for native file browser
- Auto-save on toggle changes
- Reset to defaults button included

---

#### UI-105 [READY] First-Run Onboarding
**Area:** Frontend/Onboarding
**Owner:** -
**Priority:** P0
**Status:** READY
**Files:**
- `src/lib/components/Onboarding.svelte`
- `src/lib/utils/firstRun.ts`

**Deps:** UI-104
**Acceptance:**
- [ ] Welcome screen on first launch
- [ ] Directory picker for initial indexing
- [ ] Quick settings (file types, exclusions)
- [ ] Start indexing button
- [ ] Progress indicator during first index
- [ ] Skip to app when complete

**Notes:**
- Detect first run via localStorage flag
- Call `start_indexing([paths])` command
- Listen to `indexing:progress` events
- Show estimated time remaining

---

#### UI-106 [READY] Indexing Status Display
**Area:** Frontend/Indexing
**Owner:** -
**Priority:** P1
**Status:** READY
**Files:**
- `src/lib/components/IndexStatus.svelte`

**Deps:** UI-101
**Acceptance:**
- [ ] Show current indexing progress (files/total, %)
- [ ] Real-time updates via Tauri events
- [ ] Pause/resume/cancel buttons
- [ ] Error display (per-file errors)
- [ ] Last indexed timestamp

**Notes:**
- Use `get_index_status()` command
- Listen to `indexing:progress`, `indexing:complete`, `indexing:error` events
- Update UI smoothly (max 60 FPS)

---

### P1 - Important

#### UI-107 [READY] Preview Panel
**Area:** Frontend/Preview
**Owner:** -
**Priority:** P1
**Status:** READY
**Files:**
- `src/lib/components/PreviewPanel.svelte`
- `src/lib/components/preview/TextPreview.svelte`
- `src/lib/components/preview/ImagePreview.svelte`
- `src/lib/components/preview/MarkdownPreview.svelte`

**Deps:** UI-103
**Acceptance:**
- [ ] Space bar for quick preview without opening
- [ ] Text files: syntax highlighting
- [ ] Images: full preview with zoom
- [ ] Markdown: rendered preview
- [ ] PDF: first page preview
- [ ] Metadata display (size, dates, path)

**Notes:**
- Use `get_file_detail(file_id)` command
- Lazy load content on demand
- Cache previews for navigation

---

#### UI-108 [READY] Advanced Search Filters
**Area:** Frontend/Search
**Owner:** -
**Priority:** P1
**Status:** READY
**Files:**
- `src/lib/components/SearchFilters.svelte`

**Deps:** UI-102
**Acceptance:**
- [ ] Filter by file type (txt, md, pdf, etc.)
- [ ] Filter by size range (min/max)
- [ ] Filter by date range (modified, created)
- [ ] Filter by tags (when implemented)
- [ ] Save filter presets

**Notes:**
- Maps to SearchFilters in search_files command
- Show filter chips above results
- Clear all filters button

---

#### UI-109 [READY] Keyboard Shortcuts
**Area:** Frontend/UX
**Owner:** -
**Priority:** P1
**Status:** READY
**Files:**
- `src/lib/utils/shortcuts.ts`
- `src/lib/components/ShortcutHelp.svelte`

**Deps:** UI-101
**Acceptance:**
- [ ] Cmd/Ctrl+K: Search
- [ ] Cmd/Ctrl+,: Settings
- [ ] Cmd/Ctrl+R: Refresh index
- [ ] Cmd/Ctrl+?: Show shortcuts help
- [ ] Arrow keys: Navigation
- [ ] Enter: Open file
- [ ] Space: Quick preview
- [ ] Esc: Close modals/preview

**Notes:**
- Use event listeners on document
- Prevent default for system shortcuts
- Show cheat sheet modal

---

### P2 - Nice to Have

#### UI-110 [BACKLOG] Empty States
**Area:** Frontend/UX
**Owner:** -
**Priority:** P2
**Status:** BACKLOG
**Files:**
- `src/lib/components/EmptyState.svelte`

**Acceptance:**
- [ ] No results state with suggestions
- [ ] No indexed files state with CTA
- [ ] Error state with retry button
- [ ] Loading skeletons

---

#### UI-111 [BACKLOG] Loading States & Animations
**Area:** Frontend/UX
**Owner:** -
**Priority:** P2
**Status:** BACKLOG
**Files:**
- `src/lib/components/LoadingSpinner.svelte`
- `src/lib/utils/transitions.ts`

**Acceptance:**
- [ ] Skeleton loaders for search results
- [ ] Smooth page transitions (120ms)
- [ ] Progress bars for indexing
- [ ] Fade-in animations for content

---

#### UI-112 [BACKLOG] Error Handling & Toasts
**Area:** Frontend/UX
**Owner:** -
**Priority:** P2
**Status:** BACKLOG
**Files:**
- `src/lib/components/Toast.svelte`
- `src/lib/stores/notifications.ts`

**Acceptance:**
- [ ] Toast notifications for actions
- [ ] Error messages with suggestions
- [ ] Success confirmations
- [ ] Auto-dismiss after 5s

---

## Phase 0: Foundation ✅ COMPLETE

All backend tasks complete. See STATUS.md for details.

---

## Phase 2: AI Features (Upcoming)

### AI-201 [BACKLOG] Local Embedding Generation
**Status:** BACKLOG
**Notes:** Implement ONNX Runtime with all-MiniLM-L6-v2 model

### AI-202 [BACKLOG] Vector Similarity Search
**Status:** BACKLOG
**Notes:** Implement vector index and search

### AI-203 [BACKLOG] Hybrid Search Fusion
**Status:** BACKLOG
**Notes:** RRF fusion of FTS + semantic results

---

## Completed Tasks

### CX-001 to CX-011 [DONE]
See STATUS.md for complete Phase 0 summary (9/11 tasks complete, 82%)

---

## Notes & Blockers

**Current Focus:** Phase 1 Desktop UI
**Next Milestone:** Working Tauri app with search UX
**Timeline:** Week 3 (per implementation plan)

**Technical Notes:**
- Svelte 5 with runes ($state, $derived)
- lucide-svelte removed (incompatible) - use plain SVG or find alternative
- Neural Gold theme tokens in Cortex_VSCode_Context.md
- All backend commands ready: search_files, get_file_detail, start_indexing, get_index_status, stop_indexing, get_search_stats

**Performance Targets:**
- Startup: <2s cold start
- Search: <100ms keyword
- UI: 60 FPS scrolling
- Memory: <150MB idle

---

**Format:**
- ID: [Area-Number]
- Status: BACKLOG → READY → DOING → REVIEW → BLOCKED → DONE
- Priority: P0 (critical), P1 (high), P2 (normal), P3 (nice-to-have)
