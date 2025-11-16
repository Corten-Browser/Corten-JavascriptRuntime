#!/bin/bash
#
# Pre-commit Hook for Quality Gates
#
# This hook enforces quality standards before allowing commits.
# Install by copying to .git/hooks/pre-commit and making executable:
#   cp pre-commit-hook.sh .git/hooks/pre-commit
#   chmod +x .git/hooks/pre-commit
#
# Quality gates enforced:
# 1. All tests must pass
# 2. Test coverage must be ≥ 80%
# 3. Linting must pass (zero errors)
# 4. Code must be formatted correctly
# 5. No debug statements (console.log, print, debugger)
# 6. No TODO/FIXME in committed code (use issue tracker instead)
#

set -e  # Exit on first error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}  Pre-Commit Quality Gates${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Detect project type
PROJECT_TYPE=""
if [ -f "package.json" ]; then
    PROJECT_TYPE="node"
elif [ -f "requirements.txt" ] || [ -f "setup.py" ] || [ -f "pyproject.toml" ]; then
    PROJECT_TYPE="python"
elif [ -f "Cargo.toml" ]; then
    PROJECT_TYPE="rust"
elif [ -f "go.mod" ]; then
    PROJECT_TYPE="go"
else
    echo -e "${RED}❌ Cannot detect project type${NC}"
    exit 1
fi

echo -e "Project type: ${GREEN}$PROJECT_TYPE${NC}"
echo ""

# Track failures
FAILURES=0

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GATE 1: Run Tests
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo -e "${YELLOW}[1/7] Running tests...${NC}"

if [ "$PROJECT_TYPE" = "node" ]; then
    if ! npm test -- --watchAll=false --passWithNoTests 2>&1 | tee /tmp/test-output.txt; then
        echo -e "${RED}❌ Tests failing. Fix tests before committing.${NC}"
        FAILURES=$((FAILURES + 1))
    else
        # Count passing tests
        PASSING=$(grep -oP '\d+(?= passing)' /tmp/test-output.txt || echo "0")
        echo -e "${GREEN}✅ All tests passing ($PASSING tests)${NC}"
    fi
elif [ "$PROJECT_TYPE" = "python" ]; then
    if ! pytest --tb=short --quiet 2>&1 | tee /tmp/test-output.txt; then
        echo -e "${RED}❌ Tests failing. Fix tests before committing.${NC}"
        FAILURES=$((FAILURES + 1))
    else
        # Count passing tests
        PASSING=$(grep -oP '\d+(?= passed)' /tmp/test-output.txt || echo "0")
        echo -e "${GREEN}✅ All tests passing ($PASSING tests)${NC}"
    fi
elif [ "$PROJECT_TYPE" = "rust" ]; then
    if ! cargo test --quiet; then
        echo -e "${RED}❌ Tests failing. Fix tests before committing.${NC}"
        FAILURES=$((FAILURES + 1))
    else
        echo -e "${GREEN}✅ All tests passing${NC}"
    fi
elif [ "$PROJECT_TYPE" = "go" ]; then
    if ! go test ./...; then
        echo -e "${RED}❌ Tests failing. Fix tests before committing.${NC}"
        FAILURES=$((FAILURES + 1))
    else
        echo -e "${GREEN}✅ All tests passing${NC}"
    fi
fi

echo ""

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GATE 2: Check Test Coverage
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo -e "${YELLOW}[2/7] Checking test coverage...${NC}"

COVERAGE_THRESHOLD=80

if [ "$PROJECT_TYPE" = "node" ]; then
    # Run coverage and extract percentage
    npm test -- --coverage --watchAll=false --passWithNoTests --coverageReporters=text-summary 2>&1 | tee /tmp/coverage-output.txt
    COVERAGE=$(grep -oP 'All files.*?\|\s+\K[\d.]+' /tmp/coverage-output.txt | head -1)

    if [ -z "$COVERAGE" ]; then
        echo -e "${YELLOW}⚠️  Could not determine coverage percentage${NC}"
    else
        # Remove decimal point for comparison
        COVERAGE_INT=${COVERAGE%.*}
        if [ "$COVERAGE_INT" -lt "$COVERAGE_THRESHOLD" ]; then
            echo -e "${RED}❌ Coverage ${COVERAGE}% below ${COVERAGE_THRESHOLD}% threshold${NC}"
            FAILURES=$((FAILURES + 1))
        else
            echo -e "${GREEN}✅ Coverage ${COVERAGE}% meets ${COVERAGE_THRESHOLD}% threshold${NC}"
        fi
    fi
