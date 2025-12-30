# Ollama Workflow Guide for IBKR Box Spread Project

**Status**: ✅ Active
**Date**: 2025-12-24
**Purpose**: Practical guide for using Ollama in daily development workflow

## Quick Start

### Prerequisites

- ✅ Ollama installed and running
- ✅ At least one model downloaded (llama3.2 recommended)
- ✅ MCP server configured in Cursor
- ✅ Cursor restarted after MCP configuration

### Verify Setup

```bash
# Check Ollama is running
ollama list

# Should show:
# NAME               ID              SIZE      MODIFIED
# llama3.2:latest    a80c4f17acd5    2.0 GB    [recent date]
```

## Common Workflows

### 1. Code Review Workflow

**When to use:** Before committing code, especially trading logic or risk management code.

**Steps:**

1. **Write or modify code** in your editor
2. **Open Cursor chat** and use Ollama:
   ```
   "Use Ollama to review this code for bugs, security issues, and best practices: [paste code or reference file]"
   ```
3. **Review Ollama's suggestions** - Always validate recommendations
4. **Address issues** identified by Ollama
5. **Run tests** to verify fixes
6. **Commit** with confidence

**Example:**
```
User: "Use Ollama to review native/src/box_spread_calc.cpp for potential bugs and security issues"

AI: [Uses Ollama MCP tool to analyze code]
    [Returns analysis with specific recommendations]
```

**Best Practices:**
- Be specific about what to review (bugs, security, performance, etc.)
- Include context about the code's purpose
- Always test suggestions before implementing
- Combine with human code review for critical code

### 2. Documentation Generation Workflow

**When to use:** When code lacks documentation or needs API documentation updates.

**Steps:**

1. **Identify code** that needs documentation
2. **Use Ollama** to generate documentation:
   ```
   "Generate API documentation for [module/function] using Ollama. Include usage examples and parameter descriptions."
   ```
3. **Review generated documentation** - Edit for accuracy and completeness
4. **Add to codebase** - Integrate into existing documentation structure
5. **Commit** documentation updates

**Example:**
```
User: "Generate API documentation for the calculate_box_spread_profit function using Ollama"

AI: [Uses Ollama to analyze function]
    [Generates comprehensive documentation with examples]
```

**Target Modules:**
- `native/src/box_spread_calc.cpp` - Core calculations
- `native/src/risk_calculator.cpp` - Risk management
- `native/src/order_manager.cpp` - Order execution
- `python/services/` - Python service layer

### 3. Code Explanation & Commenting Workflow

**When to use:** When complex trading mathematics or algorithms need explanation.

**Steps:**

1. **Identify complex code** that needs explanation
2. **Use Ollama** to explain and add comments:
   ```
   "Explain this calculation and add detailed comments using Ollama: [code snippet]"
   ```
3. **Review explanations** - Ensure accuracy
4. **Integrate comments** into code
5. **Verify** comments match implementation

**Example:**
```
User: "Explain this APR scaling calculation and add comments using Ollama: [code]"

AI: [Uses Ollama to explain the calculation]
    [Generates detailed comments explaining the math]
```

**Focus Areas:**
- Box spread pricing formulas
- Risk calculations (Greeks, convexity)
- Margin and capital efficiency calculations
- Complex mathematical transformations

### 4. Bug Detection Workflow

**When to use:** When debugging issues or before major releases.

**Steps:**

1. **Identify code** to analyze for bugs
2. **Use Ollama** to find potential issues:
   ```
   "Find potential bugs in this code using Ollama, focusing on [edge cases/race conditions/memory leaks]: [code]"
   ```
3. **Review findings** - Prioritize critical issues
4. **Fix bugs** identified
5. **Test fixes** thoroughly
6. **Document** fixes if needed

**Example:**
```
User: "Find potential bugs in the order execution path using Ollama, focusing on race conditions and error handling"

AI: [Uses Ollama to analyze code]
    [Identifies specific potential issues with explanations]
```

**Common Bug Types to Check:**
- Race conditions in async code
- Memory leaks in C++ code
- Edge cases in calculations
- Error handling gaps
- Resource cleanup issues

### 5. Performance Optimization Workflow

**When to use:** When performance issues are identified or optimizing critical paths.

**Steps:**

1. **Identify performance bottleneck**
2. **Use Ollama** to analyze and suggest optimizations:
   ```
   "Analyze this code for performance issues using Ollama and suggest optimizations: [code]"
   ```
3. **Review suggestions** - Evaluate trade-offs
4. **Implement optimizations** - Test carefully
5. **Benchmark** improvements
6. **Document** performance changes

