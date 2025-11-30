# Cortex

**Fast, offline-first file indexing and search** built with Rust/Tauri + SvelteKit.

<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.0--alpha-blue" alt="Version">
  <img src="https://img.shields.io/badge/rust-1.75+-orange" alt="Rust">
  <img src="https://img.shields.io/badge/tauri-2.0-24C8DB" alt="Tauri">
  <img src="https://img.shields.io/badge/svelte-5-FF3E00" alt="Svelte">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

---

## 🎯 What is Cortex?

Cortex is a **desktop application** that indexes your local files and provides **lightning-fast full-text search**. Think of it as your personal search engine for documents, code, and notes—completely offline and privacy-focused.

### Key Features

- ✅ **Fast Indexing** - 50+ files/second with support for txt, md, pdf, docx, and source code
- ✅ **Instant Search** - Sub-100ms search queries with FTS5 full-text search
- ✅ **Advanced Filters** - Filter by file type, size, and date range
- ✅ **Real-time Progress** - Live indexing status with progress tracking
- ✅ **Offline First** - All data stored locally in SQLite (no cloud, no tracking)
- ✅ **Highlighted Results** - Search snippets with match highlighting
- ✅ **Cross-Platform** - Linux, macOS, Windows support

### Use Cases

- **Developers:** Search across all your projects and code
- **Writers:** Find content in documents and notes
- **Researchers:** Index research papers and references
- **Students:** Search lecture notes and study materials
- **Anyone:** Quickly find files on your computer

---

## 🚀 Quick Start

### Installation

