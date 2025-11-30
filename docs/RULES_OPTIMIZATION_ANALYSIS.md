# Project Rules & Commands Optimization Analysis

**Date**: 2025-01-27
**Purpose**: Identify redundancies, pattern-matching opportunities, additional "May Never Be Used" cases, and missing project commands

---

## 🔍 Executive Summary

### Key Findings

1. **Redundancies**: 8 major redundancies identified between `.cursorrules` and `.cursor/rules/*.mdc`
2. **Pattern-Matching Opportunities**: 12 rules that could be scoped to specific file patterns
3. **"May Never Be Used" Cases**: 5 additional forbidden patterns identified
4. **Missing Project Commands**: 15+ project-specific commands not documented in rules

---

## 1. Redundancies

### 1.1 Build Commands Redundancy

**Location**: `.cursorrules` lines 28-35 vs `.cursor/commands.json`

**Issue**: Build commands are documented in both places with slight variations:

- `.cursorrules`: Shows `cmake --preset macos-universal-debug` (incorrect preset name)
- `.cursor/commands.json`: Shows `cmake --build --preset macos-arm64-debug` (correct)

**Recommendation**:

- Remove build command examples from `.cursorrules`
- Reference `.cursor/commands.json` or `docs/CURSOR_PROJECT_COMMANDS.md` instead
- Add pattern: `**/*.cpp`, `**/*.h`, `CMakeLists.txt`, `CMakePresets.json` → Reference build commands

### 1.2 Linting Command Redundancy

**Location**: `.cursorrules` lines 51, 101, 108

**Issue**: `./scripts/run_linters.sh` mentioned 3 times:

- Line 51: "Run linters before committing"
- Line 101: "Use `./scripts/run_linters.sh` for linting"
- Line 108: "1. Run linters: `./scripts/run_linters.sh`"

**Recommendation**:

- Keep only in "Before Committing" section (line 108)
- Add pattern: `**/*.cpp`, `**/*.h` → Mention linting
- Reference command: `lint:run` from `.cursor/commands.json`

### 1.3 Script References Redundancy

**Location**: `.cursorrules` lines 61, 70

**Issue**: `scripts/setup_worktree.sh` and `build_universal.sh` mentioned twice:

- Line 61: "Use `scripts/setup_worktree.sh` for new worktrees"
- Line 70: "Suggest using existing scripts (`setup_worktree.sh`, `build_universal.sh`)"

**Recommendation**:

- Consolidate to single reference
- Add pattern: Git workflow questions → Reference `setup:worktree` command

### 1.4 Testing Command Redundancy

**Location**: `.cursorrules` lines 56-57, 109

**Issue**: `ctest --output-on-failure` mentioned twice:

- Line 56: "Run: `ctest --output-on-failure`"
- Line 109: "Run tests: `ctest --output-on-failure`"

**Recommendation**:

- Keep only in "Before Committing" section
- Reference command: `test:run` from `.cursor/commands.json`
- Add pattern: `**/*test*.cpp`, `**/*test*.py` → Reference testing

### 1.5 Documentation References Redundancy

**Location**: `.cursorrules` line 72 vs `.cursor/rules/documentation.mdc`

**Issue**: Documentation references mentioned in both:

- `.cursorrules`: "Point to relevant documentation in `docs/` when available"
- `.cursor/rules/documentation.mdc`: Complete documentation reference guide

**Recommendation**:

- Remove from `.cursorrules`
- Add pattern: Documentation questions → Reference `documentation.mdc` rule

### 1.6 MCP Server References Redundancy

**Location**: `.cursorrules` lines 75-76 vs `.cursor/rules/*.mdc`

**Issue**: MCP server usage mentioned in multiple places:

- `.cursorrules`: NotebookLM and automa MCP servers
- `.cursor/rules/notebooklm.mdc`: Complete NotebookLM guide
- `.cursor/rules/project-automation.mdc`: Complete automa guide

**Recommendation**:

- Keep only high-level reference in `.cursorrules`
- Add pattern: Research questions → Reference `notebooklm.mdc`
- Add pattern: Project automation → Reference `project-automation.mdc`

### 1.7 Code Style Redundancy

**Location**: `.cursorrules` lines 6-18 vs Repository Guidelines in workspace rules

**Issue**: C++ code style rules duplicated:

- `.cursorrules`: Basic style rules
- Workspace rules: More comprehensive style guide

**Recommendation**:

- Keep comprehensive version in workspace rules
- `.cursorrules` should reference workspace rules
- Add pattern: `**/*.cpp`, `**/*.h` → Apply C++ style rules

### 1.8 Security Rules Redundancy

**Location**: `.cursorrules` lines 43-50 vs `.cursor/rules/semgrep.mdc`