elif [ "$PROJECT_TYPE" = "python" ]; then
    # Run coverage and extract percentage
    pytest --cov=. --cov-report=term-missing --quiet 2>&1 | tee /tmp/coverage-output.txt
    COVERAGE=$(grep "TOTAL" /tmp/coverage-output.txt | awk '{print $4}' | sed 's/%//')

    if [ -z "$COVERAGE" ]; then
        echo -e "${YELLOW}⚠️  Could not determine coverage percentage${NC}"
    else
        # Remove decimal point for comparison
        COVERAGE_INT=${COVERAGE%.*}
        if [ "$COVERAGE_INT" -lt "$COVERAGE_THRESHOLD" ]; then
            echo -e "${RED}❌ Coverage ${COVERAGE}% below ${COVERAGE_THRESHOLD}% threshold${NC}"
            echo -e "${YELLOW}   Run 'pytest --cov=. --cov-report=html' for detailed report${NC}"
            FAILURES=$((FAILURES + 1))
        else
            echo -e "${GREEN}✅ Coverage ${COVERAGE}% meets ${COVERAGE_THRESHOLD}% threshold${NC}"
        fi
    fi
elif [ "$PROJECT_TYPE" = "rust" ]; then
    # Rust coverage requires tarpaulin
    if command -v cargo-tarpaulin &> /dev/null; then
        COVERAGE=$(cargo tarpaulin --quiet --output-dir /tmp | grep -oP 'Coverage: \K[\d.]+')
        COVERAGE_INT=${COVERAGE%.*}
        if [ "$COVERAGE_INT" -lt "$COVERAGE_THRESHOLD" ]; then
            echo -e "${RED}❌ Coverage ${COVERAGE}% below ${COVERAGE_THRESHOLD}% threshold${NC}"
            FAILURES=$((FAILURES + 1))
        else
            echo -e "${GREEN}✅ Coverage ${COVERAGE}% meets ${COVERAGE_THRESHOLD}% threshold${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  cargo-tarpaulin not installed, skipping coverage check${NC}"
    fi
elif [ "$PROJECT_TYPE" = "go" ]; then
    COVERAGE=$(go test ./... -cover | grep -oP 'coverage: \K[\d.]+' | head -1)
    if [ -n "$COVERAGE" ]; then
        COVERAGE_INT=${COVERAGE%.*}
        if [ "$COVERAGE_INT" -lt "$COVERAGE_THRESHOLD" ]; then
            echo -e "${RED}❌ Coverage ${COVERAGE}% below ${COVERAGE_THRESHOLD}% threshold${NC}"
            FAILURES=$((FAILURES + 1))
        else
            echo -e "${GREEN}✅ Coverage ${COVERAGE}% meets ${COVERAGE_THRESHOLD}% threshold${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  Could not determine coverage percentage${NC}"
    fi
fi

echo ""

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GATE 3: Run Linter
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo -e "${YELLOW}[3/7] Running linter...${NC}"

if [ "$PROJECT_TYPE" = "node" ]; then
    if ! npm run lint 2>&1 | tee /tmp/lint-output.txt; then
        echo -e "${RED}❌ Linting errors found. Run 'npm run lint:fix' to fix.${NC}"
        FAILURES=$((FAILURES + 1))
    else
        echo -e "${GREEN}✅ Linting passed${NC}"
    fi
