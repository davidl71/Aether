# Prompt Tower Usage Guide

<!--
@index: cursor-ide
@category: guide
@tags: prompt-tower, cursor, ai-assistant, prompt-engineering
@last-updated: 2025-01-27
-->

This guide explains how to use the Prompt Tower extension in Cursor IDE to enhance your prompts and get better AI assistance for this project.

## Overview

**Prompt Tower** is a Cursor extension that enhances and refines your prompts, leading to more accurate and efficient AI-generated code. It's particularly valuable for complex projects like this one with extensive documentation and specific coding standards.

## Key Benefits

- **One-Click Enhancement**: Quickly refine prompts with a single click
- **Multiple Formats**: Support for JSON, XML, YAML, and plain text
- **System Prompt Access**: Utilize Cursor's actual system prompt for consistency
- **Multi-Language Support**: Works with prompts in various languages
- **Better Context**: Helps structure prompts that leverage your documentation

## Installation

1. **Install from Extensions**:
   - Open Cursor
   - Go to Extensions view (`Cmd+Shift+X`)
   - Search for "Prompt Tower"
   - Click Install

2. **Or Install via Command**:
   ```bash
   cursor --install-extension prompttower.prompttower
   ```

3. **Verify Installation**:
   - Check Extensions view - Prompt Tower should appear in installed extensions
   - Look for Prompt Tower icon in the Cursor interface

## Basic Usage

### Quick Enhancement

1. **Write your prompt** in the Cursor chat
2. **Click the Prompt Tower enhancement button** (usually appears near the prompt input)
3. **Review the enhanced prompt** - it will be more structured and detailed
4. **Use the enhanced version** for better AI responses

### Example: Before and After

**Before Enhancement**:
```
Fix the box spread calculation
```

**After Enhancement** (with Prompt Tower):
```
Review and fix the box spread calculation in native/src/box_spread_calc.cpp.
Ensure calculations match the formulas documented in docs/BOX_SPREAD_COMPREHENSIVE_GUIDE.md.
Follow C++20 standards, 2-space indentation, and Allman braces per .cursorrules.
Include proper error handling and validation as per COMMON_PATTERNS.md.
```

## Advanced Usage with This Project

### 1. Using with @docs References

Prompt Tower can help structure prompts that leverage your extensive documentation:

**Example Prompt**:
```
@docs API_DOCUMENTATION_INDEX.md @docs TWS_INTEGRATION_STATUS.md
How do I implement error handling for TWS API connection failures?
```

**Enhanced by Prompt Tower**:
```
Using the documentation in API_DOCUMENTATION_INDEX.md and TWS_INTEGRATION_STATUS.md,
implement comprehensive error handling for TWS API connection failures.

Requirements:
- Follow patterns in EWRAPPER_BEST_PRACTICES.md
- Use error handling patterns from COMMON_PATTERNS.md
- Ensure thread-safety per CODEBASE_ARCHITECTURE.md
- Include proper logging with spdlog
- Handle reconnection scenarios
```

### 2. Multi-File Refactoring Prompts

**Example**:
```
Refactor the order management system
```

**Enhanced**:
```
Refactor the order management system across:
- native/src/order_manager.cpp (C++ core)
- python/integration/strategy_runner.py (Python integration)

Requirements:
- Maintain consistency with CODEBASE_ARCHITECTURE.md
- Follow patterns in COMMON_PATTERNS.md
- Ensure all tests pass (29/29 tests)
- Update documentation if architecture changes
- Follow C++20 and Python 3.11+ standards
```

### 3. Complex Feature Implementation

**Example**:
```
Add risk management for box spreads
```

**Enhanced**:
```
Implement comprehensive risk management for box spread positions.

Reference Documentation:
- @docs BOX_SPREAD_COMPREHENSIVE_GUIDE.md (risk factors)
- @docs IMPLEMENTATION_GUIDE.md (implementation patterns)
- @docs COMMON_PATTERNS.md (coding patterns)

Implementation Requirements:
- Add VaR calculations per existing risk management patterns
- Implement position sizing limits
- Add real-time risk monitoring
- Follow C++20 standards with 2-space indentation
- Include comprehensive error handling
- Add unit tests following existing test patterns
- Update documentation in docs/
```

## Best Practices

### 1. Always Reference Documentation

**Good**:
```
@docs API_DOCUMENTATION_INDEX.md @docs TWS_INTEGRATION_STATUS.md
[Your prompt here]
```

