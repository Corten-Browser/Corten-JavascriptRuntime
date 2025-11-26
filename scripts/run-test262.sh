#!/bin/bash
# Test262 Compliance Test Runner
#
# Runs Test262 tests against the Corten JavaScript Runtime using the test262_harness.
# This script provides a convenient interface to run subsets of the Test262 test suite.

set -e

# Default values
TEST_DIR="test262/test/language/expressions"
MODE="parse"
LIMIT=""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS] [TEST_DIRECTORY]

Run Test262 compliance tests against Corten JavaScript Runtime.

OPTIONS:
    -e, --execute       Run in execution mode (parse + execute, default: parse only)
    -l, --limit NUM     Limit number of tests to run
    -h, --help          Show this help message

EXAMPLES:
    # Run parse-only tests for expressions (default)
    $0

    # Run parse + execute tests for expressions with limit
    $0 --execute --limit 100

    # Test statements only
    $0 test262/test/language/statements

    # Test a specific feature
    $0 test262/test/language/expressions/addition

COMMON TEST DIRECTORIES:
    test262/test/language/expressions      - Expression tests
    test262/test/language/statements       - Statement tests
    test262/test/language/literals         - Literal value tests
    test262/test/language/types            - Type system tests

EOF
    exit 0
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -e|--execute)
            MODE="execute"
            shift
            ;;
        -l|--limit)
            LIMIT="--limit $2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        -*)
            echo -e "${RED}Error: Unknown option: $1${NC}"
            usage
            ;;
        *)
            TEST_DIR="$1"
            shift
            ;;
    esac
done

# Check if Test262 repository exists
if [ ! -d "test262" ]; then
    echo -e "${RED}Error: test262 directory not found${NC}"
    echo "Please clone Test262 first:"
    echo "  git clone --depth 1 https://github.com/tc39/test262.git test262"
    exit 1
fi

# Check if test directory exists
if [ ! -d "$TEST_DIR" ]; then
    echo -e "${RED}Error: Test directory not found: $TEST_DIR${NC}"
    exit 1
fi

# Build the test262 runner if needed
echo -e "${YELLOW}Building test262_harness...${NC}"
cargo build --release --bin run_test262 2>&1 | grep -v "Compiling\|Finished\|Running" || true

# Run the tests
echo -e "${GREEN}Running Test262 tests...${NC}"
echo "Directory: $TEST_DIR"
echo "Mode: $MODE"

if [ "$MODE" = "execute" ]; then
    cargo run --release --bin run_test262 -- $LIMIT --execute "$TEST_DIR"
else
    cargo run --release --bin run_test262 -- $LIMIT "$TEST_DIR"
fi