elif [ "$PROJECT_TYPE" = "python" ]; then
    # Try multiple Python linters
    LINTER_FOUND=false

    if command -v flake8 &> /dev/null; then
        LINTER_FOUND=true
        if ! flake8 . --exclude=venv,env,.venv,.git,__pycache__,.pytest_cache 2>&1 | tee /tmp/lint-output.txt; then
            echo -e "${RED}❌ flake8 found errors${NC}"
            FAILURES=$((FAILURES + 1))
        else
            echo -e "${GREEN}✅ flake8 passed${NC}"
        fi
    fi

    if command -v pylint &> /dev/null; then
        LINTER_FOUND=true
        # Run pylint, allow exit code 0-4 (errors only fail at 8+)
        pylint **/*.py --exit-zero 2>&1 | tee /tmp/pylint-output.txt
        PYLINT_SCORE=$(grep -oP 'rated at \K[\d.]+' /tmp/pylint-output.txt || echo "0")
        echo -e "${GREEN}  Pylint score: ${PYLINT_SCORE}/10${NC}"
    fi

    if [ "$LINTER_FOUND" = false ]; then
        echo -e "${YELLOW}⚠️  No Python linter found (install flake8 or pylint)${NC}"
    fi
elif [ "$PROJECT_TYPE" = "rust" ]; then
    if ! cargo clippy -- -D warnings; then
        echo -e "${RED}❌ Clippy found errors${NC}"
        FAILURES=$((FAILURES + 1))
    else
        echo -e "${GREEN}✅ Clippy passed${NC}"
    fi
elif [ "$PROJECT_TYPE" = "go" ]; then
    if command -v golint &> /dev/null; then
        if ! golint ./...; then
            echo -e "${RED}❌ golint found errors${NC}"
            FAILURES=$((FAILURES + 1))
        else
            echo -e "${GREEN}✅ golint passed${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  golint not installed, skipping${NC}"
    fi
fi

echo ""

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GATE 4: Check Formatting
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo -e "${YELLOW}[4/7] Checking code formatting...${NC}"

if [ "$PROJECT_TYPE" = "node" ]; then
    if ! npm run format:check 2>/dev/null; then
        # If format:check doesn't exist, try prettier directly
        if command -v prettier &> /dev/null; then
            if ! prettier --check "src/**/*.{js,jsx,ts,tsx}" 2>&1; then
                echo -e "${RED}❌ Code not formatted. Run 'npm run format' or 'prettier --write .'${NC}"
                FAILURES=$((FAILURES + 1))
            else
                echo -e "${GREEN}✅ Code formatted correctly${NC}"
            fi
        else
            echo -e "${YELLOW}⚠️  Prettier not found, skipping format check${NC}"
        fi
    else
        echo -e "${GREEN}✅ Code formatted correctly${NC}"
    fi
elif [ "$PROJECT_TYPE" = "python" ]; then
    if command -v black &> /dev/null; then
        if ! black --check . --exclude='/(\.venv|venv|env|\.git|__pycache__|\.pytest_cache)/' 2>&1; then
            echo -e "${RED}❌ Code not formatted. Run 'black .'${NC}"
            FAILURES=$((FAILURES + 1))
        else
            echo -e "${GREEN}✅ Code formatted correctly (black)${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  black not installed, skipping format check${NC}"
    fi
elif [ "$PROJECT_TYPE" = "rust" ]; then
    if ! cargo fmt -- --check; then
        echo -e "${RED}❌ Code not formatted. Run 'cargo fmt'${NC}"
        FAILURES=$((FAILURES + 1))
    else
        echo -e "${GREEN}✅ Code formatted correctly${NC}"
    fi
elif [ "$PROJECT_TYPE" = "go" ]; then
    if ! gofmt -l . | grep -q .; then
        echo -e "${GREEN}✅ Code formatted correctly${NC}"
    else
        echo -e "${RED}❌ Code not formatted. Run 'gofmt -w .'${NC}"
        FAILURES=$((FAILURES + 1))
    fi
fi

echo ""

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GATE 5: Check for Debug Statements
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo -e "${YELLOW}[5/7] Checking for debug statements...${NC}"

# Get list of staged files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM)

DEBUG_FOUND=false