**Better** (with Prompt Tower):
```
Using API_DOCUMENTATION_INDEX.md and TWS_INTEGRATION_STATUS.md as reference,
[Your detailed prompt with specific requirements]
```

### 2. Include Context About Project Standards

Always mention:
- **Code Style**: C++20, 2-space indentation, Allman braces
- **Architecture**: Reference CODEBASE_ARCHITECTURE.md
- **Patterns**: Reference COMMON_PATTERNS.md
- **Testing**: Ensure tests pass (29/29)
- **Documentation**: Update relevant docs

### 3. Be Specific About File Locations

**Good**:
```
Fix the calculation
```

**Better**:
```
Fix the box spread calculation in native/src/box_spread_calc.cpp,
ensuring it matches formulas in docs/BOX_SPREAD_COMPREHENSIVE_GUIDE.md
```

### 4. Reference Multiple Documentation Files

When working on complex features, reference multiple docs:
```
@docs CODEBASE_ARCHITECTURE.md @docs COMMON_PATTERNS.md @docs IMPLEMENTATION_GUIDE.md
[Your prompt]
```

### 5. Use Format-Specific Enhancement

For JSON/YAML config files:
- Prompt Tower can enhance prompts for configuration changes
- Helps structure prompts for complex config modifications
- Ensures proper format validation

## Integration with Project Workflow

### Development Workflow

1. **Write Initial Prompt**: Start with your basic idea
2. **Enhance with Prompt Tower**: Click enhancement button
3. **Add @docs References**: Include relevant documentation
4. **Review Enhanced Prompt**: Ensure it includes project context
5. **Submit**: Get better AI assistance

### Code Review Workflow

1. **Identify Issue**: What needs to be fixed/improved?
2. **Enhance Prompt**: Use Prompt Tower to structure review request
3. **Reference Standards**: Include relevant docs and patterns
4. **Get Suggestions**: AI provides code that follows your standards

### Documentation Updates

1. **Identify Gap**: What documentation needs updating?
2. **Enhance Prompt**: Structure request for documentation
3. **Reference Existing Docs**: Include related documentation
4. **Get Draft**: AI helps draft documentation following your format

## Common Use Cases

### Use Case 1: Implementing TWS API Feature

**Without Prompt Tower**:
```
Add market data subscription
```

**With Prompt Tower Enhancement**:
```
Implement market data subscription using TWS API.

Reference:
- @docs TWS_INTEGRATION_STATUS.md (API version 10.40.01)
- @docs EWRAPPER_BEST_PRACTICES.md (callback patterns)
- @docs COMMON_PATTERNS.md (error handling)

Requirements:
- Use EWrapper callbacks (tickPrice, tickSize)
- Implement proper error handling
- Follow thread-safe patterns
- Add logging with spdlog
- Ensure connection management
- Update TWS_INTEGRATION_STATUS.md when complete
```

### Use Case 2: Adding New Test

**Without Prompt Tower**:
```
Add test for box spread calculation
```

**With Prompt Tower Enhancement**:
```
Add comprehensive test for box spread calculation in native/tests/.

Reference:
- Existing test patterns in native/tests/box_spread_calc_test.cpp
- Test framework: Catch2
- Test naming: snake_case with descriptive names

Requirements:
- Test edge cases (zero width, negative prices)
- Test boundary conditions
- Follow existing test structure
- Ensure test passes with existing 29 tests
- Use descriptive test names
```

### Use Case 3: Refactoring Code

**Without Prompt Tower**:
```
Refactor the strategy runner
```

**With Prompt Tower Enhancement**:
```
Refactor python/integration/strategy_runner.py following NautilusTrader patterns.

Reference:
- @docs NAUTILUS_LEARNINGS.md (event-driven patterns)
- @docs COMMON_PATTERNS.md (Python patterns)
- @docs CODEBASE_ARCHITECTURE.md (system design)

Requirements:
- Implement event-driven architecture
- Add proper lifecycle management
- Follow Python 3.11+ standards
- Maintain backward compatibility
- Update all tests
- Update documentation
```

## Tips for Maximum Effectiveness

### 1. Start Simple, Enhance Later

- Write your basic prompt first
- Use Prompt Tower to enhance it
- Review and refine if needed

### 2. Combine with @docs

- Prompt Tower + @docs = Powerful combination
- Enhanced prompts that reference your docs
- Better context for AI assistant

### 3. Review Enhanced Prompts

- Don't blindly use enhanced prompts
- Review and adjust if needed
- Add project-specific context

### 4. Save Good Prompts

