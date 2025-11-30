# Cursor Bugbot Integration Guide

**Date**: 2025-01-27
**Status**: ✅ Configuration Complete

---

## Overview

Cursor Bugbot is an AI-powered tool that automatically reviews GitHub pull requests (PRs), identifying bugs, security vulnerabilities, and code quality issues. It analyzes PR diffs and provides comments with explanations and suggested fixes.

**Key Features:**

- Automatic PR reviews on each update
- Manual triggering via comments
- Security vulnerability detection
- Code quality analysis
- Project-specific rule configuration

---

## Setup Instructions

### 1. Access Cursor Dashboard

1. Navigate to [cursor.com/dashboard](https://cursor.com/dashboard)
2. Log in with your Cursor account
3. Navigate to the **Bugbot** tab

### 2. Connect GitHub

1. In the Bugbot tab, click **"Connect GitHub"** (or **"Manage Connections"** if already connected)
2. Follow the GitHub installation flow:
   - Authorize Cursor to access your GitHub account
   - Select the organization or repositories where Bugbot should run
   - Grant necessary permissions

**Requirements:**

- Admin access to Cursor account
- Admin access to GitHub organization (for org-level installation)
- Or repository admin access (for repo-level installation)

### 3. Enable Bugbot on Repositories

1. Return to the Cursor dashboard
2. In the Bugbot tab, enable Bugbot on specific repositories:
   - Toggle repositories on/off from your installations list
   - Bugbot runs only on PRs you author (by default)

---

## Configuration Options

### Repository Settings

**Enable/Disable Bugbot:**

- Toggle Bugbot per repository from your installations list
- Bugbot runs only on PRs you author (default behavior)

### Personal Settings

**Review Triggers:**

- **Automatic**: Run on every PR update (default)
- **Manual Only**: Run only when mentioned by commenting `cursor review` or `bugbot run`
- **Once Per PR**: Run only once per PR, skipping subsequent commits

**Configuration Location:**

- Cursor Dashboard → Bugbot → Personal Settings

---

## Project-Specific Rules

### Configuration File

Create `.cursor/BUGBOT.md` in your project root to provide context for reviews.

**How It Works:**

- Bugbot includes the root `.cursor/BUGBOT.md` file
- Additional `.cursor/BUGBOT.md` files found while traversing upward from changed files are also included
- This allows directory-specific rules for different parts of the codebase

**Example Structure:**

```
.cursor/
  BUGBOT.md          # Root-level rules (always included)

native/src/
  .cursor/
    BUGBOT.md        # C++-specific rules (included when reviewing native/src/)

agents/backend/
  .cursor/
    BUGBOT.md        # Rust-specific rules (included when reviewing agents/backend/)
```

### Our Configuration

**Location**: `.cursor/BUGBOT.md`

**Contents:**

- Project overview and context
- Critical security requirements (trading software safety)
- Code style requirements (C++20, 2-space indent, Allman braces)
- Build system requirements
- Testing requirements
- Static analysis guidelines
- Common issues to flag
- Pre-commit checklist
- Multi-language considerations

**See**: `.cursor/BUGBOT.md` for complete configuration

---

## Usage

### Automatic Reviews

Bugbot automatically reviews PRs when:

- A new PR is created
- New commits are pushed to an existing PR
- PR is updated in any way

**Note**: Runs only on PRs you author (unless configured otherwise)

### Manual Triggering

Trigger Bugbot manually by commenting on a PR:

```
cursor review
```

Or:

```
bugbot run
```

### Verbose Mode

For detailed logs and troubleshooting:

```
cursor review verbose=true
```

Or:

```
bugbot run verbose=true
```

**Verbose Output Includes:**

- Detailed analysis logs
- Request ID for support
- Step-by-step review process

---

## Troubleshooting

### Bugbot Not Running

1. **Check Permissions:**
   - Verify Bugbot has necessary repository access
   - Ensure GitHub app is installed and enabled
   - Check organization/repository settings

2. **Verify Installation:**
   - Confirm GitHub app is installed: GitHub → Settings → Applications → Installed GitHub Apps
   - Check Cursor dashboard: Bugbot tab → Verify repository is enabled

3. **Check PR Author:**
   - Bugbot runs only on PRs you author (default)
   - Verify you're the PR author or adjust settings

### Bugbot Not Providing Useful Reviews

1. **Check Configuration:**
   - Verify `.cursor/BUGBOT.md` exists and is properly formatted
   - Ensure project-specific rules are clear and actionable
   - Review rules for completeness

2. **Use Verbose Mode:**
   - Comment `cursor review verbose=true` to see detailed analysis
   - Check request ID for support if issues persist

3. **Improve Rules:**
   - Add more specific rules to `.cursor/BUGBOT.md`
   - Include examples of good/bad code patterns
   - Specify project-specific requirements

### Getting Support

If Bugbot isn't functioning as expected:

1. **Use Verbose Mode:**
   - Comment `cursor review verbose=true` on a PR
   - Note the request ID from verbose output

2. **Report Issues:**
   - Include request ID when reporting issues
   - Provide PR link and description
   - Share relevant configuration files

3. **Check Documentation:**
   - [Cursor Bugbot Documentation](https://docs.cursor.com/en/bugbot)
   - Cursor Community Forum

---

## Pricing

### Pro Plan

- **Cost**: $40 per month
- **Features**: Unlimited Bugbot reviews
- **Limit**: Up to 200 PRs per month across all repositories

### Teams Plan

- **Cost**: $40 per user per month
- **Features**: Unlimited code reviews across all PRs
- **Usage**: Pooled usage across your team

**Note**: Check [Cursor Pricing](https://cursor.com/pricing) for current pricing and plan details.

---

## Best Practices

### Writing Effective Bugbot Rules

1. **Be Specific:**
   - Include concrete examples of good/bad code
   - Specify exact requirements (indentation, naming, etc.)
   - List common issues to flag

2. **Provide Context:**
   - Explain project structure and organization
   - Describe critical security requirements
   - Include build system and testing requirements

3. **Update Regularly:**
   - Keep rules current with project changes
   - Add new patterns as they're discovered
   - Remove obsolete requirements

### Review Workflow

1. **Create PR:**
   - Push changes to a branch
   - Create pull request on GitHub

2. **Wait for Review:**
   - Bugbot automatically reviews (if enabled)
   - Or trigger manually with `cursor review`

3. **Address Issues:**
   - Review Bugbot comments
   - Fix identified issues
   - Push updates to PR

4. **Re-review:**
   - Bugbot automatically reviews updates
   - Or trigger manually again

5. **Merge:**
   - Address all critical issues
   - Merge when ready

---

## Integration with Existing Workflows

### CI/CD Integration

Bugbot complements existing CI/CD workflows:

- **GitHub Actions**: Runs alongside CI checks
- **Pre-commit Hooks**: Works with existing validation
- **Static Analysis**: Complements linters and analyzers

**Workflow:**

1. Developer creates PR
2. Bugbot reviews code quality and security
3. CI/CD runs tests and builds
4. All checks must pass before merge

### Static Analysis Integration

Bugbot works alongside static analysis tools:

- **Semgrep**: Security scanning (via MCP server)
- **cppcheck**: C++ static analysis
- **Clang Static Analyzer**: Advanced C++ analysis
- **clang-tidy**: Code quality checks

**Best Practice**: Use Bugbot for AI-powered review, static analysis for automated checks.

---

## Examples

### Example 1: Security Issue Detection

**Bugbot Comment:**

```
⚠️ Security Issue: Hardcoded API Key Detected

File: native/src/market_data_client.cpp:42

Found hardcoded API key in source code. This is a security risk.

Recommendation:

- Move API key to environment variable
- Use secure configuration management
- Never commit credentials to repository
```

### Example 2: Code Style Issue

**Bugbot Comment:**

```
⚠️ Code Style: Incorrect Indentation

File: native/src/box_spread_calc.cpp:15

Line uses tabs instead of spaces. Project requires 2-space indentation.

Current:
    if (condition) {

Should be:
  if (condition) {
```

### Example 3: Missing Test Coverage

**Bugbot Comment:**

```
⚠️ Testing: Missing Test Coverage

File: native/src/order_manager.cpp

New function `execute_order()` has no corresponding test file.

Recommendation:

- Create test file: native/tests/order_manager_test.cpp
- Add test cases for all code paths
- Ensure tests pass before merging
```

---

## References

- **Cursor Bugbot Documentation**: [docs.cursor.com/en/bugbot](https://docs.cursor.com/en/bugbot)
- **Cursor Dashboard**: [cursor.com/dashboard](https://cursor.com/dashboard)
- **Project Bugbot Rules**: `.cursor/BUGBOT.md`
- **Cursor Setup Guide**: `docs/research/integration/CURSOR_SETUP.md`
- **Security Guidelines**: `.cursorrules` (Security & Best Practices section)

---

## Status

✅ **Configuration Complete**

- `.cursor/BUGBOT.md` created with project-specific rules
- Documentation created
- Ready for GitHub connection and activation

**Next Steps:**

1. Connect GitHub account in Cursor dashboard
2. Enable Bugbot on repository
3. Create a test PR to verify integration
4. Adjust rules in `.cursor/BUGBOT.md` as needed

---

**Last Updated**: 2025-01-27
**Maintained By**: Project Team
