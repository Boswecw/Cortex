# Cortex Documentation Index

Complete guide to all Cortex documentation, organized by purpose and audience.

---

## 📘 User Documentation

**Getting Started:**
- [README.md](README.md) - Project overview and quick start
- [SETUP.md](SETUP.md) - Detailed installation and setup
- [docs/USER_GUIDE.md](docs/USER_GUIDE.md) - Complete user guide

**Features:**
- **VS Code Claude Export** - Export indexed content for AI coding assistants
  - [VSCODE_EXPORT_COMPLETE.md](VSCODE_EXPORT_COMPLETE.md) - Feature overview
  - [CORTEX_CONTEXT_EXAMPLE.md](CORTEX_CONTEXT_EXAMPLE.md) - Example export format
  - [CORTEX_FEATURE_PROMPTS.md](CORTEX_FEATURE_PROMPTS.md) - Prompt templates

---

## 👨‍💻 Developer Documentation

**Development:**
- [docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md) - Architecture and development workflow
- [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) - How to contribute
- [docs/API_REFERENCE.md](docs/API_REFERENCE.md) - Tauri command reference
- [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) - Build and release process

**Testing:**
- [TESTING.md](TESTING.md) - Testing guide and coverage
- [tests/README.md](src-tauri/tests/README.md) - Test suite documentation

**Performance:**
- [PERFORMANCE.md](PERFORMANCE.md) - Benchmarks and optimization

---

## 📊 Project Status & Progress

**Current Status:**
- [STATUS.md](STATUS.md) - Overall project status
- [DUE_DILIGENCE_REPORT.md](DUE_DILIGENCE_REPORT.md) - Quality assessment (Dec 4, 2025)

**Implementation Reports:**
- [VSCODE_CLAUDE_EXPORT_IMPLEMENTATION_COMPLETE.md](VSCODE_CLAUDE_EXPORT_IMPLEMENTATION_COMPLETE.md) - Export feature implementation

**Phase Summaries:**
- [PHASE-1-SUMMARY.md](PHASE-1-SUMMARY.md) - Phase 1: Core Backend
- [PHASE-2-FINAL-SUMMARY.md](PHASE-2-FINAL-SUMMARY.md) - Phase 2: UI Implementation
- [PHASE-2-SESSION-SUMMARY.md](PHASE-2-SESSION-SUMMARY.md) - Phase 2 sessions
- [PHASE-2-SUMMARY.md](PHASE-2-SUMMARY.md) - Phase 2 overview

**Feature Implementation:**
- [AI-201-PROGRESS.md](AI-201-PROGRESS.md) - AI features progress
- [UI-IMPLEMENTATION-LOG.md](UI-IMPLEMENTATION-LOG.md) - UI development log

**Session Summaries (CX-series):**
- [CX-003-SUMMARY.md](CX-003-SUMMARY.md) - Session 003
- [CX-004-SUMMARY.md](CX-004-SUMMARY.md) - Session 004
- [CX-005-SUMMARY.md](CX-005-SUMMARY.md) - Session 005
- [CX-007-SUMMARY.md](CX-007-SUMMARY.md) - Session 007
- [CX-008-SUMMARY.md](CX-008-SUMMARY.md) - Session 008
- [CX-010-SUMMARY.md](CX-010-SUMMARY.md) - Session 010
- [CX-011-SUMMARY.md](CX-011-SUMMARY.md) - Session 011

---

## 🔧 Fix & Quality Assurance Sessions

**December 4, 2025 - Production Readiness Fixes:**
- [docs/sessions/FIXES_COMPLETED_2025-12-04.md](docs/sessions/FIXES_COMPLETED_2025-12-04.md)
  - **Issues Fixed:** Critical type mismatch, path traversal vulnerability, unused code
  - **Impact:** Export feature production-ready
  
- [docs/sessions/TEST_FIXES_SESSION_2025-12-04.md](docs/sessions/TEST_FIXES_SESSION_2025-12-04.md)
  - **Issues Fixed:** 23 test compilation errors
  - **Impact:** Test suite now compiles and runs (50 tests passing)

**Summary:** All critical, high, and medium priority issues resolved in 3.2 hours.

---

