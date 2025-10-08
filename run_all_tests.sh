#!/bin/bash
# Run all radix-router test examples
# Usage: ./run_all_tests.sh [--release]

set -e

RELEASE=""
if [ "$1" = "--release" ]; then
    RELEASE="--release"
    echo "Running in RELEASE mode for better performance"
    echo ""
fi

echo "╔════════════════════════════════════════════════════════════╗"
echo "║      Radix Router - Complete Test Suite Runner            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Functional tests
functional_tests=(
    "basic:Basic Functionality"
    "edge_cases:Edge Cases & Boundary Conditions"
    "integration:Real-World Integration Scenarios"
    "vars_filter_test:Variables & Filter Functions"
)

# Performance tests
performance_tests=(
    "benchmark:Performance Benchmarks"
    "concurrency_test:Concurrent Performance"
    "stress_test:Stress & Load Testing"
)

echo "┌────────────────────────────────────────────────────────────┐"
echo "│  Part 1: Functional Tests                                 │"
echo "└────────────────────────────────────────────────────────────┘"
echo ""

for test_info in "${functional_tests[@]}"; do
    IFS=':' read -r test_name test_desc <<< "$test_info"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Running: $test_desc"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo run --example "$test_name" $RELEASE
    echo ""
done

echo "┌────────────────────────────────────────────────────────────┐"
echo "│  Part 2: Performance Tests                                │"
echo "└────────────────────────────────────────────────────────────┘"
echo ""

# Always use release mode for performance tests
PERF_MODE="--release"
if [ -z "$RELEASE" ]; then
    echo "Note: Performance tests will run in RELEASE mode for accurate results"
    echo ""
fi

for test_info in "${performance_tests[@]}"; do
    IFS=':' read -r test_name test_desc <<< "$test_info"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Running: $test_desc"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo run --example "$test_name" $PERF_MODE
    echo ""
done

echo "╔════════════════════════════════════════════════════════════╗"
echo "║              All Tests Completed Successfully! ✓           ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Summary:"
echo "  • Functional tests: ${#functional_tests[@]} passed"
echo "  • Performance tests: ${#performance_tests[@]} passed"
echo "  • Total: $((${#functional_tests[@]} + ${#performance_tests[@]})) test suites"
echo ""