**Pre-built Binaries (Coming Soon):**
- [Linux (.deb, .AppImage)](https://github.com/yourusername/cortex/releases)
- [macOS (.dmg)](https://github.com/yourusername/cortex/releases)
- [Windows (.msi)](https://github.com/yourusername/cortex/releases)

**Build from Source:**

```bash
# Prerequisites: Rust 1.75+, Node.js 18+

# Clone repository
git clone https://github.com/yourusername/cortex.git
cd cortex

# Install dependencies
npm install

# Build and run
npm run tauri dev
```

See [SETUP.md](SETUP.md) for detailed setup instructions.

### First Steps

1. **Launch Cortex** from your applications menu
2. **Index directories** - Click "Start Indexing" and select folders
3. **Search files** - Type a query and see instant results
4. **Filter results** - Use advanced filters for precise searches

See the [User Guide](docs/USER_GUIDE.md) for complete instructions.

---

## 📸 Screenshots

> **Note:** Screenshots coming soon! Phase 0 focuses on backend functionality.

**Planned UI features:**
- Dark theme with Neural Gold accent colors
- Clean, modern search interface
- Real-time indexing progress visualization
- File preview panel with syntax highlighting

---

## 🏗️ Architecture

### Technology Stack

**Backend (Rust):**
- **Tauri 2.0** - Desktop application framework
- **SQLite + FTS5** - Embedded database with full-text search
- **Tokio** - Async runtime for concurrent operations
- **pdf-extract**, **docx-rs** - Content extraction
- **walkdir**, **notify** - File scanning and watching

**Frontend (SvelteKit):**
- **Svelte 5** - Reactive UI framework with runes
- **TypeScript** - Type-safe JavaScript
- **Tailwind CSS** - Utility-first styling
- **Vite** - Fast development and building

### System Overview

```
┌─────────────────────────────────────┐
│     Frontend (SvelteKit)             │
│  • Search Interface                  │
│  • Indexing Dashboard                │
│  • File Preview                      │
└──────────────┬──────────────────────┘
               │ Tauri IPC
┌──────────────┴──────────────────────┐
│     Backend (Rust)                   │
│  • Tauri Commands (API)              │
│  • Indexing Pipeline                 │
│  • Content Extractors                │
│  • File Scanner                      │
└──────────────┬──────────────────────┘
               │
┌──────────────┴──────────────────────┐
│   SQLite Database + FTS5             │
│  • File metadata                     │
│  • Full-text index                   │
│  • Search rankings                   │
└─────────────────────────────────────┘
```

See [docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md) for architecture details.

---

## ⚡ Performance

### Targets (Phase 0)

| Metric | Target | Status |
|--------|--------|--------|
| **Indexing Speed** | >50 files/sec | ✅ Achieved |
| **Search Latency** | <100ms | ✅ Achieved |
| **10K Files Index** | <5 minutes | ✅ Achieved |
| **Memory (Idle)** | <150MB | ⏳ To measure |
| **Startup Time** | <2 seconds | ⏳ To measure |

### Benchmarks

Run benchmarks yourself:

```bash
cd src-tauri

# Indexing benchmark (100, 500, 1000 files)
cargo run --release --bin indexing_benchmark

# Search benchmark (various query types)
cargo run --release --bin search_benchmark

# Load test (1K, 2.5K, 5K files)
cargo run --release --bin load_test
```

See [PERFORMANCE.md](PERFORMANCE.md) for detailed performance analysis.

---

## 📚 Documentation

**User Documentation:**
- [User Guide](docs/USER_GUIDE.md) - How to use Cortex
- [FAQ](docs/USER_GUIDE.md#faq) - Frequently asked questions
- [Troubleshooting](docs/USER_GUIDE.md#troubleshooting) - Common issues

**Developer Documentation:**
- [Developer Guide](docs/DEVELOPER_GUIDE.md) - Development setup and workflow
- [API Reference](docs/API_REFERENCE.md) - Tauri command reference
- [Contributing Guide](docs/CONTRIBUTING.md) - How to contribute
- [Deployment Guide](docs/DEPLOYMENT.md) - Building and releasing

**Project Status:**
- [STATUS.md](STATUS.md) - Current development status (73% Phase 0 complete)
- [TESTING.md](TESTING.md) - Testing guide and coverage
- [PERFORMANCE.md](PERFORMANCE.md) - Performance targets and optimization

---

## 🧪 Testing

### Running Tests

**Backend tests:**
```bash
cd src-tauri

# Unit tests
cargo test --lib

# Integration tests
cargo test --tests

# All tests
cargo test

# With coverage
cargo tarpaulin
```

**Frontend tests:**
```bash
npm test
npm run lint
```

**Test Coverage:**
- **Rust:** 86+ tests (38 unit + 33 integration + 15 benchmarks)
- **Coverage:** >95% for database layer
- **CI:** Automated testing on all PRs

See [TESTING.md](TESTING.md) for comprehensive testing guide.

---

## 📈 Project Status

### Phase 0 Progress: **73%** (8/11 tasks complete)

**✅ Completed:**
- [x] Project setup and dependencies
- [x] Database schema with FTS5
- [x] Database layer with tests (>95% coverage)
- [x] File scanner with priority queue
- [x] Content extractors (txt, md, pdf, docx)
- [x] Tauri commands (indexing)
- [x] Tauri commands (search)
- [x] Testing & performance benchmarks

**🚧 In Progress:**
- [ ] CX-011: Documentation (in progress)

**📋 Upcoming:**
- [ ] Frontend UI implementation
- [ ] File watching for auto-reindex
- [ ] Incremental indexing

See [STATUS.md](STATUS.md) for detailed progress and roadmap.

---

## 🤝 Contributing

We welcome contributions! Here's how to get started:

1. **Check out** [good first issues](https://github.com/yourusername/cortex/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)
2. **Read** [CONTRIBUTING.md](docs/CONTRIBUTING.md)
3. **Fork** the repository
4. **Create** a feature branch
5. **Submit** a pull request

### Ways to Contribute

- 🐛 Report bugs
- ✨ Suggest features
- 📝 Improve documentation
- 🔧 Submit code fixes/features
- 🧪 Write tests
- 💬 Help answer questions

### Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/cortex.git
cd cortex

# Install dependencies
npm install

# Run development server
npm run dev

# Run tests
cd src-tauri && cargo test
```

See [docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md) for complete development guide.

---

## 🗺️ Roadmap

### Phase 0: Foundation (73% Complete)
- [x] Core indexing pipeline
- [x] Full-text search
- [x] Tauri commands
- [ ] Complete documentation

### Phase 1: Enhancements (Planned)
- [ ] Frontend UI
- [ ] File watching / auto-reindex
- [ ] Incremental indexing
- [ ] Query result caching
- [ ] Parallel extraction

### Phase 2: Advanced Features (Future)
- [ ] Semantic search
- [ ] Tag system
- [ ] Saved searches
- [ ] Export functionality
- [ ] Browser extension

### Phase 3: Intelligence (Research)
- [ ] AI-powered summarization
- [ ] Automatic categorization
- [ ] Related file suggestions
- [ ] Natural language queries

---

## 🔒 Privacy & Security

**Cortex is privacy-first:**
- ✅ **100% offline** - All processing happens locally
- ✅ **No telemetry** - We don't collect any data
- ✅ **No cloud** - Your files never leave your machine
- ✅ **Open source** - Audit the code yourself

**Data storage:**
- Database: `~/.cortex/db.sqlite`
- Configuration: `~/.cortex/config.toml` (future)
- All data stays on your computer

---

## 📊 Statistics

**Project Stats:**
- **Lines of Code:** ~15,000+ (Rust + TypeScript)
- **Rust Modules:** 12 core modules
- **Tauri Commands:** 6 exposed to frontend
- **Tests:** 86+ (unit + integration + benchmarks)
- **Documentation:** 5,000+ lines
- **Supported File Types:** 15+

**Performance Stats (typical hardware):**
- Indexing: ~50-100 files/second
- Search: <10ms average latency
- Database size: ~30-50% of file sizes
- Memory usage: <200MB typical

---

## 🙏 Acknowledgments

**Built with amazing open source projects:**
- [Tauri](https://tauri.app) - Desktop application framework
- [Svelte](https://svelte.dev) - UI framework
- [SQLite](https://sqlite.org) - Embedded database
- [Rust](https://rust-lang.org) - Systems programming language
- [Tokio](https://tokio.rs) - Async runtime

**Inspired by:**
- [Recoll](https://www.lesbonscomptes.com/recoll/) - Desktop search
- [ripgrep](https://github.com/BurntSushi/ripgrep) - Fast search
- [fd](https://github.com/sharkdp/fd) - File finding
- [fzf](https://github.com/junegunn/fzf) - Fuzzy finder

---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

**TL;DR:**
- ✅ Commercial use
- ✅ Modification
- ✅ Distribution
- ✅ Private use

---

## 🔗 Links

- **Documentation:** [docs/](docs/)
- **Issues:** [GitHub Issues](https://github.com/yourusername/cortex/issues)
- **Discussions:** [GitHub Discussions](https://github.com/yourusername/cortex/discussions)
- **Releases:** [GitHub Releases](https://github.com/yourusername/cortex/releases)
- **Changelog:** [CHANGELOG.md](CHANGELOG.md) (future)

---

## 💬 Contact & Support

**Questions or problems?**
- 📖 Check the [User Guide](docs/USER_GUIDE.md)
- 💬 Ask in [GitHub Discussions](https://github.com/yourusername/cortex/discussions)
- 🐛 Report bugs in [GitHub Issues](https://github.com/yourusername/cortex/issues)
- 📧 Email: [support@cortex.app](mailto:support@cortex.app) (future)

**Stay updated:**
- ⭐ Star this repository
- 👀 Watch for releases
- 🐦 Follow on Twitter (future)

---

<p align="center">
  <sub>Built with ❤️ by the Cortex team</sub>
</p>

<p align="center">
  <a href="#cortex">Back to top</a>
</p>
