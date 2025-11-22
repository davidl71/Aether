# Using @docs in Cursor IDE

## Overview

Cursor's `@docs` feature allows you to reference documentation files directly in your prompts, giving the AI assistant better context about the APIs and libraries you're using. This significantly improves code suggestions and reduces the need for clarification prompts.

## Benefits

1. **Better Context**: AI understands your specific API versions and usage patterns
2. **Accurate Suggestions**: Code suggestions match your actual dependencies
3. **Fewer Prompts**: Less back-and-forth asking about API details
4. **Consistent Patterns**: AI follows your documented patterns

## How to Use @docs

### Basic Syntax

In any Cursor prompt, reference documentation files using `@docs`:

```
@docs API_DOCUMENTATION_INDEX.md How do I implement error handling for TWS API?
```

### Multiple References

You can reference multiple files:

```
@docs API_DOCUMENTATION_INDEX.md @docs TWS_INTEGRATION_STATUS.md
How do I connect to TWS API in paper trading mode?
```

### Inline Code Comments

Add `@docs` references in your code comments:

```cpp
// @docs API_DOCUMENTATION_INDEX.md - TWS API EWrapper
class MyTWSClient : public DefaultEWrapper {
  // Implementation
};
```

## Available Documentation Files

### Core API Documentation

- **`API_DOCUMENTATION_INDEX.md`**: Complete index of all APIs and libraries
- **`TWS_INTEGRATION_STATUS.md`**: TWS API integration details
- **`EWRAPPER_STATUS.md`**: EWrapper implementation status

### Implementation Guides

- **`IMPLEMENTATION_GUIDE.md`**: Step-by-step implementation guide
- **`QUICK_START.md`**: Quick start guide
- **`WORKTREE_SETUP.md`**: Development environment setup

### Build & Configuration

- **`DISTRIBUTED_COMPILATION.md`**: Build optimization guide
- **`CURSOR_SETUP.md`**: Cursor IDE setup
- **`CURSOR_IGNORE_SETUP.md`**: File exclusion configuration

## Best Practices

### 1. Reference Specific Sections

Instead of referencing entire files, be specific:

```
@docs API_DOCUMENTATION_INDEX.md#spdlog How do I log errors?
```

### 2. Combine with Code Context

Reference docs alongside code:

```
@docs API_DOCUMENTATION_INDEX.md
Looking at native/src/tws_client.cpp, how should I handle connection errors?
```

### 3. Update Documentation

Keep documentation current:

- Update `API_DOCUMENTATION_INDEX.md` when adding dependencies
- Document API version changes
- Add usage examples for complex APIs

### 4. Use in .cursorrules

Reference docs in `.cursorrules` for project-wide context:

```markdown
## API References
- See `docs/API_DOCUMENTATION_INDEX.md` for all external APIs
- See `docs/TWS_INTEGRATION_STATUS.md` for TWS API details
```

## Examples

### Example 1: TWS API Connection

**Prompt:**

```
@docs API_DOCUMENTATION_INDEX.md @docs TWS_INTEGRATION_STATUS.md
How do I connect to TWS API on port 7497 (paper trading)?
```

**Expected Result:**
AI provides code using correct TWS API classes (`EClientSocket`, `DefaultEWrapper`) with paper trading port.

### Example 2: Logging

**Prompt:**

```
@docs API_DOCUMENTATION_INDEX.md#spdlog
How do I log a warning message?
```

**Expected Result:**
AI suggests `spdlog::warn()` with proper formatting.

### Example 3: Testing

**Prompt:**

```
@docs API_DOCUMENTATION_INDEX.md#Catch2
How do I write a test for the box spread calculator?
```

**Expected Result:**
AI provides Catch2 test structure matching your existing tests.

## Creating Documentation Files

### Structure

Documentation files should include:

1. **Overview**: What the API/library does
2. **Official Docs**: Link to official documentation
3. **Version**: Version used in project
4. **Key Classes/Functions**: Important APIs
5. **Usage Examples**: Code examples
6. **Location**: Where it's used in codebase

### Template

```markdown
# API Name

## Overview
Brief description of what this API does.

## Official Documentation
- **URL**: https://example.com/docs
- **Version**: 1.2.3

## Key Classes/Functions
- `ClassName`: Description
- `function_name()`: Description

## Usage Examples
\`\`\`cpp
// Example code
\`\`\`

## Location in Codebase
- Headers: `path/to/headers/`
- Implementation: `path/to/src/`
```

## Troubleshooting

### @docs Not Working

1. **Check File Path**: Ensure file exists in `docs/` directory
2. **Check Syntax**: Use `@docs filename.md` format
3. **Reload Cursor**: Restart Cursor if changes aren't recognized

### AI Not Using Documentation

1. **Be Explicit**: Mention specific sections
2. **Combine Context**: Reference docs + code together
3. **Update Docs**: Ensure documentation is current

## Next Steps

1. **Review**: Check `docs/API_DOCUMENTATION_INDEX.md` for your APIs
2. **Update**: Add any missing APIs or update versions
3. **Use**: Start using `@docs` in your prompts
4. **Iterate**: Refine documentation based on usage

## Related Documentation

- **`.cursorrules`**: Project-wide AI rules
- **`CURSOR_SETUP.md`**: Cursor IDE configuration
- **`CURSOR_IGNORE_SETUP.md`**: File exclusion setup
- **`API_DOCUMENTATION_INDEX.md`**: Complete API index
