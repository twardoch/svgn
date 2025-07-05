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
MIN_FILES="${3:-5}" # Reverted to exit if less than this
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

# Determine OS-specific commands
STAT_CMD=""
DATE_CMD=""
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS (BSD stat, gdate from coreutils if installed)
    STAT_CMD="stat -f%z"
    if command -v gdate >/dev/null 2>&1; then
        DATE_CMD="gdate"
    else
        DATE_CMD="date"
        warn "gdate not found. Install coreutils (brew install coreutils) for more precise timing on macOS."
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux (GNU stat)
    STAT_CMD="stat -c%s"
    DATE_CMD="date"
else
    error "Unsupported OS: $OSTYPE. Cannot determine stat and date commands."
    exit 1
fi

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
        error "Found only ${#svg_files[@]} SVG files, need at least $MIN_FILES for meaningful benchmark."
        exit 1
    }
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
        echo "- **Date:** $($DATE_CMD)"
        echo "- **Iterations per file:** $ITERATIONS"
        echo "- **SVG files source:** $SVG_DIR"
        echo ""
        echo "## Detailed Results"
        echo ""
        echo "| File | Tool | Time (ms) | Original Size (bytes) | Optimized Size (bytes) | Reduction (%) |"
        echo "|------|------|-----------|-----------------------|------------------------|---------------|"
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
        start_time=$($DATE_CMD +%s.%N)
        eval "$tool_cmd \"$svg_file\" > \"$output_file\" 2>/dev/null"
        local end_time
        end_time=$($DATE_CMD +%s.%N)
        local duration
        duration=$(echo "($end_time - $start_time) * 1000" | bc -l)
        total_time=$(echo "$total_time + $duration" | bc -l)
    done

    local avg_time
    avg_time=$(echo "scale=4; $total_time / $ITERATIONS" | bc -l)
    
    local original_size
    original_size=$($STAT_CMD "$svg_file")
    local optimized_size
    optimized_size=$($STAT_CMD "$output_file")
    local reduction
    reduction=$(echo "scale=2; (($original_size - $optimized_size) * 100) / $original_size" | bc -l)

    echo "| $(basename "$svg_file") | $tool_name | $avg_time | $original_size | $optimized_size | ${reduction}% |" >> "$REPORT_FILE"
    
    # Return values for overall summary
    echo "$avg_time $original_size $optimized_size"
}

# Main benchmark function
run_benchmark() {
    log "Starting benchmark..."
    prepare_report

    local svgo_total_time=0
    local svgn_total_time=0
    local svgo_total_original_size=0
    local svgn_total_original_size=0
    local svgo_total_optimized_size=0
    local svgn_total_optimized_size=0
    local file_count=${#svg_files[@]}

    for file in "${svg_files[@]}"; do
        log "Benchmarking $file..."
        
        # Benchmark svgo
        local svgo_metrics
        svgo_metrics=$(benchmark_file "svgo" "bun --bun $(which svgo) -i" "$file")
        local svgo_file_time=$(echo "$svgo_metrics" | awk '{print $1}')
        local svgo_file_original_size=$(echo "$svgo_metrics" | awk '{print $2}')
        local svgo_file_optimized_size=$(echo "$svgo_metrics" | awk '{print $3}')
        svgo_total_time=$(echo "$svgo_total_time + $svgo_file_time" | bc -l)
        svgo_total_original_size=$(echo "$svgo_total_original_size + $svgo_file_original_size" | bc -l)
        svgo_total_optimized_size=$(echo "$svgo_total_optimized_size + $svgo_file_optimized_size" | bc -l)

        # Benchmark svgn
        local svgn_metrics
        svgn_metrics=$(benchmark_file "svgn" "./target/release/svgn -i" "$file")
        local svgn_file_time=$(echo "$svgn_metrics" | awk '{print $1}')
        local svgn_file_original_size=$(echo "$svgn_metrics" | awk '{print $2}')
        local svgn_file_optimized_size=$(echo "$svgn_metrics" | awk '{print $3}')
        svgn_total_time=$(echo "$svgn_total_time + $svgn_file_time" | bc -l)
        svgn_total_original_size=$(echo "$svgn_total_original_size + $svgn_file_original_size" | bc -l)
        svgn_total_optimized_size=$(echo "$svgn_total_optimized_size + $svgn_file_optimized_size" | bc -l)
    done

    # Calculate overall averages
    local svgo_avg_time_overall=$(echo "scale=4; $svgo_total_time / $file_count" | bc -l)
    local svgn_avg_time_overall=$(echo "scale=4; $svgn_total_time / $file_count" | bc -l)
    
    local svgo_avg_reduction_overall=$(echo "scale=2; (($svgo_total_original_size - $svgo_total_optimized_size) * 100) / $svgo_total_original_size" | bc -l)
    local svgn_avg_reduction_overall=$(echo "scale=2; (($svgn_total_original_size - $svgn_total_optimized_size) * 100) / $svgn_total_original_size" | bc -l)

    # Append summary to report
    {
        echo ""
        echo "## Overall Summary"
        echo ""
        echo "| Metric | svgo | svgn |"
        echo "|--------|------|------|"
        echo "| Average Time per File (ms) | $svgo_avg_time_overall | $svgn_avg_time_overall |"
        echo "| Average Reduction (%) | $svgo_avg_reduction_overall | $svgn_avg_reduction_overall |"
        echo ""
    } >> "$REPORT_FILE"

    # Calculate speedup
    local speedup="N/A"
    if (( $(echo "$svgn_avg_time_overall > 0" | bc -l) )); then
        speedup=$(echo "scale=2; $svgo_avg_time_overall / $svgn_avg_time_overall" | bc -l)
    fi

    if (( $(echo "$speedup > 1" | bc -l) )); then
        echo "svgn is ${speedup}x faster than svgo (overall average)." >> "$REPORT_FILE"
        success "svgn is ${speedup}x faster than svgo (overall average)."
    elif (( $(echo "$speedup < 1" | bc -l) )); then
        local slowdown=$(echo "scale=2; $svgn_avg_time_overall / $svgo_avg_time_overall" | bc -l)
        echo "svgn is ${slowdown}x slower than svgo (overall average)." >> "$REPORT_FILE"
        warn "svgn is ${slowdown}x slower than svgo (overall average)."
    else
        echo "Both tools have similar overall performance." >> "$REPORT_FILE"
        log "Both tools have similar overall performance."
    fi

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