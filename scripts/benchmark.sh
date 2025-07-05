#!/usr/bin/env bash
# this_file: scripts/benchmark.sh

set -euo pipefail
cd "$(dirname "$0")"/..

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEMP_DIR=$(mktemp -d)
SVG_DIR="${1:-testdata}"
ITERATIONS="${2:-3}"
MIN_FILES="${3:-5}"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
REPORT_DIR="docs/benchmarks"
REPORT_FILE="$REPORT_DIR/benchmark_${TIMESTAMP}.md"

cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

log() {
    echo -e "${BLUE}[BENCHMARK]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Check if tools exist
check_tools() {
    log "Checking required tools..."
    if ! command -v bun >/dev/null 2>&1; then
        error "bun not found. Please install it."
        exit 1
    fi
    if ! command -v svgo >/dev/null 2>&1; then
        error "svgo not found. Please install with 'bun add -g svgo'."
        exit 1
    fi
    if ! command -v bc >/dev/null 2>&1; then
        error "bc (basic calculator) not found. Please install bc (e.g., 'apt install bc' or 'brew install bc')."
        exit 1
    fi
    if [[ ! -f "./target/release/svgn" ]]; then
        error "./target/release/svgn not found. Please build with 'cargo build --release'."
        exit 1
    fi
    success "All tools available"
}

# Find SVG files recursively
find_svg_files() {
    log "Finding SVG files in $SVG_DIR..."
    if [[ ! -d "$SVG_DIR" ]]; then
        error "Directory $SVG_DIR does not exist"
        exit 1
    fi
    mapfile -t svg_files < <(find "$SVG_DIR" -type f -name "*.svg" -not -path '*/.*/*')
    if [[ ${#svg_files[@]} -lt $MIN_FILES ]]; then
        warn "Found only ${#svg_files[@]} SVG files, which is less than the minimum of $MIN_FILES."
    fi
    success "Found ${#svg_files[@]} SVG files to benchmark."
}

# Prepare report file
prepare_report() {
    mkdir -p "$REPORT_DIR"
    {
        echo "---"
        echo "layout: default"
        echo "title: Benchmark Results - ${TIMESTAMP}"
        echo "nav_order: 6"
        echo "---"
        echo ""
        echo "# Benchmark Results"
        echo ""
        echo "Comparison between \`svgo\` and \`svgn\`."
        echo "- **Date:** $(date)"
        echo "- **Iterations per file:** $ITERATIONS"
        echo "- **SVG files source:** $SVG_DIR"
        echo ""
        echo "| File | Tool | Time (ms) | Original Size (bytes) | Optimized Size (bytes) | Reduction |"
        echo "|------|------|-----------|-----------------------|------------------------|-----------|"
    } > "$REPORT_FILE"
}

# Benchmark a single tool on a single file
benchmark_file() {
    local tool_name="$1"
    local tool_cmd="$2"
    local svg_file="$3"
    
    local total_time=0
    local output_file="$TEMP_DIR/$(basename "$svg_file")"

    for (( i=0; i < ITERATIONS; i++ )); do
        local start_time
        start_time=$(date +%s.%N)
        eval "$tool_cmd \"$svg_file\" > \"$output_file\" 2>/dev/null"
        local end_time
        end_time=$(date +%s.%N)
        local duration
        duration=$(echo "($end_time - $start_time) * 1000" | bc -l)
        total_time=$(echo "$total_time + $duration" | bc -l)
    done

    local avg_time
    avg_time=$(echo "scale=4; $total_time / $ITERATIONS" | bc -l)
    
    local original_size
    original_size=$(stat -f%z "$svg_file")
    local optimized_size
    optimized_size=$(stat -f%z "$output_file")
    local reduction
    reduction=$(echo "scale=2; (($original_size - $optimized_size) * 100) / $original_size" | bc -l)

    echo "| $(basename "$svg_file") | $tool_name | $avg_time | $original_size | $optimized_size | ${reduction}% |" >> "$REPORT_FILE"
}

# Main benchmark function
run_benchmark() {
    log "Starting benchmark..."
    prepare_report

    for file in "${svg_files[@]}"; do
        log "Benchmarking $file..."
        benchmark_file "svgo" "bun --bun $(which svgo) -i" "$file"
        benchmark_file "svgn" "./target/release/svgn -i" "$file"
    done

    success "Benchmark finished. Report generated at: $REPORT_FILE"
}

# Print usage
usage() {
    echo "Usage: $0 [SVG_DIR] [ITERATIONS] [MIN_FILES]"
    echo
    echo "Arguments:"
    echo "  SVG_DIR     Directory containing SVG files (default: testdata)"
    echo "  ITERATIONS  Number of benchmark iterations (default: 3)"
    echo "  MIN_FILES   Minimum number of SVG files required (default: 5)"
}

# Main execution
main() {
    if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
        usage
        exit 0
    fi

    check_tools
    find_svg_files
    run_benchmark
}

main "$@"
