# Repository & Codebase Health Automation Plan

**Date**: 2025-11-20
**Purpose**: Comprehensive plan for automating repository and codebase health monitoring

---

## Overview

This plan outlines automation tasks to monitor and maintain repository health, code quality, security, and build system integrity.

---

## Automation Categories

### 1. 🔍 Code Quality & Static Analysis

#### 1.1 Linter & Static Analysis Automation

**Frequency**: Daily
**Script**: New - `scripts/automate_linter_analysis.py`
**Purpose**: Automate existing linter runs and track trends
**Current**: `scripts/run_linters.sh` exists but not automated

**Actions**:

- Run `scripts/run_linters.sh` (cppcheck, clang-tidy, Infer)
- Run Trunk linters (bandit, ruff, clippy, shellcheck, etc.)
- Parse and aggregate results
- Track issue trends over time
- Identify new issues vs. resolved issues
- Create Todo2 tasks for high-priority issues
- Generate health score

**Cron Schedule**: `0 7 * * *` (7 AM daily)

**Output**: `docs/LINTER_ANALYSIS_REPORT.md`

**Tools Available**:

- ✅ cppcheck (C++)
- ✅ clang-tidy (C++)
- ✅ Trunk (multi-language: Python, Rust, Go, TypeScript, Shell)
- ✅ bandit (Python security)
- ✅ ruff (Python linting)
- ✅ clippy (Rust)
- ✅ shellcheck (Shell scripts)

---

#### 1.2 Test Coverage Analysis

**Frequency**: Daily
**Script**: New - `scripts/automate_test_coverage.py`
**Purpose**: Monitor test coverage trends and identify gaps

**Actions**:

- Run C++ tests with coverage (gcov/lcov)
- Run Python tests with coverage (pytest-cov)
- Run Rust tests with coverage (cargo-tarpaulin)
- Aggregate coverage metrics
- Track coverage trends over time
- Identify untested files/functions
- Calculate coverage by component
- Create Todo2 tasks for low-coverage areas (<80%)
- Generate coverage report

**Cron Schedule**: `0 7 * * *` (7 AM daily, after linters)

**Output**: `docs/TEST_COVERAGE_REPORT.md`

**Target Coverage**:

- Critical code: 90%+
- Core logic: 80%+
- Utilities: 70%+

---

#### 1.3 Code Complexity Analysis

**Frequency**: Weekly
**Script**: New - `scripts/automate_complexity_analysis.py`
**Purpose**: Identify overly complex code that needs refactoring

**Actions**:

- Analyze cyclomatic complexity (C++, Python, Rust)
- Identify functions with high complexity (>15)
- Track complexity trends
- Find code duplication
- Suggest refactoring opportunities
- Create Todo2 tasks for high-complexity code

**Cron Schedule**: `0 10 * * 1` (10 AM Mondays)

**Output**: `docs/COMPLEXITY_ANALYSIS_REPORT.md`

**Tools**:

- lizard (multi-language complexity)
- radon (Python complexity)
- clippy (Rust complexity hints)

---

### 2. 🔒 Security & Dependency Health

#### 2.1 Dependency Security Scan

**Frequency**: Daily
**Script**: New - `scripts/automate_dependency_security.py`
**Purpose**: Check for vulnerable dependencies

**Actions**:

- Scan Python dependencies (pip-audit, safety, osv-scanner)
- Scan Rust dependencies (cargo-audit)
- Scan npm dependencies (npm audit)
- Check for known CVEs
- Categorize by severity (critical, high, medium, low)
- Alert on critical/high vulnerabilities
- Create Todo2 tasks for security updates
- Track vulnerability trends

**Cron Schedule**: `0 6 * * *` (6 AM daily)

**Output**: `docs/DEPENDENCY_SECURITY_REPORT.md`

**Tools Available**:

- ✅ osv-scanner (configured in Trunk)
- ✅ pip-audit (Python)
- ✅ cargo-audit (Rust)
- ✅ npm audit (Node.js)
- ✅ trufflehog (secrets scanning, configured in Trunk)

---

#### 2.2 Secret Scanning

**Frequency**: Daily
**Script**: New - `scripts/automate_secret_scanning.py`
**Purpose**: Detect accidentally committed secrets

**Actions**:

- Run trufflehog (already in Trunk config)
- Scan for API keys, tokens, passwords
- Check git history for secrets
- Alert on findings
- Create Todo2 tasks for secret rotation
- Track secret detection trends

**Cron Schedule**: `0 6 * * *` (6 AM daily, with security scan)

**Output**: `docs/SECRET_SCAN_REPORT.md`

**Tools**:

- ✅ trufflehog (configured in Trunk)

---

#### 2.3 Dependency Update Check

**Frequency**: Weekly
**Script**: New - `scripts/automate_dependency_updates.py`
**Purpose**: Identify outdated dependencies

**Actions**:

- Check Python dependencies (pip list --outdated)
- Check Rust dependencies (cargo outdated)
- Check npm dependencies (npm outdated)
- Identify security updates
- Check compatibility with current codebase
- Create Todo2 tasks for safe updates
- Generate update report

**Cron Schedule**: `0 9 * * 1` (9 AM Mondays)

**Output**: `docs/DEPENDENCY_UPDATE_REPORT.md`

**Tools**:

- pip list --outdated
- cargo-outdated (Rust)
- npm outdated

---

### 3. 🏗️ Build & Infrastructure Health

#### 3.1 Build Health Check

**Frequency**: Daily
**Script**: New - `scripts/automate_build_health.py`
**Purpose**: Monitor build system health

**Actions**:

- Test build on all platforms/configs
- Measure build times (trend analysis)
- Identify build failures
- Check build artifact sizes
- Monitor compilation warnings
- Track build success rate
- Alert on build issues
- Create Todo2 tasks for build problems

**Cron Schedule**: `0 5 * * *` (5 AM daily)

**Output**: `docs/BUILD_HEALTH_REPORT.md`

**Build Configs to Test**:

- Debug (macOS)
- Release (macOS)
- Universal binary
- C++ TUI
- Python bindings
- Rust backend

---

#### 3.2 Configuration Drift Detection

**Frequency**: Weekly
**Script**: New - `scripts/automate_config_drift.py`
**Purpose**: Detect configuration inconsistencies

**Actions**:

- Compare CMakeLists.txt across components
- Check for missing configs
- Validate config formats
- Detect environment-specific drift
- Compare .cursorrules vs. actual usage
- Check MCP server configurations
- Create Todo2 tasks for drift fixes

**Cron Schedule**: `0 11 * * 1` (11 AM Mondays)

**Output**: `docs/CONFIG_DRIFT_REPORT.md`

---

#### 3.3 File System Health Check

**Frequency**: Weekly
**Script**: New - `scripts/automate_filesystem_health.py`
**Purpose**: Monitor repository file system health

**Actions**:

- Check for large files (>10MB)
- Find duplicate files
- Identify orphaned files
- Check .gitignore coverage
- Find missing .gitignore entries
- Detect binary files in wrong locations
- Check file permissions
- Create cleanup tasks

**Cron Schedule**: `0 12 * * 1` (Noon Mondays)

**Output**: `docs/FILESYSTEM_HEALTH_REPORT.md`

---

### 4. 📊 Repository Metrics & Trends

#### 4.1 Code Metrics Dashboard

**Frequency**: Daily
**Script**: New - `scripts/automate_code_metrics.py`
**Purpose**: Track codebase metrics and trends

**Actions**:

- Count lines of code by language
- Track file counts
- Measure code growth rate
- Calculate test-to-code ratio
- Track commit frequency
- Measure code churn
- Generate metrics dashboard

**Cron Schedule**: `0 8 * * *` (8 AM daily)

**Output**: `docs/CODE_METRICS_DASHBOARD.md`

**Metrics Tracked**:

- Total LOC (by language)
- Test LOC vs. production LOC
- Files by type
- Average file size
- Code growth rate
- Commit frequency

---

#### 4.2 Git Repository Health

**Frequency**: Weekly
**Script**: New - `scripts/automate_git_health.py`
**Purpose**: Monitor git repository health

**Actions**:

- Check for large commits
- Find merge conflicts in history
- Detect force pushes
- Check branch hygiene
- Find stale branches
- Monitor commit message quality
- Track contributor activity
- Create cleanup tasks

**Cron Schedule**: `0 13 * * 1` (1 PM Mondays)

**Output**: `docs/GIT_HEALTH_REPORT.md`

---

### 5. 🔗 Integration & API Health

#### 5.1 API Contract Validation

**Frequency**: Daily
**Script**: New - `scripts/automate_api_contract_validation.py`
**Purpose**: Validate API contract consistency

**Actions**:

- Validate `agents/shared/API_CONTRACT.md` format
- Check for breaking changes
- Verify endpoint consistency
- Validate message schemas
- Check version compatibility
- Create Todo2 tasks for contract issues

**Cron Schedule**: `0 8 * * *` (8 AM daily)

**Output**: `docs/API_CONTRACT_VALIDATION_REPORT.md`

---

#### 5.2 Integration Status Monitoring

