#!/bin/bash
# Performance test script for Cortex export functionality
# Tests export on the Cortex codebase itself

set -e

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║         Cortex Export Performance Test                   ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CORTEX_ROOT="$PROJECT_ROOT/src-tauri"

echo "📁 Project root: $PROJECT_ROOT"
echo ""

# Count files in project
echo "📊 Analyzing Cortex codebase..."
RUST_FILES=$(find "$CORTEX_ROOT/src" -name "*.rs" 2>/dev/null | wc -l)
SVELTE_FILES=$(find "$PROJECT_ROOT/src" -name "*.svelte" 2>/dev/null | wc -l)
TS_FILES=$(find "$PROJECT_ROOT/src" -name "*.ts" 2>/dev/null | wc -l)
MD_FILES=$(find "$PROJECT_ROOT" -name "*.md" -not -path "*/node_modules/*" -not -path "*/.svelte-kit/*" 2>/dev/null | wc -l)
TOTAL_FILES=$((RUST_FILES + SVELTE_FILES + TS_FILES + MD_FILES))

echo "  Rust files:    $RUST_FILES"
echo "  Svelte files:  $SVELTE_FILES"
echo "  TypeScript:    $TS_FILES"
echo "  Markdown:      $MD_FILES"
echo "  ──────────────────────"
echo "  Total files:   $TOTAL_FILES"
echo ""

# Calculate total size
TOTAL_SIZE=$(du -sh "$PROJECT_ROOT/src" "$CORTEX_ROOT/src" 2>/dev/null | awk '{sum+=$1} END {print sum}')
echo "  Total size:    ${TOTAL_SIZE}MB (approx)"
echo ""

# Check if cortex is built
if [ ! -f "$CORTEX_ROOT/target/release/cortex" ]; then
    echo "${YELLOW}⚠️  Release binary not found. Building...${NC}"
    cd "$CORTEX_ROOT"
    cargo build --release
    echo ""
fi

# Create test database
echo "🗄️  Setting up test database..."
TEST_DB_DIR="/tmp/cortex-perf-test-$$"
mkdir -p "$TEST_DB_DIR"
TEST_DB="$TEST_DB_DIR/cortex.db"

export CORTEX_DB_PATH="$TEST_DB"

echo "  Database: $TEST_DB"
echo ""

# Start timing
START_TIME=$(date +%s.%N)

echo "📚 Indexing files (this may take a moment)..."
# Note: This would require running the actual Cortex app
# For now, we'll document the manual steps

cat << 'EOF'

╔════════════════════════════════════════════════════════════════╗
║                    MANUAL PERFORMANCE TEST                     ║
╚════════════════════════════════════════════════════════════════╝

To complete the performance test manually:

1. Start the Cortex application:
   cd src-tauri
   cargo run --release

2. In the application:
   a. Index the Cortex project directory itself
   b. Wait for indexing to complete
   c. Go to Export section
   d. Choose "VS Code Claude Export"
   e. Note the start time
   f. Click "Export"
   g. Note the end time when export completes

3. Check the output:
   - Export location: .cortex-export/
   - Verify CONTEXT.md was created
   - Check file size and content

4. Record metrics:
   - Total files indexed: _____
   - Indexing time: _____ seconds
   - Export time: _____ seconds
   - Output size: _____ MB
   - Files/second: (total files ÷ export time)

═══════════════════════════════════════════════════════════════

AUTOMATED TEST (using CLI - if available):

You can also test export performance programmatically by:

1. Building the project:
   cargo build --release

2. The export functionality can be tested via:
   - Tauri commands from the UI
   - Integration tests (to be added)
   - Performance benchmark binary (in development)

═══════════════════════════════════════════════════════════════

PERFORMANCE BENCHMARKS:

Target performance goals:
✅ 100 files in < 5-10 seconds
✅ 1000 files in < 30-60 seconds
✅ Memory usage < 500MB

Current Cortex codebase (~$TOTAL_FILES files):
Expected export time: < 5 seconds (estimate)

═══════════════════════════════════════════════════════════════

CLEANUP:

To remove test database:
   rm -rf $TEST_DB_DIR

EOF

# Cleanup function
cleanup() {
    echo ""
    echo "🧹 Cleanup..."
    if [ -d "$TEST_DB_DIR" ]; then
        rm -rf "$TEST_DB_DIR"
        echo "  ✓ Removed test database"
    fi
}

# Register cleanup
trap cleanup EXIT

echo ""
echo "${GREEN}✓ Performance test script ready${NC}"
echo ""
echo "See manual steps above to complete the performance test."
echo ""
