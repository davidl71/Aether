# Critical Action Plan - Project Scorecard Recommendations

**Generated**: 2025-11-29
**Overall Score**: 50.3% (Target: 80%+ for production readiness)
**Priority**: Critical blockers must be addressed

---

## 🎯 Executive Summary

This action plan addresses the **3 critical blockers** preventing production readiness:

1. **Security Controls** (45.5% → Target: 70%+) - +25% impact
2. **CodeQL Setup** (0% → Target: 100%) - +10% impact
3. **Testing** (0% → Target: 30%+) - +15% impact

**Total Potential Impact**: +50% to overall score (50.3% → 100.3%)

---

## 🔴 CRITICAL PRIORITY 1: Security Controls

**Current Score**: 45.5%
**Target Score**: 70%+
**Impact**: +25% to security score
**Status**: ⚠️ Partially Complete (security.py created, needs integration)

### 1.1 Complete Security Module Integration

**Status**: ✅ Security module created (`python/services/security.py`)
**Remaining Work**: Integration across all services

#### Tasks

1. **Integrate security into all FastAPI services**
   - [ ] Review all FastAPI services in `python/services/`
   - [ ] Add `RateLimitMiddleware` to each service
   - [ ] Add `PathBoundaryEnforcer` for file operations
   - [ ] Configure `AccessControl` for protected endpoints
   - [ ] Test rate limiting on all endpoints
   - [ ] Verify path validation prevents directory traversal

   **Files to Update**:
   - `python/services/swiftness_api.py` (✅ Already done)
   - `python/services/*.py` (other services if any)

2. **Add security to C++ components**
   - [ ] Review file I/O operations in `native/src/`
   - [ ] Add path validation for file reads/writes
   - [ ] Implement rate limiting for API clients (TWS, Alpaca)
   - [ ] Add input validation for user-provided paths
   - [ ] Test path boundary enforcement

   **Files to Review**:
   - `native/src/brokers/http_client.cpp`
   - `native/src/tws_client.cpp`
   - `native/src/config_manager.cpp`
   - Any file reading/writing user input

3. **Add security headers and CORS configuration**
   - [ ] Configure secure CORS origins (remove `allow_origins=["*"]`)
   - [ ] Add security headers (X-Content-Type-Options, X-Frame-Options, etc.)
   - [ ] Configure HTTPS enforcement
   - [ ] Add CSP headers

   **Files to Update**:
   - `python/services/swiftness_api.py`
   - `web/vite.config.ts` (if applicable)

4. **Environment variable configuration**
   - [ ] Document required environment variables
   - [ ] Create `.env.example` with security settings
   - [ ] Set up production environment variables
   - [ ] Configure API key management

   **Files to Create/Update**:
   - `.env.example`
   - `docs/SECURITY_CONFIGURATION.md`

5. **Security testing**
   - [ ] Write tests for rate limiting
   - [ ] Write tests for path boundary enforcement
   - [ ] Write tests for access control
   - [ ] Test directory traversal prevention
   - [ ] Test rate limit enforcement

   **Test Files to Create**:
   - `python/tests/test_security.py`
   - `native/tests/security_test.cpp`

**Estimated Time**: 8-12 hours
**Dependencies**: None
**Priority**: Critical

---

### 1.2 Security Documentation

- [ ] Document security features in `docs/SECURITY.md`
- [ ] Add security best practices guide
- [ ] Document rate limiting configuration
- [ ] Document path boundary enforcement rules
- [ ] Add security checklist for new features

**Files to Create**:

- `docs/SECURITY.md`
- `docs/SECURITY_BEST_PRACTICES.md`

**Estimated Time**: 2-3 hours
**Dependencies**: 1.1 (after implementation)
**Priority**: High

---

## 🔴 CRITICAL PRIORITY 2: CodeQL Setup

**Current Score**: 0%
**Target Score**: 100%
**Impact**: +10% to security score
**Status**: ❌ Not Started

### 2.1 Enable CodeQL Workflow

#### Tasks