**Example:**
```
User: "Optimize this market data processing loop using Ollama"

AI: [Uses Ollama to analyze code]
    [Suggests specific optimizations with explanations]
```

**Focus Areas:**
- Market data processing loops
- Calculation performance
- Memory allocation patterns
- I/O operations

### 6. Strategy Research Workflow

**When to use:** When researching trading strategies or market concepts.

**Steps:**

1. **Formulate research question**
2. **Use Ollama** to research:
   ```
   "Explain [trading concept] using Ollama, focusing on [specific aspect]"
   ```
3. **Review explanation** - Cross-reference with documentation
4. **Apply insights** to implementation
5. **Document** learnings

**Example:**
```
User: "Explain box spread arbitrage mechanics using Ollama, focusing on risk-free rate implications"

AI: [Uses Ollama to provide detailed explanation]
    [Includes relevant concepts and calculations]
```

**Research Topics:**
- Box spread construction and pricing
- Risk-free rate arbitrage
- Margin efficiency strategies
- Market microstructure concepts

## Query Templates

### Code Review Templates

**General Review:**
```
"Use Ollama to review [file/function] for bugs, security issues, and best practices"
```

**Security-Focused:**
```
"Analyze [code] for security vulnerabilities using Ollama, focusing on [input validation/authentication/data protection]"
```

**Performance-Focused:**
```
"Review [code] for performance issues using Ollama, focusing on [memory usage/algorithm efficiency/bottlenecks]"
```

### Documentation Templates

**API Documentation:**
```
"Generate API documentation for [function/module] using Ollama. Include parameter descriptions, return values, usage examples, and error conditions"
```

**Code Comments:**
```
"Add detailed comments to [function] explaining [specific aspect] using Ollama"
```

**Usage Examples:**
```
"Generate usage examples for [module] using Ollama, including common use cases and edge cases"
```

### Analysis Templates

**Bug Detection:**
```
"Find potential bugs in [code] using Ollama, focusing on [edge cases/race conditions/memory leaks/error handling]"
```

**Code Explanation:**
```
"Explain [calculation/algorithm] in [code] using Ollama, focusing on [mathematical concepts/implementation details]"
```

**Optimization:**
```
"Analyze [code] for performance issues using Ollama and suggest specific optimizations"
```

## Integration with Development Workflow

### Pre-Commit Workflow

**Optional Integration:**
```bash
# .git/hooks/pre-commit (optional)
#!/bin/bash
# Use Ollama to review code changes
# Generate documentation updates
```

**Manual Workflow:**
1. Make code changes
2. Use Ollama to review changes
3. Address issues
4. Commit

### Code Review Process

**Workflow:**
1. Developer creates PR
2. Use Ollama for initial automated review
3. Human reviewer focuses on trading logic correctness
4. Merge after both reviews

**Benefits:**
- Ollama catches style, security, documentation issues
- Human reviewer focuses on trading logic
- Faster review process
- Better code quality

### Documentation Maintenance

**Workflow:**
1. Code changes committed
2. Use Ollama to generate/update documentation
3. Review generated docs
4. Commit documentation updates

**Automation:**
- Can be integrated into CI/CD
- Auto-generate docs on code changes
- Maintain consistency across modules

## Model Selection Guide

### Current Model: llama3.2

**Best for:**
- ✅ General code analysis
- ✅ Documentation generation
- ✅ Quick questions
- ✅ Strategy research

**Limitations:**
- ⚠️ May miss some advanced code patterns
- ⚠️ Less specialized for code than codellama

### Recommended Additional Models

**For Code Analysis:**
```bash
ollama pull codellama
```
- Better understanding of code patterns
- Improved code generation and analysis
- Specialized for programming tasks

**For Documentation:**
```bash
ollama pull mistral
```
- Better natural language generation
- Improved documentation quality
- Good for explanations

**For Quick Tasks:**
```bash
ollama pull phi3
```
- Fast responses
- Lower resource usage
- Good for simple queries

## Best Practices

### 1. Privacy-First Approach

**Guidelines:**
- ✅ Always use Ollama for proprietary trading code
- ✅ Use Cursor AI for general development questions
- ✅ Never send trading strategies to cloud services
- ✅ Keep sensitive calculations local

**Decision Tree:**
```
Is this code proprietary/sensitive?
├─ Yes → Use Ollama
└─ No → Use Cursor AI (faster, more capable)
```

### 2. Query Optimization

**Tips:**
- **Be specific**: Include what you want analyzed (bugs, security, performance, etc.)
- **Provide context**: Mention the code's purpose and domain
- **Break down complex questions**: Split large analyses into focused queries
- **Review output**: Always validate Ollama's suggestions