- Keep track of effective prompt patterns
- Reuse successful prompt structures
- Build a library of enhanced prompts

### 5. Iterate and Refine

- First response might need refinement
- Use Prompt Tower again on follow-up prompts
- Build on previous context

## Troubleshooting

### Prompt Tower Not Appearing

1. **Check Installation**:
   - Verify extension is installed
   - Check Extensions view
   - Restart Cursor if needed

2. **Check Extension Settings**:
   - Look for Prompt Tower in settings
   - Ensure it's enabled
   - Check for any error messages

### Enhanced Prompts Too Verbose

- Review enhanced prompt
- Remove unnecessary details
- Keep project-specific context
- Focus on essential requirements

### Enhanced Prompts Missing Context

- Manually add @docs references
- Include project-specific requirements
- Reference relevant documentation
- Add code style requirements

## Integration with Other Tools

### With @docs Feature

Prompt Tower works excellently with Cursor's `@docs` feature:
1. Write prompt with @docs references
2. Enhance with Prompt Tower
3. Get better structured prompts with documentation context

### With .cursorrules

Prompt Tower respects your `.cursorrules`:
- Enhanced prompts align with project standards
- References coding style guidelines
- Includes architecture patterns

### With MCP Servers

Prompt Tower complements MCP servers:
- Semgrep MCP: Security-focused prompts
- Filesystem MCP: File-aware prompts
- Git MCP: Version-aware prompts

## Examples for This Project

### Example 1: Box Spread Implementation

**Original Prompt**:
```
Implement box spread detection
```

**Enhanced Prompt**:
```
Implement comprehensive box spread detection system.

Documentation References:
- @docs BOX_SPREAD_COMPREHENSIVE_GUIDE.md (mechanics and strategies)
- @docs IMPLEMENTATION_GUIDE.md (step-by-step implementation)
- @docs COMMON_PATTERNS.md (coding patterns)

Implementation Requirements:
- Create box_spread_detector.cpp in native/src/
- Follow C++20 standards with 2-space indentation
- Use Allman braces for multi-line scopes
- Implement validation per BOX_SPREAD_COMPREHENSIVE_GUIDE.md
- Add comprehensive error handling
- Include unit tests in native/tests/
- Update documentation when complete
- Ensure all 29 tests pass
```

### Example 2: Python Integration

**Original Prompt**:
```
Add Python bindings for box spread calculation
```

**Enhanced Prompt**:
```
Add Python bindings for box spread calculation using Cython.

Documentation References:
- @docs IMPLEMENTATION_GUIDE.md (Python integration patterns)
- @docs COMMON_PATTERNS.md (Python coding standards)

Implementation Requirements:
- Create bindings in python/bindings/
- Follow Python 3.11+ standards
- Use Cython for C++ integration
- Match existing binding patterns
- Add Python tests
- Update python/README.md
- Ensure backward compatibility
```

### Example 3: Documentation Update

**Original Prompt**:
```
Update the API documentation
```

**Enhanced Prompt**:
```
Update API_DOCUMENTATION_INDEX.md with new TWS API features.

Reference:
- @docs TWS_INTEGRATION_STATUS.md (current status)
- @docs API_DOCUMENTATION_INDEX.md (existing format)

Requirements:
- Follow existing documentation format
- Include version information (TWS API 10.40.01)
- Add examples where appropriate
- Update cross-references
- Maintain consistency with other docs
```

## Conclusion

Prompt Tower is a valuable tool for this project because:

1. **Complex Codebase**: Helps structure prompts for complex trading system
2. **Extensive Documentation**: Works well with @docs feature
3. **Specific Standards**: Ensures prompts include project standards
4. **Multi-Language**: Supports C++, Python, Rust, Go, TypeScript
5. **Better Results**: More accurate AI assistance

By combining Prompt Tower with your existing documentation (`@docs`), coding standards (`.cursorrules`), and MCP servers, you'll get significantly better AI assistance for this complex trading system.

## See Also

- [CURSOR_SETUP.md](CURSOR_SETUP.md) - Complete Cursor IDE setup
- [CURSOR_RECOMMENDATIONS.md](CURSOR_RECOMMENDATIONS.md) - Extension recommendations
- [CURSOR_DOCS_USAGE.md](CURSOR_DOCS_USAGE.md) - Using @docs feature
- [CURSOR_AI_TUTORIAL.md](CURSOR_AI_TUTORIAL.md) - Cursor AI best practices
- [.cursorrules](../.cursorrules) - AI assistant guidelines