1. **Create CodeQL workflow file**
   - [ ] Create `.github/workflows/codeql.yml`
   - [ ] Configure CodeQL for C++ (primary language)
   - [ ] Configure CodeQL for Python (secondary language)
   - [ ] Configure CodeQL for TypeScript/JavaScript (web)
   - [ ] Set up scheduled runs (weekly)
   - [ ] Configure PR analysis

   **File to Create**:
   - `.github/workflows/codeql.yml`

   **Template**:

   ```yaml
   name: "CodeQL"

   on:
     push:
       branches: [ "main" ]
     pull_request:
       branches: [ "main" ]
     schedule:
       - cron: '0 0 * * 0' # Weekly on Sunday

   jobs:
     analyze:
       name: Analyze
       runs-on: ubuntu-latest
       permissions:
         actions: read
         contents: read
         security-events: write

       strategy:
         fail-fast: false
         matrix:
           language: [ 'cpp', 'python', 'javascript' ]

       steps:
       - name: Checkout repository
         uses: actions/checkout@v4

       - name: Initialize CodeQL
         uses: github/codeql-action/init@v3
         with:
           languages: ${{ matrix.language }}

       - name: Autobuild
         uses: github/codeql-action/autobuild@v3

       - name: Perform CodeQL Analysis
         uses: github/codeql-action/analyze@v3
   ```

2. **Configure CodeQL queries**
   - [ ] Enable security queries
   - [ ] Enable quality queries
   - [ ] Configure query suites
   - [ ] Add custom queries if needed

   **File to Create**:
   - `.github/codeql/codeql-config.yml`

3. **Test CodeQL workflow**
   - [ ] Push workflow to test branch
   - [ ] Verify CodeQL runs successfully
   - [ ] Review initial findings
   - [ ] Fix any workflow configuration issues

4. **Document CodeQL setup**
   - [ ] Document how to view CodeQL results
   - [ ] Document how to fix CodeQL findings
   - [ ] Add CodeQL to security documentation

**Estimated Time**: 2-3 hours
**Dependencies**: None
**Priority**: Critical

---

### 2.2 Address Initial CodeQL Findings

- [ ] Review first CodeQL scan results
- [ ] Prioritize security findings
- [ ] Create tasks for fixing high-priority findings
- [ ] Fix low-hanging fruit (quick fixes)
- [ ] Document complex findings for later

**Estimated Time**: 4-8 hours (depends on findings)
**Dependencies**: 2.1 (after first scan)
**Priority**: High

---

## 🔴 CRITICAL PRIORITY 3: Testing

**Current Score**: 0.0%
**Target Score**: 30%+
**Impact**: +15% to testing score
**Status**: ❌ Not Started

### 3.1 Fix Failing Tests

#### Tasks

1. **Identify failing tests**
   - [ ] Run test suite: `ctest --test-dir build --output-on-failure`
   - [ ] Run Python tests: `pytest python/tests/`
   - [ ] Document all failing tests
   - [ ] Categorize failures (compilation, runtime, assertion)
   - [ ] Prioritize by criticality

   **Commands**:

   ```bash
   # C++ tests
   cd build && ctest --output-on-failure

   # Python tests
   pytest python/tests/ -v
   ```

2. **Fix compilation errors**
   - [ ] Fix C++ compilation errors
   - [ ] Fix Python import errors
   - [ ] Fix missing dependencies
   - [ ] Update CMake configuration if needed
   - [ ] Verify all tests compile

3. **Fix runtime errors**
   - [ ] Fix test setup/teardown issues
   - [ ] Fix mock data issues
   - [ ] Fix API client initialization
   - [ ] Fix test environment configuration

4. **Fix assertion failures**
   - [ ] Review test expectations
   - [ ] Update outdated assertions
   - [ ] Fix floating-point comparison issues
   - [ ] Fix timing-related test failures

**Estimated Time**: 8-16 hours
**Dependencies**: None
**Priority**: Critical

---

### 3.2 Increase Test Coverage

#### Tasks

1. **Measure current coverage**
   - [ ] Set up coverage tools (gcov for C++, coverage.py for Python)
   - [ ] Run coverage analysis
   - [ ] Generate coverage reports
   - [ ] Identify uncovered code paths

   **Tools**:
   - C++: `gcov`, `lcov`
   - Python: `coverage.py`, `pytest-cov`

2. **Add tests for critical paths**
   - [ ] Box spread calculation tests
   - [ ] Risk management tests
   - [ ] Order validation tests
   - [ ] Market data handling tests
   - [ ] Configuration validation tests

   **Priority Areas**:
   - `native/src/strategies/box_spread/` (trading logic)
   - `native/src/order_manager.cpp` (order validation)
   - `native/src/risk_calculator.cpp` (risk management)
   - `python/integration/` (API integrations)

3. **Add integration tests**
   - [ ] TWS API integration tests (mock)
   - [ ] Alpaca API integration tests (mock)
   - [ ] End-to-end workflow tests
   - [ ] Multi-broker scenario tests

4. **Add unit tests for utilities**
   - [ ] Rate limiter tests
   - [ ] Configuration manager tests
   - [ ] Type conversion tests
   - [ ] Validation utility tests