**Example:**
```
❌ Bad: "Review this code"
✅ Good: "Review this box spread calculation function for potential bugs, focusing on edge cases, error handling, and numerical precision issues"
```

### 3. Always Validate

**Critical Rule:**
- ✅ Always review Ollama's suggestions
- ✅ Test all recommendations before implementing
- ✅ Verify accuracy of generated documentation
- ✅ Cross-reference with official documentation

**Why:**
- Models can make mistakes
- Context may be misunderstood
- Trading code requires precision
- Security is critical

### 4. Combine with Other Tools

**Tool Selection:**
- **Ollama**: Proprietary code analysis
- **Semgrep**: Security scanning
- **Context7**: Library documentation
- **NotebookLM**: Research synthesis
- **Cursor AI**: General development

## Troubleshooting

### Ollama Not Responding

**Symptoms:**
- MCP tools return errors
- Models not found
- Timeout errors

**Solutions:**
1. Check Ollama service:
   ```bash
   brew services list | grep ollama
   ollama list
   ```

2. Restart Ollama:
   ```bash
   brew services restart ollama
   ```

3. Verify API:
   ```bash
   curl http://localhost:11434/api/tags
   ```

### MCP Server Not Available

**Symptoms:**
- Ollama tools not appearing in Cursor
- "Server not found" errors

**Solutions:**
1. Restart Cursor completely (not just reload window)
2. Check `.cursor/mcp.json` configuration
3. Verify `uvx mcp-ollama` works:
   ```bash
   uvx mcp-ollama --help
   ```

### Model Not Found

**Symptoms:**
- "Model not found" errors
- Empty model list

**Solutions:**
1. List available models:
   ```bash
   ollama list
   ```

2. Pull missing model:
   ```bash
   ollama pull llama3.2
   ```

3. Verify installation:
   ```bash
   ollama show llama3.2
   ```

### Slow Performance

**Symptoms:**
- Long response times
- Timeout errors

**Solutions:**
1. Use smaller models for quick tasks (phi3)
2. Enable GPU acceleration (if available)
3. Close other resource-intensive applications
4. Consider using Cursor AI for non-sensitive tasks

## Performance Tips

### Model Selection

- **Quick tasks**: Use `llama3.2` or `phi3`
- **Code analysis**: Use `codellama` (when available)
- **Documentation**: Use `llama3.2` or `mistral` (when available)
- **Complex analysis**: Use `llama3.1` or `mistral` (when available)

### Query Optimization

- **Be specific**: Narrow queries get faster, better results
- **Provide context**: Include relevant code snippets
- **Break down**: Split large analyses into smaller queries
- **Cache results**: Save common analyses for reuse

### Resource Management

- **Monitor RAM**: Models consume 2-8GB per model
- **Clean up**: Remove unused models
- **GPU acceleration**: Enable if available
- **Batch queries**: Group similar queries together

## Examples

### Example 1: Code Review

**Scenario:** Reviewing a new box spread calculation function

**Query:**
```
"Use Ollama to review the calculate_box_spread_profit function in native/src/box_spread_calc.cpp for potential bugs, focusing on edge cases, numerical precision, and error handling"
```

**Expected Output:**
- List of potential issues
- Specific recommendations
- Code improvements
- Edge case handling suggestions

### Example 2: Documentation Generation

**Scenario:** Adding API documentation for risk calculator

**Query:**
```
"Generate comprehensive API documentation for the RiskCalculator class using Ollama. Include method descriptions, parameter details, return values, usage examples, and error conditions"
```

**Expected Output:**
- Complete API documentation
- Usage examples
- Parameter descriptions
- Error handling documentation

### Example 3: Code Explanation

**Scenario:** Explaining complex convexity calculation

**Query:**
```
"Explain the convexity calculation in the calculate_convexity function using Ollama. Add detailed comments explaining the mathematical concepts and implementation details"
```

**Expected Output:**
- Mathematical explanation
- Implementation details
- Detailed code comments
- Related concepts

## Next Steps

1. ✅ Read this workflow guide
2. ✅ Familiarize yourself with query templates
3. 🔄 Start using Ollama for code reviews
4. 🔄 Generate documentation for key modules
5. 📋 Evaluate additional models (codellama, mistral)
6. 📋 Create custom prompt templates for your team

## References

- **Technical Integration**: `docs/MCP_OLLAMA_INTEGRATION.md`
- **Integration Plan**: `docs/OLLAMA_INTEGRATION_PLAN.md`
- **Usage Guidelines**: `.cursor/rules/ollama.mdc`
- **Ollama Documentation**: https://ollama.ai/
- **MCP Specification**: https://modelcontextprotocol.io/

---

**Last Updated**: 2025-12-24
**Status**: Active Guide
**Maintained By**: Development Team