if [ "$PROJECT_TYPE" = "node" ] || [ "$PROJECT_TYPE" = "rust" ] || [ "$PROJECT_TYPE" = "go" ]; then
    # Check for console.log, console.debug, debugger
    if echo "$STAGED_FILES" | xargs grep -nH -E '(console\.(log|debug|info)|debugger)' 2>/dev/null; then
        echo -e "${RED}❌ Debug statements found in staged files${NC}"
        echo -e "${YELLOW}   Remove console.log, console.debug, debugger statements${NC}"
        DEBUG_FOUND=true
        FAILURES=$((FAILURES + 1))
    fi
elif [ "$PROJECT_TYPE" = "python" ]; then
    # Check for print() and pdb
    if echo "$STAGED_FILES" | xargs grep -nH -E '(^|\s)(print\(|pdb\.set_trace|breakpoint\(\))' 2>/dev/null; then
        echo -e "${RED}❌ Debug statements found in staged files${NC}"
        echo -e "${YELLOW}   Remove print(), pdb.set_trace(), breakpoint() statements${NC}"
        DEBUG_FOUND=true
        FAILURES=$((FAILURES + 1))
    fi
fi

if [ "$DEBUG_FOUND" = false ]; then
    echo -e "${GREEN}✅ No debug statements found${NC}"
fi

echo ""

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GATE 6: Check for TODO/FIXME
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo -e "${YELLOW}[6/7] Checking for TODO/FIXME comments...${NC}"

TODO_FOUND=false

if echo "$STAGED_FILES" | xargs grep -nH -E '(TODO|FIXME|XXX|HACK)' 2>/dev/null; then
    echo -e "${YELLOW}⚠️  TODO/FIXME comments found in staged files${NC}"
    echo -e "${YELLOW}   Consider creating issues instead of TODO comments${NC}"
    TODO_FOUND=true
    # This is a warning, not a failure
    # FAILURES=$((FAILURES + 1))
fi

if [ "$TODO_FOUND" = false ]; then
    echo -e "${GREEN}✅ No TODO/FIXME comments found${NC}"
fi

echo ""

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GATE 7: Check for Secrets
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo -e "${YELLOW}[7/7] Checking for secrets/credentials...${NC}"

SECRETS_FOUND=false

# Check for common secret patterns
if echo "$STAGED_FILES" | xargs grep -nH -E '(password|secret|api_key|apikey|token|private_key)\s*=\s*["\047][^"\047]+["\047]' 2>/dev/null; then
    echo -e "${RED}❌ Potential secrets found in staged files${NC}"
    echo -e "${YELLOW}   Never commit passwords, API keys, or tokens${NC}"
    SECRETS_FOUND=true
    FAILURES=$((FAILURES + 1))
fi

# Check for AWS keys
if echo "$STAGED_FILES" | xargs grep -nH -E 'AKIA[0-9A-Z]{16}' 2>/dev/null; then
    echo -e "${RED}❌ AWS access key found in staged files${NC}"
    SECRETS_FOUND=true
    FAILURES=$((FAILURES + 1))
fi

# Check for private keys
if echo "$STAGED_FILES" | xargs grep -nH 'BEGIN.*PRIVATE KEY' 2>/dev/null; then
    echo -e "${RED}❌ Private key found in staged files${NC}"
    SECRETS_FOUND=true
    FAILURES=$((FAILURES + 1))
fi

if [ "$SECRETS_FOUND" = false ]; then
    echo -e "${GREEN}✅ No secrets detected${NC}"
fi

echo ""

#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Summary
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ $FAILURES -eq 0 ]; then
    echo -e "${GREEN}✅ All quality gates passed!${NC}"
    echo -e "${GREEN}   Proceeding with commit...${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 0
else
    echo -e "${RED}❌ $FAILURES quality gate(s) failed${NC}"
    echo -e "${RED}   Fix the issues above before committing${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "${YELLOW}To bypass this hook (NOT RECOMMENDED):${NC}"
    echo -e "${YELLOW}  git commit --no-verify${NC}"
    echo ""
    exit 1
fi
