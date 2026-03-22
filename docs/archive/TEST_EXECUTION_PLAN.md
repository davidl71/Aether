# Test Execution Plan

**Date**: 2025-12-11
**Status**: Ready for Execution

---

## Execution Steps

### Step 1: Verify Python Environment

```bash
cd /Users/davidl/Projects/Trading/Aether

# Check Python version (requires 3.11+)
python3 --version

# Check pytest installation
python3 -m pytest --version

# Check coverage tools
python3 -c "import pytest_cov; print('pytest-cov installed')"
python3 -c "import coverage; print('coverage installed')"
```

### Step 2: Run Python Tests

**Recommended: Use uv/uvx for isolated environments**

```bash
# Option A: Use uvx (isolated pytest environment) - RECOMMENDED
./scripts/run_tests_uvx.sh

# Option B: Use uv (project-managed environment)
./scripts/run_tests_with_uv.sh

# Option C: Use standard test runner script
./scripts/run_python_tests.sh

# Option D: Use pytest directly
pytest python/tests/ python/integration/ -v

# Option E: Run specific test file with uvx
uvx pytest python/tests/test_security.py -v
```

### Step 3: Run Tests with Coverage

```bash
# Option A: Use uvx with coverage (RECOMMENDED)
./scripts/run_tests_uvx.sh --coverage

# Option B: Use uv with coverage
./scripts/run_tests_with_uv.sh --coverage

# Option C: Use standard test runner with coverage
./scripts/run_python_tests.sh --coverage

# Option D: Use pytest directly
pytest python/tests/ python/integration/ \
    --cov=python/services \
    --cov=python/tui \
    --cov=python/integration \
    --cov-report=term \
    --cov-report=html

# Option E: Use uvx directly
uvx pytest python/tests/ python/integration/ \
    --cov=python/services \
    --cov=python/tui \
    --cov-report=html \
    --cov-report=term
```

### Step 4: Verify Security Module Coverage

```bash
# Check security module coverage specifically
pytest python/tests/test_security.py \
    --cov=python/services/security \
    --cov-report=term

# Should show > 30% coverage for security module
```

### Step 5: Generate Baseline Coverage Report

```bash
# Generate comprehensive coverage report
./scripts/generate_python_coverage.sh --html

# View report
open htmlcov/index.html
```

---

## Expected Test Results

### Test Files to Execute

**Unit Tests** (`python/tests/`):

- `test_security.py` - 5 test classes, multiple pytest-style tests
  - Expected: All tests should pass
  - Coverage target: > 30% for security module

**Integration Tests** (`python/integration/`):

- `scripts/swiftness_integration_manual.py` – Manual Swiftness integration check (mock or real data)
- `test_swiftness_import.py`
- `test_relationship_graph.py`
- `test_nats_client.py`
  - Expected: May have some failures if dependencies not configured
  - Note: Integration tests may require external services

---

## Troubleshooting

### Import Errors

If you see import errors:

```bash
# Add python directory to PYTHONPATH
export PYTHONPATH="${PYTHONPATH}:$(pwd)/python"

# Or run from python directory
cd python
pytest tests/ -v
```

### Missing Dependencies

If dependencies are missing:

```bash
# Install from requirements
pip install -r requirements.txt

# Or install dev dependencies
pip install pytest pytest-cov coverage
```

### FastAPI Import Issues

If FastAPI imports fail:

```bash
# Install FastAPI
pip install fastapi
```

---

## Success Criteria

✅ **Tests Pass**: All unit tests should pass
✅ **Security Coverage**: Security module > 30% coverage
✅ **Baseline Established**: Coverage report generated
✅ **Documentation**: Test results documented

---

## Next Steps After Execution

1. Document test results in `docs/TEST_STATUS.md`
2. Generate baseline coverage report (Task 2.3)
3. Create missing test files based on gap analysis
4. Fix any test failures
5. Verify coverage targets met

---

**Last Updated**: 2025-12-11