## 🗂️ Planning & Design Documents

**Implementation Plans:**
- [CORTEX_VSCODE_CLAUDE_IMPLEMENTATION_PLAN.md](CORTEX_VSCODE_CLAUDE_IMPLEMENTATION_PLAN.md) - Export feature planning
- [Cortex_VSCode_Context.md](Cortex_VSCode_Context.md) - Context integration design
- [cortex-claude-context.md](cortex-claude-context.md) - Claude integration specs

---

## 📁 Documentation Organization

```
cortex/
├── README.md                          # Main project overview
├── DOCUMENTATION.md                    # This file (index)
├── SETUP.md                           # Installation guide
├── STATUS.md                          # Current project status
├── TESTING.md                         # Testing guide
├── PERFORMANCE.md                     # Performance metrics
├── DUE_DILIGENCE_REPORT.md            # Quality assessment
│
├── docs/                              # Core documentation
│   ├── USER_GUIDE.md                  # User documentation
│   ├── DEVELOPER_GUIDE.md             # Developer documentation
│   ├── API_REFERENCE.md               # API documentation
│   ├── CONTRIBUTING.md                # Contribution guide
│   ├── DEPLOYMENT.md                  # Deployment guide
│   │
│   └── sessions/                      # Fix/implementation sessions
│       ├── FIXES_COMPLETED_2025-12-04.md
│       └── TEST_FIXES_SESSION_2025-12-04.md
│
├── Feature Documentation (Root)       # Feature-specific docs
│   ├── VSCODE_EXPORT_COMPLETE.md
│   ├── VSCODE_CLAUDE_EXPORT_IMPLEMENTATION_COMPLETE.md
│   ├── CORTEX_CONTEXT_EXAMPLE.md
│   └── CORTEX_FEATURE_PROMPTS.md
│
├── Progress Reports (Root)            # Historical progress
│   ├── PHASE-1-SUMMARY.md
│   ├── PHASE-2-FINAL-SUMMARY.md
│   ├── PHASE-2-SESSION-SUMMARY.md
│   ├── AI-201-PROGRESS.md
│   ├── UI-IMPLEMENTATION-LOG.md
│   └── CX-*-SUMMARY.md (7 files)
│
└── Planning (Root)                    # Design documents
    ├── CORTEX_VSCODE_CLAUDE_IMPLEMENTATION_PLAN.md
    ├── Cortex_VSCode_Context.md
    └── cortex-claude-context.md
```

---

## 🔍 Finding Documentation

### By Purpose

**I want to:**
- **Use Cortex** → Start with [README.md](README.md) and [docs/USER_GUIDE.md](docs/USER_GUIDE.md)
- **Develop Cortex** → See [docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md)
- **Export to VS Code Claude** → See [VSCODE_EXPORT_COMPLETE.md](VSCODE_EXPORT_COMPLETE.md)
- **Contribute** → See [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md)
- **Deploy** → See [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md)
- **Run tests** → See [TESTING.md](TESTING.md)
- **Check status** → See [STATUS.md](STATUS.md)

### By Role

**I am a:**
- **User** → User Documentation section above
- **Developer** → Developer Documentation section above
- **Contributor** → [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md)
- **Maintainer** → All sections relevant
- **Auditor** → Project Status & Fix Sessions sections

---

## 📊 Documentation Maintenance

**Last Updated:** December 4, 2025

**Documentation Standards:**
- All docs use GitHub-flavored Markdown
- Code examples use syntax highlighting
- Links use relative paths
- Headers use descriptive anchor-friendly text
- Session reports include timestamps and commit refs

**To Update This Index:**
1. Add new documentation files to appropriate section
2. Maintain alphabetical order within sections
3. Include brief description of content
4. Update "Last Updated" date
5. Commit with message: `docs: update documentation index`

---

## 🤝 Contributing to Documentation

Found an error? Documentation unclear? See [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for how to:
- Report documentation issues
- Propose documentation improvements
- Submit documentation PRs

**Good documentation:**
- Is accurate and up-to-date
- Uses clear, simple language
- Includes working code examples
- Has proper headers and structure
- Links to related documentation

---

*Cortex Documentation Index - Last updated December 4, 2025*