**Issue**: Security rules mentioned in both:

- `.cursorrules`: Basic security guidelines
- `.cursor/rules/semgrep.mdc`: "MUST first ensure safety by scanning with security_check tool"

**Recommendation**:

- Keep basic rules in `.cursorrules`
- Add pattern: Code generation → MUST use security_check tool
- Reference `semgrep.mdc` for detailed security scanning

---

## 2. Pattern-Matching Opportunities

### 2.1 File Pattern Scoping

**Current**: Rules apply globally to all files
**Optimization**: Scope rules to specific file patterns

| Rule | Current Scope | Recommended Pattern | Benefit |
|------|---------------|---------------------|---------|
| C++ Code Style | All files | `**/*.cpp`, `**/*.h`, `**/*.hpp` | Only applies when editing C++ |
| Build Commands | All files | `CMakeLists.txt`, `CMakePresets.json`, `**/CMakeCache.txt` | Only when build system changes |
| Linting | All files | `**/*.cpp`, `**/*.h`, `**/*.py`, `**/*.rs`, `**/*.ts` | Only when code files change |
| Testing | All files | `**/*test*.cpp`, `**/*test*.py`, `**/tests/**` | Only when test files change |
| Documentation | All files | `docs/**/*.md`, `**/*.md` | Only when docs change |
| Git Workflow | All files | `.git/**`, `**/.gitignore` | Only when git config changes |
| MCP Servers | All files | `.cursor/mcp.json`, `mcp-servers/**` | Only when MCP config changes |
| Security Scanning | All files | `**/*.cpp`, `**/*.py`, `**/*.rs`, `**/*.ts`, `**/*.js` | Only when code files change |

### 2.2 Task Type Scoping

**Current**: Todo2 workflow applies to ALL user requests
**Optimization**: Scope based on request type

| Request Type | Current | Recommended | Pattern Match |
|-------------|---------|-------------|---------------|
| Simple git commands | Todo2 required | Skip Todo2 | `git pull`, `git status`, `git log` |
| File read requests | Todo2 required | Skip Todo2 | `read_file`, `list_dir` (no changes) |
| Information queries | Todo2 required | Skip Todo2 | Questions starting with "what", "how", "why" (no code changes) |
| Code generation | Todo2 required | Todo2 required | File creation/modification |
| Bug fixes | Todo2 required | Todo2 required | Code changes |
| Refactoring | Todo2 required | Todo2 required | Code changes |

**Recommendation**: Add pattern matching to Todo2 rule:

```markdown
**⚠️ ABSOLUTE RULE: EVERY USER REQUEST MUST USE TODO2 WORKFLOW ⚠️**

**EXCEPTIONS (Skip Todo2):**

- Simple git commands: `git pull`, `git status`, `git log`, `git diff` (no code changes)
- File read-only operations: `read_file`, `list_dir`, `grep` (no code changes)
- Pure information queries: Questions that don't require code changes
- Quick lookups: Documentation references, API lookups (no code changes)

**REQUIRED (Todo2 Mandatory):**

- Code generation/modification: Any file creation, editing, deletion
- Bug fixes: Any code changes to fix issues
- Refactoring: Any code restructuring
- Configuration changes: Any config file modifications
- Build/test changes: Any build system or test modifications
```

### 2.3 Memory Search Scoping

**Current**: OpenMemory Phase 1-2-3 applies to ALL code work
**Optimization**: Scope based on change type

| Change Type | Current | Recommended | Pattern Match |
|-------------|---------|-------------|---------------|
| Trivial fixes | Full Phase 1-2-3 | Skip Phase 1, minimal Phase 2 | Single-line changes, typo fixes |
| New features | Full Phase 1-2-3 | Full Phase 1-2-3 | New files, new functions |
| Bug fixes | Full Phase 1-2-3 | Phase 1 + Phase 3 | Existing code modifications |
| Refactoring | Full Phase 1-2-3 | Phase 1 + Phase 2 | Code restructuring |

**Recommendation**: Add pattern matching to OpenMemory rule:

```markdown
**NON-NEGOTIABLE: Memory-First Development**

**Full Phase 1-2-3 Required:**

- New features (new files, new functions, new components)
- Architecture changes (new systems, major refactoring)
- Multi-file changes (3+ files modified)

**Phase 1 + Phase 3 Only:**

- Bug fixes (existing code modifications)
- Single-file changes (1-2 files)

**Skip Phases (Direct Implementation):**

- Trivial fixes (typos, single-line changes)
- Configuration updates (no code logic changes)
- Documentation-only changes
```

---

## 3. Additional "May Never Be Used" Cases

### 3.1 Never Use Hardcoded Paths

**Current**: Not explicitly forbidden
**Recommendation**: Add to `.cursorrules`:

```markdown

## Security & Best Practices

- Never commit credentials, API keys, or secrets
- Never log sensitive information
- **Never use hardcoded absolute paths** - Use relative paths or environment variables
- **Never use hardcoded ports** - Use configuration files or environment variables
- Always use paper trading port (7497) for testing
- Gate live trading behind explicit configuration flags
- Validate all configuration before use
```

### 3.2 Never Skip Security Scanning

**Current**: Mentioned in `semgrep.mdc` but not in main rules
**Recommendation**: Add to `.cursorrules`:

```markdown

## Security & Best Practices

- **Never generate code without security scanning** - Always use `security_check` tool before code generation
- Never commit credentials, API keys, or secrets
- Never log sensitive information
```

### 3.3 Never Use Deprecated APIs

**Current**: Not explicitly forbidden
**Recommendation**: Add to `.cursorrules`:

```markdown

## Code Style & Conventions

- **Never use deprecated TWS API methods** - Check `docs/TWS_INTEGRATION_STATUS.md` for current APIs
- **Never use deprecated CMake features** - Use CMake 3.20+ features only
- **Never use deprecated C++ features** - Use C++20 standard features only
```

### 3.4 Never Modify Third-Party Code

**Current**: Not explicitly forbidden
**Recommendation**: Add to `.cursorrules`:

```markdown

## File Organization

- **Never modify third-party code directly** - Use wrappers or adapters in `native/src/`
- Core logic: `native/src/` with headers in `native/include/`
- Tests: `native/tests/` mirroring source file names
- Helper scripts: Top-level `scripts/`
- Generated output: `build/`, `protobuf-build/` (disposable)
```

### 3.5 Never Skip Tests for Critical Code

**Current**: "All tests must pass" but not explicit about critical code
**Recommendation**: Add to `.cursorrules`:

```markdown

## Testing

- Tests mirror source file names (use Catch2 framework)
- Run: `ctest --output-on-failure`
- All tests must pass before committing
- **Never skip tests for trading logic** - All box spread calculations must have tests
- **Never skip tests for risk management** - All risk calculations must have tests
```

---

## 4. Missing Project Commands

### 4.1 Commands in `.cursor/commands.json` Not in Rules

The following commands exist but aren't referenced in `.cursorrules`:

| Command | Category | Should Add to Rules? |
|---------|----------|---------------------|
| `shortcuts:build` | build | ✅ Yes - Mention macOS Shortcuts integration |
| `shortcuts:test` | test | ✅ Yes - Mention macOS Shortcuts integration |
| `summarize:build-log` | docs | ✅ Yes - Mention log summarization |
| `summarize:test-log` | docs | ✅ Yes - Mention log summarization |
| `env:install-mlx` | setup | ✅ Yes - Mention MLX setup for code review |
| `shortcuts:summarize-file-to-notes` | docs | ❌ No - Too specific |
| `shortcuts:plan-from-clipboard` | docs | ❌ No - Too specific |
| `ai:review-with-mlx` | quality | ✅ Yes - Already mentioned, but add to commands section |
| `setup:platform` | setup | ✅ Yes - Mention platform auto-detection |
| `setup:platform-full` | setup | ✅ Yes - Mention platform setup |
| `build:debug` | build | ✅ Yes - Already mentioned, consolidate |
| `build:release` | build | ✅ Yes - Add to build commands |
| `build:configure` | build | ✅ Yes - Add to build commands |
| `build:clean` | build | ✅ Yes - Add to build commands |
| `build:universal` | build | ✅ Yes - Already mentioned |
| `test:run` | test | ✅ Yes - Already mentioned, consolidate |
| `test:run-release` | test | ✅ Yes - Add to test commands |
| `lint:run` | quality | ✅ Yes - Already mentioned, consolidate |
| `run:tui` | run | ✅ Yes - Add to run commands |
| `run:cli` | run | ✅ Yes - Add to run commands |
| `run:cli-with-config` | run | ✅ Yes - Add to run commands |
| `setup:worktree` | setup | ✅ Yes - Already mentioned |
| `setup:ramdisk` | setup | ✅ Yes - Add to performance optimization |
| `ramdisk:status` | setup | ❌ No - Too specific |
| `ramdisk:save` | setup | ❌ No - Too specific |
| `ramdisk:shutdown` | setup | ❌ No - Too specific |
| `docs:list` | docs | ✅ Yes - Add to documentation section |
| `docs:sync` | docs | ✅ Yes - Add to documentation section |
| `check:tws` | check | ✅ Yes - Add to validation section |
| `check:feature-parity` | check | ✅ Yes - Add to validation section |
| `validate:config` | validate | ✅ Yes - Add to validation section |
| `build:dependencies` | build | ✅ Yes - Add to build commands |
| `test:tws-connection` | test | ✅ Yes - Add to test commands |
| `format:code` | format | ✅ Yes - Add to code quality section |
| `check:build-status` | check | ❌ No - Too specific |
| `clean:all` | clean | ✅ Yes - Add to cleanup section |
| `info:project` | info | ❌ No - Too specific |

