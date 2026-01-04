#!/bin/bash
# Script to run all astra-gui examples in succession
# Each example runs until you close it, then the next one starts
# Automatically discovers examples from the examples directory

set -e

cd "$(dirname "$0")"

# Automatically find all example files (excluding shared directory)
mapfile -t example_files < <(find ../crates/astra-gui-wgpu/examples -name "*.rs" -type f | grep -v "shared" | sort)

# Extract just the example names (without .rs extension)
examples=()
for file in "${example_files[@]}"; do
    example_name=$(basename "$file" .rs)
    examples+=("$example_name")
done

total=${#examples[@]}

if [ $total -eq 0 ]; then
    echo "Error: No examples found!"
    exit 1
fi

current=0

echo "========================================="
echo "Found $total astra-gui examples"
echo "Close each window to proceed to the next"
echo "Press Ctrl+C to stop at any time"
echo "========================================="
echo ""

for example in "${examples[@]}"; do
    current=$((current + 1))
    echo "[$current/$total] Running example: $example"
    echo "----------------------------------------"

    cargo run --example "$example" --release

    echo ""
    echo "Example '$example' closed."
    echo ""

    # Small pause between examples
    sleep 0.5
done

echo "========================================="
echo "All $total examples completed!"
echo "========================================="
