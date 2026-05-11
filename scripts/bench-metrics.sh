
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BENCH_DIR="$ROOT_DIR/ax-codec-core"

echo "=================================================="
echo " ax_codec Comprehensive Benchmark Metrics"
echo "=================================================="
echo ""

echo "--- Compile Time ---"
echo "Clean build (debug):"
cd "$ROOT_DIR"
cargo clean
time cargo build --all-features --workspace 2>&1 | tail -1 || true

echo ""
echo "Clean build (release):"
cargo clean
time cargo build --all-features --workspace --release 2>&1 | tail -1 || true

echo ""

echo "--- Binary Size (release, stripped) ---"
cd "$ROOT_DIR"
cargo build --all-features --workspace --release

for crate in ax-codec-core ax-codec-derive ax-codec-bytes ax-codec-net; do
    if [ -f "target/release/lib${crate}.rlib" ]; then
        ls -lh "target/release/lib${crate}.rlib" | awk '{print "  " $9 ": " $5}'
    fi
done

echo ""
echo "Binary size with cargo-bloat (ax-codec-core example):"
if command -v cargo-bloat &>/dev/null; then
    cargo bloat --release -p ax-codec-core --example dhat_profile 2>&1 | head -20 || true
else
    echo "  (cargo-bloat not installed: cargo install cargo-bloat)"
fi

echo ""

echo "--- Peak RSS (example run) ---"
cd "$BENCH_DIR"

if [ "$(uname)" = "Darwin" ]; then
    echo "Running dhat_profile example (macOS /usr/bin/time -l)..."
    /usr/bin/time -l cargo run --example dhat_profile --features std 2>&1 | grep "maximum resident" || true
else
    echo "Running dhat_profile example (Linux /usr/bin/time -v)..."
    /usr/bin/time -v cargo run --example dhat_profile --features std 2>&1 | grep "Maximum resident" || true
fi

echo ""

echo "--- DHAT Heap Profile ---"
cd "$BENCH_DIR"
echo "Running with DHAT profiler..."
DHAT_HEAP_PROFILE=1 cargo run --features "std dhat-heap" --example dhat_profile 2>&1 | tail -5 || true
if [ -f dhat-heap.json ]; then
    echo "DHAT output: dhat-heap.json"
    echo "View with: dhat --compare dhat-heap.json"
else
    echo "  (DHAT requires: dhat-heap feature enabled)"
fi

echo ""

echo "--- Heaptrack ---"
if [ "$(uname)" = "Darwin" ]; then
    echo "  heaptrack is Linux-only. On macOS use:"
    echo "    - dhat (cargo run --features dhat-heap --example dhat_profile)"
    echo "    - Instruments (instruments -t Time\ Profiler cargo run ...)"
    echo "    - jemalloc profiling (see examples/jemalloc_profile.rs)"
    echo "    - leaks tool: leaks --atExit -- cargo run --example dhat_profile"
elif command -v heaptrack &>/dev/null; then
    cd "$BENCH_DIR"
    heaptrack cargo run --example dhat_profile --features std 2>&1 | tail -5 || true
    echo "View with: heaptrack_gui heaptrack.*.gz"
else
    echo "  (heaptrack not installed)"
    echo "    Linux: sudo apt install heaptrack"
fi

echo ""

echo "--- Payload Size Comparison ---"
cd "$BENCH_DIR"
cargo run --example payload_sizes --features std 2>&1 | grep -v "Compiling\|Finished\|Running" || true

echo ""

echo "--- Decode Latency (quick) ---"
# Use timeout to avoid hanging if criterion gets stuck
if command -v timeout &>/dev/null; then
    timeout 60 cargo bench --all-features --bench comprehensive_bench -- --quick 2>&1 | grep "time:" | head -20 || true
elif command -v gtimeout &>/dev/null; then
    gtimeout 60 cargo bench --all-features --bench comprehensive_bench -- --quick 2>&1 | grep "time:" | head -20 || true
else
    echo "  (timeout command not found, skipping decode latency to avoid hanging)"
    echo "    Install with: brew install coreutils"
fi

echo ""
echo "=================================================="
echo " Done. Full report in target/criterion/"
echo "=================================================="