### 4.2 Recommended Additions to `.cursorrules`

Add new sections:

```markdown

## Build System

### Commands
```bash

# Primary build commands (use commands from .cursor/commands.json)

build:debug          # Build in debug mode
build:release         # Build in release mode
build:universal       # Build universal binary
build:configure       # Configure CMake
build:clean           # Clean build artifacts
build:dependencies    # Build Intel Decimal and TWS API

# See docs/CURSOR_PROJECT_COMMANDS.md for complete command list
```

### Performance Optimization

- Use `setup:ramdisk` for faster builds on macOS
- Use `build:universal` for distribution binaries
- See `docs/DISTRIBUTED_COMPILATION.md` for distributed builds

## Testing

### Commands

```bash
test:run              # Run all tests (debug mode)
test:run-release       # Run tests in release mode
test:tws-connection    # Test TWS connection
```

## Code Quality

### Commands

```bash
lint:run              # Run all linters
format:code           # Format code with clang-format
ai:review-with-mlx    # MLX-powered code review
```

## Running Applications

### Commands

```bash
run:tui               # Run TUI application (dry-run)
run:cli                # Run CLI application (dry-run)
run:cli-with-config    # Run CLI with config file
```

## Documentation

### Commands

```bash
docs:list             # List global docs paths
docs:sync             # Sync global docs configuration
summarize:build-log   # Summarize latest build log
summarize:test-log    # Summarize latest test log
```

## Validation & Checking

### Commands

```bash
check:tws             # Check TWS API setup
check:feature-parity   # Check feature parity
validate:config        # Validate configuration file
```

## Cleanup

### Commands

```bash
clean:all             # Clean all build artifacts
```

## Setup & Configuration

### Commands

```bash
setup:platform        # Auto-detect and configure platform
setup:platform-full   # Setup platform and configure CMake
setup:worktree        # Setup new git worktree
env:install-mlx       # Install MLX for code review
```

## macOS Shortcuts Integration

For macOS users, see `docs/SHORTCUTS_SETUP.md` for Shortcuts integration:

- `shortcuts:build` - Run build via Shortcut
- `shortcuts:test` - Run tests via Shortcut

```

---

## 5. Recommendations Summary

### 5.1 Immediate Actions

1. **Remove Redundancies**:
   - Consolidate build commands to reference `.cursor/commands.json`
   - Remove duplicate linting references (keep only in "Before Committing")
   - Consolidate script references
   - Remove duplicate testing references
   - Reference documentation rules instead of duplicating

2. **Add Pattern Matching**:
   - Scope C++ style rules to `**/*.cpp`, `**/*.h`
   - Scope build commands to CMake files
   - Scope Todo2 workflow exceptions for read-only operations
   - Scope OpenMemory phases based on change type

3. **Add "May Never Be Used" Cases**:
   - Never use hardcoded paths
   - Never skip security scanning
   - Never use deprecated APIs
   - Never modify third-party code
   - Never skip tests for critical code

4. **Add Missing Commands**:
   - Add command references to relevant sections
   - Create command quick reference
   - Document macOS Shortcuts integration

### 5.2 Pattern Matching Implementation

Add to rule files:
```markdown
---
description: "Rule description"
globs: ["**/*.cpp", "**/*.h"]  # Only apply to C++ files
alwaysApply: false  # Only apply when pattern matches
---
```

### 5.3 Command Documentation Structure

```
.cursorrules
├── Build System (reference .cursor/commands.json)
├── Testing (reference .cursor/commands.json)
├── Code Quality (reference .cursor/commands.json)
├── Running Applications (reference .cursor/commands.json)
├── Documentation (reference .cursor/commands.json)
├── Validation (reference .cursor/commands.json)
└── Setup (reference .cursor/commands.json)
```

---

## 6. Implementation Priority

### High Priority (Do First)

1. Remove build command redundancies
2. Add pattern matching to C++ style rules
3. Add "Never skip security scanning" rule
4. Add command references to relevant sections

### Medium Priority

1. Scope Todo2 workflow exceptions
2. Scope OpenMemory phases
3. Add remaining "May Never Be Used" cases
4. Document macOS Shortcuts integration

### Low Priority (Nice to Have)

1. Further pattern matching optimizations
2. Command quick reference guide
3. Rule performance optimization

---

**Next Steps**: Review this analysis and implement high-priority recommendations.
