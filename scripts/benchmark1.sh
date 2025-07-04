#!/usr/bin/env bash
# this_file: scripts/benchmark1.sh

set -euo pipefail
cd $(dirname "$0")/..

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
MIN_FILES="${3:-10}"

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

    if ! command -v npx >/dev/null 2>&1; then
        error "npx not found. Please install Node.js and npm."
        exit 1
    fi

    if ! npx svgo --version >/dev/null 2>&1; then
        error "svgo not found. Please install svgo with 'npm install -g svgo'."
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

    if ! ./target/release/svgn --version >/dev/null 2>&1; then
        error "./target/release/svgn not executable or failed to run. Please build with 'cargo build --release'."
        exit 1
    fi

    success "All tools available"
}

# Find SVG files recursively
find_svg_files() {
    log "Finding SVG files recursively in $SVG_DIR..."

    if [[ ! -d "$SVG_DIR" ]]; then
        error "Directory $SVG_DIR does not exist"
        exit 1
    fi

    mapfile -t svg_files < <(find "$SVG_DIR" -type f -name "*.svg" -not -path '*/\.*')

    if [[ ${#svg_files[@]} -lt $MIN_FILES ]]; then
        error "Found only ${#svg_files[@]} SVG files, need at least $MIN_FILES"
        exit 1
    fi

    success "Found ${#svg_files[@]} SVG files"
}

# Benchmark a single tool
benchmark_tool() {
    local tool_name="$1"
    local tool_cmd="$2"
    local files=("${@:3}")

    log "Benchmarking $tool_name (${#files[@]} files, $ITERATIONS iterations)..."

    local total_time=0
    local successful_files=0
    local failed_files=0

    # Create a subdirectory for this tool's outputs
    local tool_output_dir="$TEMP_DIR/$tool_name"
    mkdir -p "$tool_output_dir"

    for iteration in $(seq 1 $ITERATIONS); do
        log "  Iteration $iteration/$ITERATIONS"
        local iteration_time=0

        # Create iteration-specific directory
        local iteration_dir="$tool_output_dir/iteration_$iteration"
        mkdir -p "$iteration_dir"

        for svg_file in "${files[@]}"; do
            local output_file="$iteration_dir/$(basename "$svg_file")"

            # Time the command
            local start_time=$(date +%s.%N)
            if eval "$tool_cmd \"$svg_file\" > \"$output_file\" 2>/dev/null"; then
                local end_time=$(date +%s.%N)
                local file_time=$(echo "$end_time - $start_time" | bc -l 2>/dev/null || echo "0")
                iteration_time=$(echo "$iteration_time + $file_time" | bc -l 2>/dev/null || echo "$iteration_time")
                ((successful_files++))
            else
                ((failed_files++))
            fi
        done

        total_time=$(echo "$total_time + $iteration_time" | bc -l 2>/dev/null || echo "$total_time")
        log "    Iteration time: ${iteration_time}s"
    done

    local avg_time="0"
    local avg_per_file="0"

    if [[ "$ITERATIONS" -gt 0 ]] && [[ "$total_time" != "0" ]]; then
        avg_time=$(echo "scale=6; $total_time / $ITERATIONS" | bc -l 2>/dev/null || echo "0")
    fi

    if [[ "${#files[@]}" -gt 0 ]] && [[ "$avg_time" != "0" ]]; then
        avg_per_file=$(echo "scale=6; $avg_time / ${#files[@]}" | bc -l 2>/dev/null || echo "0")
    fi

    log "  $tool_name completed: total_time=$total_time, avg_time=$avg_time, avg_per_file=$avg_per_file, successful=$successful_files, failed=$failed_files"
    echo "$tool_name,$avg_time,$avg_per_file,$successful_files,$failed_files"
}

# Main benchmark function
run_benchmark() {
    log "Starting benchmark comparison..."

    # Prepare file subset for testing
    local test_files=("${svg_files[@]:0:100}") # Use first 100 files max

    # Create results file
    local results_file="$TEMP_DIR/benchmark_results.csv"
    echo "Tool,Total_Time_Avg,Per_File_Avg,Successful_Files,Failed_Files" >"$results_file"

    # Benchmark npx svgo
    local svgo_result
    svgo_result=$(benchmark_tool "svgo" "bunx --bun svgo -i" "${test_files[@]}")
    echo "$svgo_result" >>"$results_file"

    # Benchmark our svgn
    local svgn_result
    svgn_result=$(benchmark_tool "svgn" "./target/release/svgn -i " "${test_files[@]}")
    echo "$svgn_result" >>"$results_file"

    # Display results
    log "Benchmark Results:"
    echo
    printf "%-12s %-15s %-15s %-15s %-15s\n" "Tool" "Total Time (s)" "Per File (s)" "Success" "Failed"
    printf "%-12s %-15s %-15s %-15s %-15s\n" "----" "-------------" "-------------" "-------" "------"

    while IFS=',' read -r tool total_time per_file success failed; do
        if [[ "$tool" != "Tool" ]]; then # Skip header
            # Validate that total_time and per_file are numeric
            if [[ "$total_time" =~ ^[0-9]+\.?[0-9]*$ ]] && [[ "$per_file" =~ ^[0-9]+\.?[0-9]*$ ]]; then
                printf "%-12s %-15.6f %-15.6f %-15s %-15s\n" "$tool" "$total_time" "$per_file" "$success" "$failed"
            else
                printf "%-12s %-15s %-15s %-15s %-15s\n" "$tool" "$total_time" "$per_file" "$success" "$failed"
            fi
        fi
    done <"$results_file"

    # Calculate speedup
    local svgo_time=$(echo "$svgo_result" | cut -d',' -f2)
    local svgn_time=$(echo "$svgn_result" | cut -d',' -f2)
    local speedup=$(echo "scale=2; $svgo_time / $svgn_time" | bc -l)

    echo
    if (($(echo "$speedup > 1" | bc -l))); then
        success "svgn is ${speedup}x faster than npx svgo"
    elif (($(echo "$speedup < 1" | bc -l))); then
        local slowdown=$(echo "scale=2; $svgn_time / $svgo_time" | bc -l)
        warn "svgn is ${slowdown}x slower than npx svgo"
    else
        log "Both tools have similar performance"
    fi

    # Save detailed results
    local final_results="benchmark_results_$(date +%Y%m%d_%H%M%S).csv"
    cp "$results_file" "$final_results"
    log "Detailed results saved to: $final_results"
}

# Print usage
usage() {
    echo "Usage: $0 [SVG_DIR] [ITERATIONS] [MIN_FILES]"
    echo
    echo "Arguments:"
    echo "  SVG_DIR     Directory containing SVG files (default: testdata)"
    echo "  ITERATIONS  Number of benchmark iterations (default: 3)"
    echo "  MIN_FILES   Minimum number of SVG files required (default: 10)"
    echo
    echo "Example:"
    echo "  $0 testdata 5 20"
    echo "  $0 /path/to/svg/files"
}

# Main execution
main() {
    if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
        usage
        exit 0
    fi

    log "SVG Optimization Benchmark Tool"
    log "Comparing npx svgo vs ./target/release/svgn"
    echo

    check_tools
    find_svg_files
    run_benchmark

    success "Benchmark completed!"
}

main "$@"