**Frequency**: Weekly
**Script**: New - `scripts/automate_integration_health.py`
**Purpose**: Monitor integration health

**Actions**:

- Check TWS API connectivity
- Verify NATS server status
- Test database connections
- Validate external API access
- Check integration test results
- Create Todo2 tasks for failures

**Cron Schedule**: `0 14 * * 1` (2 PM Mondays)

**Output**: `docs/INTEGRATION_HEALTH_REPORT.md`

---

## Implementation Priority

### Phase 1: Critical Security & Quality (Week 1)

1. ✅ Dependency Security Scan (highest priority)
2. ✅ Linter Automation (enhance existing)
3. ✅ Test Coverage Analysis

### Phase 2: Build & Infrastructure (Week 2)

4. Build Health Check
5. Secret Scanning
6. Code Metrics Dashboard

### Phase 3: Advanced Monitoring (Week 3-4)

7. Code Complexity Analysis
8. Dependency Update Check
9. Configuration Drift Detection
10. Git Repository Health

### Phase 4: Integration & API (Week 5+)

11. API Contract Validation
12. Integration Status Monitoring
13. File System Health Check

---

## Using IntelligentAutomationBase

All new automations should inherit from `IntelligentAutomationBase`:

```python
from scripts.base.intelligent_automation_base import IntelligentAutomationBase

class RepositoryHealthAnalyzer(IntelligentAutomationBase):
    def _get_tractatus_concept(self) -> str:
        return "What is repository health? Repo Health = Code Quality × Security × Build Success × Test Coverage × Documentation"

    def _get_sequential_problem(self) -> str:
        return "How do we systematically monitor repository health?"

    def _execute_analysis(self) -> Dict:
        # Core analysis logic
        return results

    def _generate_insights(self, results: Dict) -> str:
        # Generate insights
        return insights

    def _generate_report(self, results: Dict, insights: str) -> str:
        # Generate report
        return report
```

---

## Health Score Calculation

### Overall Repository Health Score

```
Health Score = (
    Code Quality Score × 0.3 +
    Security Score × 0.3 +
    Build Health Score × 0.2 +
    Test Coverage Score × 0.15 +
    Documentation Score × 0.05
) × 100
```

### Component Scores

- **Code Quality**: Based on linter issues, complexity, duplication
- **Security**: Based on vulnerabilities, secret detection, dependency security
- **Build Health**: Based on build success rate, build times, warnings
- **Test Coverage**: Based on coverage percentage, trend
- **Documentation**: Based on docs health (already automated)

---

## Existing Tools to Leverage

### Already Configured

- ✅ **Trunk**: Multi-language linting (bandit, ruff, clippy, shellcheck, etc.)
- ✅ **run_linters.sh**: C++ static analysis (cppcheck, clang-tidy)
- ✅ **osv-scanner**: Dependency vulnerability scanning
- ✅ **trufflehog**: Secret scanning
- ✅ **validate_docs_format.py**: API documentation validation

### To Install/Configure

- **pip-audit**: Python dependency security
- **cargo-audit**: Rust dependency security
- **cargo-outdated**: Rust dependency updates
- **lizard**: Code complexity analysis
- **gcov/lcov**: C++ test coverage
- **pytest-cov**: Python test coverage
- **cargo-tarpaulin**: Rust test coverage

---

## Success Metrics

### Automation Coverage

- **Target**: 80% of health checks automated
- **Current**: ~20% (docs health, TODO sync)
- **Goal**: Reach 80% in 4 weeks

### Issue Detection

- **Target**: Catch 90% of issues before they become critical
- **Measurement**: Track issues found by automation vs. manual

### Time Savings

- **Target**: Save 5+ hours/week on manual health checks
- **Measurement**: Track time spent on manual checks before/after

---

## Next Steps

1. **Week 1**: Implement Phase 1 (Security, Linters, Coverage)
2. **Week 2**: Implement Phase 2 (Build, Metrics, Secrets)
3. **Week 3-4**: Implement Phase 3 (Complexity, Updates, Config)
4. **Week 5+**: Implement Phase 4 (API, Integration, Filesystem)

---

## Monitoring & Alerts

### Health Score Thresholds

- **Critical**: < 60% - Immediate action required
- **Warning**: 60-80% - Attention needed
- **Good**: 80-90% - Healthy
- **Excellent**: > 90% - Optimal

### Alert Mechanisms

1. **Todo2 Tasks**: Create high-priority tasks for critical issues
2. **Reports**: Generate detailed reports in `docs/`
3. **Logs**: Log all operations to `scripts/*.log`
4. **Email** (optional): Send alerts for critical issues

---

*This plan provides comprehensive repository and codebase health monitoring automation.*