5. **Set up coverage reporting**
   - [ ] Configure coverage thresholds (30% minimum)
   - [ ] Add coverage to CI/CD
   - [ ] Generate HTML coverage reports
   - [ ] Track coverage over time

**Estimated Time**: 16-24 hours
**Dependencies**: 3.1 (after tests are fixed)
**Priority**: Critical

---

### 3.3 Test Infrastructure

- [ ] Set up test data fixtures
- [ ] Create mock TWS API server
- [ ] Create mock Alpaca API server
- [ ] Set up test database (if needed)
- [ ] Document test setup process

**Files to Create**:

- `python/tests/fixtures/`
- `python/tests/mocks/`
- `docs/TESTING.md`

**Estimated Time**: 4-6 hours
**Dependencies**: 3.1, 3.2
**Priority**: Medium

---

## 📊 Implementation Timeline

### Week 1: Security & CodeQL Setup

- **Days 1-2**: Complete security module integration (1.1)
- **Days 3-4**: CodeQL workflow setup (2.1)
- **Day 5**: Security documentation (1.2)

### Week 2: Testing Foundation

- **Days 1-3**: Fix failing tests (3.1)
- **Days 4-5**: Initial coverage analysis (3.2.1)

### Week 3: Test Coverage

- **Days 1-4**: Add critical path tests (3.2.2)
- **Day 5**: Integration tests (3.2.3)

### Week 4: Polish & Documentation

- **Days 1-2**: Test infrastructure (3.3)
- **Days 3-4**: Address CodeQL findings (2.2)
- **Day 5**: Final documentation and review

**Total Estimated Time**: 40-60 hours over 4 weeks

---

## 🎯 Success Criteria

### Security (Target: 70%+)

- ✅ Rate limiting enabled on all API endpoints
- ✅ Path boundary enforcement active
- ✅ Access control configured
- ✅ CodeQL workflow running
- ✅ Security documentation complete

### Testing (Target: 30%+)

- ✅ All existing tests passing
- ✅ 30%+ code coverage achieved
- ✅ Critical paths have tests
- ✅ Coverage reporting in CI/CD

### Overall Score (Target: 80%+)

- ✅ Security score: 70%+
- ✅ Testing score: 30%+
- ✅ CodeQL: 100%
- ✅ Overall: 80%+ (production ready)

---

## 📝 Tracking

### Progress Tracking

- [ ] Create Todo2 tasks for each major task above
- [ ] Set up weekly progress reviews
- [ ] Track time spent on each area
- [ ] Update scorecard weekly

### Metrics to Track

- Security score (current: 45.5%, target: 70%+)
- Testing score (current: 0%, target: 30%+)
- CodeQL status (current: 0%, target: 100%)
- Overall score (current: 50.3%, target: 80%+)
- Test coverage percentage
- Number of CodeQL findings

---

## 🚨 Risk Mitigation

### Potential Risks

1. **Security integration complexity**
   - **Risk**: Security module may not integrate cleanly with existing code
   - **Mitigation**: Start with one service, iterate, then expand

2. **Test failures may reveal bugs**
   - **Risk**: Fixing tests may require fixing production code
   - **Mitigation**: Prioritize critical path tests first

3. **CodeQL may find many issues**
   - **Risk**: Overwhelming number of findings
   - **Mitigation**: Focus on security findings first, quality later

4. **Time estimates may be optimistic**
   - **Risk**: Tasks take longer than estimated
   - **Mitigation**: Buffer time, prioritize critical items

---

## 📚 Resources

### Documentation

- [GitHub CodeQL Documentation](https://docs.github.com/en/code-security/codeql-cli)
- [FastAPI Security Best Practices](https://fastapi.tiangolo.com/tutorial/security/)
- [C++ Testing Best Practices](https://github.com/catchorg/Catch2)
- [Python Testing Best Practices](https://docs.pytest.org/)

### Internal Documentation

- `docs/SECURITY.md` (to be created)
- `docs/TESTING.md` (to be created)
- `docs/DESIGN_DECISIONS.md` (already exists)

---

## ✅ Next Steps

1. **Immediate** (Today):
   - Review this action plan
   - Create Todo2 tasks for Week 1 items
   - Start with security module integration

2. **This Week**:
   - Complete security integration (1.1)
   - Set up CodeQL workflow (2.1)
   - Begin fixing failing tests (3.1)

3. **This Month**:
   - Complete all critical priorities
   - Achieve 30%+ test coverage
   - Get CodeQL running
   - Reach 80%+ overall score

---

**Last Updated**: 2025-11-29
**Status**: Ready for Implementation
**Owner**: Development Team
