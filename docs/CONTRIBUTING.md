# Contributing to Cortex

Thank you for your interest in contributing to Cortex! This document provides guidelines and instructions for contributing to the project.

**Table of Contents:**
1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [How to Contribute](#how-to-contribute)
4. [Development Workflow](#development-workflow)
5. [Coding Standards](#coding-standards)
6. [Testing Requirements](#testing-requirements)
7. [Documentation](#documentation)
8. [Community](#community)

---

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of:
- Experience level
- Gender identity and expression
- Sexual orientation
- Disability
- Personal appearance
- Body size
- Race or ethnicity
- Age
- Religion or lack thereof

### Our Standards

**Positive behaviors:**
- Using welcoming and inclusive language
- Being respectful of differing viewpoints
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

**Unacceptable behaviors:**
- Trolling, insulting/derogatory comments, and personal attacks
- Public or private harassment
- Publishing others' private information without permission
- Other conduct that could reasonably be considered inappropriate

### Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be reported to the project team. All complaints will be reviewed and investigated, and will result in a response that is deemed necessary and appropriate.

---

## Getting Started

### Prerequisites

Before contributing, ensure you have:
- [Rust 1.75+](https://rustup.rs) installed
- [Node.js 18+](https://nodejs.org) installed
- [Git](https://git-scm.com/) installed
- Familiarity with [Tauri](https://tauri.app) basics
- Read the [Developer Guide](DEVELOPER_GUIDE.md)

### Development Setup

1. **Fork the repository** on GitHub

2. **Clone your fork:**
   ```bash
   git clone https://github.com/YOUR_USERNAME/cortex.git
   cd cortex
   ```

3. **Add upstream remote:**
   ```bash
   git remote add upstream https://github.com/original/cortex.git
   ```

4. **Install dependencies:**
   ```bash
   npm install
   cd src-tauri && cargo fetch && cd ..
   ```

5. **Run development server:**
   ```bash
   npm run dev
   ```

6. **Run tests:**
   ```bash
   cd src-tauri && cargo test && cd ..
   npm test
   ```

---

## How to Contribute

### Types of Contributions

We welcome all types of contributions:

**🐛 Bug Reports**
- Found a bug? Open an issue with:
  - Clear description
  - Steps to reproduce
  - Expected vs actual behavior
  - System information (OS, Cortex version)
  - Relevant logs

**✨ Feature Requests**
- Have an idea? Open an issue with:
  - Use case description
  - Proposed solution
  - Alternative solutions considered
  - Willingness to implement

**📝 Documentation**
- Improve existing docs
- Add examples
- Fix typos
- Translate to other languages

**🔧 Code Contributions**
- Bug fixes
- New features
- Performance improvements
- Refactoring

**🧪 Testing**
- Write new tests
- Improve test coverage
- Report test failures

### What to Work On

**Good First Issues:**
Look for issues labeled `good-first-issue` - these are beginner-friendly tasks.

**Help Wanted:**
Issues labeled `help-wanted` are ready for contribution and don't require deep codebase knowledge.

**Current Priorities:**
Check [STATUS.md](../STATUS.md) for current development priorities and roadmap.

---

## Development Workflow

### 1. Create a Branch

```bash
# Sync with upstream
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/my-feature
# or
git checkout -b fix/bug-description
```

**Branch naming:**
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation
- `test/` - Tests only
- `refactor/` - Code refactoring
- `perf/` - Performance improvements

### 2. Make Changes

**Follow these guidelines:**
- One feature/fix per branch
- Keep changes focused and atomic
- Write clear, self-documenting code
- Add tests for new functionality
- Update documentation as needed

**Commit frequently:**
```bash
git add .
git commit -m "feat: add search date filter"
```

### 3. Write Tests

**Required:**
- Unit tests for new functions
- Integration tests for new commands
- Update existing tests if behavior changes

**Run tests:**
```bash
# Rust tests
cd src-tauri
cargo test --lib          # Unit tests
cargo test --tests        # Integration tests
cargo clippy -- -D warnings  # Linting

# Frontend tests
npm test
npm run lint
```

### 4. Update Documentation

If your change affects:
- **User behavior** → Update [USER_GUIDE.md](USER_GUIDE.md)
- **API** → Update [API_REFERENCE.md](API_REFERENCE.md)
- **Development** → Update [DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md)
- **Deployment** → Update [DEPLOYMENT.md](DEPLOYMENT.md)

### 5. Push Changes

```bash
git push origin feature/my-feature
```

### 6. Create Pull Request

1. Go to GitHub and create a pull request
2. Fill out the PR template completely
3. Link related issues (e.g., "Closes #123")
4. Request review from maintainers

**PR checklist:**
- [ ] Tests pass locally
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (for significant changes)
- [ ] Screenshots added (for UI changes)
- [ ] Breaking changes noted

### 7. Code Review

**Expect feedback:**
- Maintainers will review your code
- May request changes
- Discussion is encouraged
- Be patient and receptive

**Make requested changes:**
```bash
# Make changes
git add .
git commit -m "fix: address review feedback"
git push origin feature/my-feature
```

**PR updated automatically** - no need to open a new one.

### 8. Merge

Once approved:
- Maintainer will merge your PR
- Branch will be deleted
- You'll be credited as a contributor!

---

## Coding Standards

### Rust Code Style

**Follow Rust conventions:**
```bash
# Format code (required before commit)
cargo fmt

# Check lints (must pass)
cargo clippy -- -D warnings
```

**Guidelines:**

**1. Error Handling:**
```rust
// Good: Use Result with descriptive errors
pub fn search_files(query: &str) -> Result<Vec<File>, CortexError> {
    if query.is_empty() {
        return Err(CortexError::InvalidQuery {
            query: query.to_string(),
            reason: "Query cannot be empty".to_string(),
        });
    }
    // ...
}

// Bad: Using unwrap
pub fn search_files(query: &str) -> Vec<File> {
    let conn = get_connection().unwrap();  // ❌ Don't do this
    // ...
}
```

**2. Documentation:**
```rust
/// Search files using FTS5 full-text search.
///
/// # Arguments
/// * `conn` - Database connection
/// * `query` - Search query string
/// * `limit` - Maximum results to return
///
/// # Returns
/// Vector of search results ranked by relevance
///
/// # Errors
/// Returns `CortexError::InvalidQuery` if query is empty
///
/// # Examples
/// ```
/// let results = search_files_fts(&conn, "rust", 50)?;
/// ```
pub fn search_files_fts(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, CortexError> {
    // Implementation
}
```

**3. Naming:**
```rust
// Good: Clear, descriptive names
fn extract_text_content(path: &Path) -> Result<String, Error>
let file_count = count_files(&directory);

// Bad: Unclear abbreviations
fn ext_txt(p: &Path) -> Result<String, Error>
let fc = cnt_f(&dir);
```

**4. Comments:**
```rust
// Explain WHY, not WHAT
// Convert to lowercase for case-insensitive matching
let normalized_query = query.to_lowercase();

// Don't state the obvious
let count = 0;  // Initialize count to zero ❌
```

### TypeScript Code Style

**Use ESLint and Prettier:**
```bash
npm run lint     # Check linting
npm run format   # Format code
```

**Guidelines:**

**1. Type Everything:**
```typescript
// Good: Explicit types
interface SearchParams {
  query: string;
  limit: number;
  offset?: number;
}

async function search(params: SearchParams): Promise<SearchResults> {
  // ...
}

// Bad: Using 'any'
async function search(params: any): Promise<any> {  // ❌
  // ...
}
```

**2. Use Modern Syntax:**
```typescript
// Good: async/await
async function fetchData() {
  try {
    const result = await invoke('command');
    return result;
  } catch (error) {
    console.error('Failed:', error);
  }
}

// Bad: Promises with .then()
function fetchData() {  // ❌
  return invoke('command')
    .then(result => result)
    .catch(error => console.error(error));
}
```

**3. Destructuring:**
```typescript
// Good: Destructure where appropriate
const { results, total, query_time_ms } = await search(query);

// Acceptable: When you need the whole object
const searchResults = await search(query);
displayResults(searchResults);
```

### Svelte Component Style

**Follow Svelte conventions:**

**1. Script organization:**
```svelte
<script lang="ts">
  // 1. Imports
  import { invoke } from '@tauri-apps/api';
  import type { SearchResults } from '$lib/types';

  // 2. Props (interface)
  interface Props {
    initialQuery?: string;
  }
  let { initialQuery = '' }: Props = $props();

  // 3. State
  let query = $state(initialQuery);
  let results = $state<SearchResults | null>(null);

  // 4. Derived state
  let hasResults = $derived(results !== null && results.total > 0);

  // 5. Functions
  async function handleSearch() {
    results = await invoke('search_files', { query });
  }
</script>

<template>
  <!-- markup -->
</template>

<style>
  /* styles */
</style>
```

**2. Component naming:**
```
SearchBar.svelte         ✅ PascalCase for components
search-bar.svelte        ❌ Don't use kebab-case
searchBar.svelte         ❌ Don't use camelCase
```

### Commit Message Format

Use [Conventional Commits](https://www.conventionalcommits.org/):

**Format:**
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style (formatting, no logic change)
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `test`: Adding or fixing tests
- `chore`: Maintenance (deps, build, etc.)

**Examples:**
```
feat(search): add date range filtering
fix(indexing): handle permission errors gracefully
docs(api): update search command parameters
test(db): add FTS5 ranking tests
perf(extract): optimize PDF text extraction
refactor(state): simplify indexing progress tracking
chore(deps): update tauri to 2.0.1
```

**Breaking changes:**
```
feat(api)!: change search_files signature

BREAKING CHANGE: search_files now requires SearchFilters parameter
Migration: Update all calls to pass filters parameter (can be null)
```

---

## Testing Requirements

### What to Test

**All pull requests must include tests for:**
1. New features
2. Bug fixes
3. Behavior changes

**Minimum coverage:**
- New functions: 100%
- Modified functions: Maintain or improve coverage
- Overall: Don't decrease total coverage

### Writing Tests

**Rust Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_validates_empty_query() {
        let conn = Connection::open_in_memory().unwrap();
        let result = search_files_fts(&conn, "", 50);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }
}
```

**Rust Integration Tests:**
```rust
// src-tauri/tests/my_feature_test.rs

use cortex_lib::commands::search::search_files;
use cortex_lib::state::AppState;

#[tokio::test]
async fn test_search_with_filters() {
    let state = Arc::new(AppState::new().await.unwrap());
    // Setup test data...

    let results = search_files(
        "query".to_string(),
        Some(filters),
        None,
        None,
        tauri::State::from(state.as_ref()),
    )
    .await
    .unwrap();

    assert_eq!(results.total, 1);
}
```

**TypeScript Tests:**
```typescript
// src/lib/utils/format.test.ts

import { describe, it, expect } from 'vitest';
import { formatFileSize } from './format';

describe('formatFileSize', () => {
  it('formats bytes correctly', () => {
    expect(formatFileSize(1024)).toBe('1.00 KB');
  });

  it('handles zero', () => {
    expect(formatFileSize(0)).toBe('0 B');
  });
});
```

### Running Tests

**Before committing:**
```bash
# Run all Rust tests
cd src-tauri
cargo test --lib --tests

# Run all frontend tests
npm test

# Run linting
cargo clippy -- -D warnings
npm run lint
```

**Continuous Integration:**
Tests automatically run on all pull requests via GitHub Actions.

---

## Documentation

### Documentation Requirements

**All contributions should include documentation:**

**Code changes:**
- Inline code documentation (doc comments)
- Update affected markdown docs

**New features:**
- User guide update (how to use)
- API reference update (if command added)
- Developer guide update (if architecture changed)

**Bug fixes:**
- Document the fix in commit message
- Update troubleshooting section if relevant

### Documentation Style

**Be clear and concise:**
- Use simple language
- Include examples
- Add screenshots for UI changes
- Explain WHY, not just HOW

**Structure:**
- Use headings for organization
- Use code blocks for examples
- Use tables for comparisons
- Use lists for steps

**Example:**
```markdown
## Searching Files

To search indexed files, use the search bar at the top of the screen.

### Basic Search

1. Enter your search terms
2. Press Enter or click Search
3. Results appear below

**Example:**
```
rust programming
```

This searches for files containing "rust" OR "programming".
```

---

## Community

### Communication Channels

**GitHub:**
- Issues: Bug reports, feature requests
- Discussions: Questions, ideas, help
- Pull Requests: Code contributions

**Future channels:**
- Discord: Real-time chat
- Reddit: Community discussions
- Twitter: Announcements

### Getting Help

**Questions about:**
- **Using Cortex** → GitHub Discussions or Issues
- **Contributing** → This guide or GitHub Discussions
- **Bug or issue** → GitHub Issues
- **Feature request** → GitHub Issues (with `enhancement` label)

### Recognition

**Contributors are recognized in:**
- GitHub contributors page
- Release notes
- README.md (for significant contributions)
- Project website (future)

**Ways to be recognized:**
- Code contributions
- Documentation improvements
- Bug reports
- Feature ideas
- Community support

---

## Release Process

### How Releases Work

1. **Development** happens on `main` branch
2. **Release candidates** tagged as `v0.1.0-rc.1`
3. **Stable releases** tagged as `v0.1.0`
4. **Artifacts built** by GitHub Actions
5. **GitHub Release** created with changelog

### Version Numbers

Follow [Semantic Versioning](https://semver.org/):
- `0.1.0` → `0.1.1` - Bug fix
- `0.1.0` → `0.2.0` - New features
- `0.1.0` → `1.0.0` - Major milestone

### Contributing to Releases

**As a contributor, you don't need to:**
- Bump version numbers
- Create tags
- Build release artifacts

**Maintainers handle:**
- Version bumping
- Changelog compilation
- Release creation

---

## Additional Resources

### Documentation
- [User Guide](USER_GUIDE.md) - How to use Cortex
- [API Reference](API_REFERENCE.md) - Command reference
- [Developer Guide](DEVELOPER_GUIDE.md) - Development setup
- [Deployment Guide](DEPLOYMENT.md) - Building and releasing

### External Resources
- [Tauri Documentation](https://tauri.app/v2/guides/)
- [Svelte 5 Documentation](https://svelte-5-preview.vercel.app/docs)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Conventional Commits](https://www.conventionalcommits.org/)

### Tools
- [Rust Analyzer](https://rust-analyzer.github.io/) - IDE support
- [Prettier](https://prettier.io/) - Code formatting
- [ESLint](https://eslint.org/) - JavaScript linting

---

## Questions?

If you have questions not covered here:
1. Check existing documentation
2. Search GitHub Issues and Discussions
3. Open a new Discussion
4. Reach out to maintainers

**Thank you for contributing to Cortex! 🎉**

---

**Contributing Guide v0.1.0** | [Report Issue](https://github.com/yourusername/cortex/issues) | [Edit on GitHub](https://github.com/yourusername/cortex/edit/main/docs/CONTRIBUTING.md)
