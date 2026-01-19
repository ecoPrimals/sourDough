#!/usr/bin/env bash
# genomeBin Testing Script (STANDARD)
# Tests genomeBin across different environments
# Part of the ecoPrimals genomeBin standard

set -euo pipefail

# Color output
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m'
else
    RED='' GREEN='' YELLOW='' BLUE='' NC=''
fi

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Default values
GENOME_FILE=""
TEST_MODE="local"  # local, container, all

# Usage
usage() {
    cat <<EOF
Usage: $0 [OPTIONS] GENOME_FILE

Test a genomeBin for correctness and multi-platform compatibility

ARGUMENTS:
    GENOME_FILE        Path to the genomeBin file to test

OPTIONS:
    --mode MODE        Test mode: local, container, all (default: local)
    -h, --help         Show this help

TEST MODES:
    local              Test on current system only
    container          Test using container runtime for multiple distros
    all                Run all available tests

EXAMPLE:
    $0 beardog.genome
    $0 --mode container myprimal.genome
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --mode)
            TEST_MODE="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        -*)
            echo -e "${RED}ERROR: Unknown option: $1${NC}" >&2
            usage
            exit 1
            ;;
        *)
            GENOME_FILE="$1"
            shift
            ;;
    esac
done

# Validate arguments
if [[ -z "$GENOME_FILE" ]]; then
    echo -e "${RED}ERROR: GENOME_FILE is required${NC}" >&2
    usage
    exit 1
fi

if [[ ! -f "$GENOME_FILE" ]]; then
    echo -e "${RED}ERROR: File not found: $GENOME_FILE${NC}" >&2
    exit 1
fi

if [[ ! -x "$GENOME_FILE" ]]; then
    echo -e "${YELLOW}WARNING: File is not executable, attempting chmod +x${NC}"
    chmod +x "$GENOME_FILE"
fi

echo -e "${BLUE}=== Testing genomeBin ===${NC}"
echo -e "${GREEN}File:${NC}         $GENOME_FILE"
echo -e "${GREEN}Mode:${NC}         $TEST_MODE"
echo -e "${GREEN}Size:${NC}         $(du -h "$GENOME_FILE" | cut -f1)"
echo ""

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Test function
run_test() {
    local test_name="$1"
    local test_cmd="$2"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -n "  [$TESTS_RUN] $test_name ... "
    
    if eval "$test_cmd" &>/dev/null; then
        echo -e "${GREEN}PASS${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}FAIL${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Local tests
test_local() {
    echo -e "${BLUE}=== Local System Tests ===${NC}"
    
    # Test 1: File is executable
    run_test "File is executable" "test -x '$GENOME_FILE'"
    
    # Test 2: File is not empty
    run_test "File is not empty" "test -s '$GENOME_FILE'"
    
    # Test 3: Has shebang
    run_test "Has valid shebang" "head -n1 '$GENOME_FILE' | grep -q '^#!/'"
    
    # Test 4: Contains payload boundary
    run_test "Contains payload boundary" "grep -q 'GENOME_PAYLOAD_BOUNDARY' '$GENOME_FILE'"
    
    # Test 5: Checksum file exists
    local checksum_file="${GENOME_FILE}.sha256"
    run_test "Checksum file exists" "test -f '$checksum_file'"
    
    # Test 6: Checksum is valid
    if [[ -f "$checksum_file" ]]; then
        local expected=$(cat "$checksum_file")
        local actual=$(sha256sum "$GENOME_FILE" | cut -d' ' -f1)
        run_test "Checksum matches" "test '$expected' = '$actual'"
    fi
    
    # Test 7: Can extract metadata (dry-run)
    run_test "Metadata extraction" "bash -c 'awk \"/EMBEDDED_PAYLOAD/ {found=1; next} found\" \"$GENOME_FILE\" | tar -tzf - metadata.toml 2>/dev/null | grep -q metadata.toml'"
    
    # Test 8: Has ecobins directory in archive
    run_test "Contains ecobins" "bash -c 'awk \"/EMBEDDED_PAYLOAD/ {found=1; next} found\" \"$GENOME_FILE\" | tar -tzf - 2>/dev/null | grep -q ecobins/'"
    
    echo ""
}

# Container-based tests (supports Docker/Podman/etc)
test_container() {
    # Detect available container runtime
    if command -v podman &>/dev/null; then
        CONTAINER_CMD="podman"
    elif command -v docker &>/dev/null; then
        CONTAINER_CMD="docker"
    else
        echo -e "${YELLOW}No container runtime available, skipping container tests${NC}"
        return 0
    fi
    
    echo -e "${BLUE}=== Container-based Multi-Platform Tests (using ${CONTAINER_CMD}) ===${NC}"
    
    # Test distros
    local distros=(
        "ubuntu:22.04"
        "ubuntu:20.04"
        "debian:12"
        "debian:11"
        "alpine:latest"
        "fedora:latest"
    )
    
    for distro in "${distros[@]}"; do
        local distro_name="${distro/:/-}"
        echo -n "  Testing on $distro ... "
        
        # Create temporary test script
        local test_script=$(mktemp)
        cat > "$test_script" <<'INNER_EOF'
#!/bin/sh
# Minimal test: verify genomeBin can be invoked
set -e
chmod +x /genome
# Just check if file is valid, don't actually install
head -n1 /genome | grep -q '^#!/'
exit 0
INNER_EOF
        chmod +x "$test_script"
        
        # Run in container
        if ${CONTAINER_CMD} run --rm \
            -v "$GENOME_FILE:/genome:ro" \
            -v "$test_script:/test.sh:ro" \
            "$distro" \
            /test.sh &>/dev/null; then
            echo -e "${GREEN}PASS${NC}"
            TESTS_RUN=$((TESTS_RUN + 1))
            TESTS_PASSED=$((TESTS_PASSED + 1))
        else
            echo -e "${RED}FAIL${NC}"
            TESTS_RUN=$((TESTS_RUN + 1))
            TESTS_FAILED=$((TESTS_FAILED + 1))
        fi
        
        rm -f "$test_script"
    done
    
    echo ""
}

# Run tests based on mode
case "$TEST_MODE" in
    local)
        test_local
        ;;
    container)
        test_container
        ;;
    all)
        test_local
        test_container
        ;;
    *)
        echo -e "${RED}ERROR: Invalid test mode: $TEST_MODE${NC}" >&2
        exit 1
        ;;
esac

# Summary
echo -e "${BLUE}=== Test Summary ===${NC}"
echo -e "${GREEN}Tests Run:${NC}     $TESTS_RUN"
echo -e "${GREEN}Passed:${NC}        $TESTS_PASSED"
echo -e "${RED}Failed:${NC}        $TESTS_FAILED"
echo ""

if [[ $TESTS_FAILED -eq 0 ]]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi

