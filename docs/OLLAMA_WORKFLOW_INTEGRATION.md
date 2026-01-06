# Ollama Workflow Integration

**Date**: 2025-12-24
**Status**: ✅ Integrated

## Overview

Ollama has been integrated into the development workflow for privacy-preserving code analysis and documentation generation. All code analysis happens locally on your machine - no code is sent to cloud services.

## Available Tools

### 1. Code Review (`ollama_code_review.py`)

**Purpose**: Review C++ code for security, correctness, and best practices using CodeLlama.

**Usage**:

```bash
# Review single file
python3 python/tools/ollama_code_review.py \
  --files native/src/risk_calculator.cpp \
  --model codellama:7b

# Review multiple files
python3 python/tools/ollama_code_review.py \
  --files native/src/risk_calculator.cpp native/src/order_manager.cpp \
  --model codellama:7b
```

**Features**:

- Privacy-preserving (all code stays local)
- Security-focused analysis
- Edge case detection
- Performance recommendations
- Best practices suggestions

**Best For**:

- Proprietary trading algorithms
- Risk management code
- Order execution logic
- Security-sensitive code

### 2. Documentation Generation (`ollama_documentation.py`)

**Purpose**: Generate comprehensive API documentation using Mistral.

**Usage**:

```bash
# Generate documentation
python3 python/tools/ollama_documentation.py \
  --file native/src/risk_calculator.cpp \
  --model mistral:7b \
  --output docs/RISK_CALCULATOR_API.md
```

**Features**:

- Comprehensive API documentation
- Usage examples
- Edge cases and error conditions
- Thread safety notes
- Performance considerations

**Best For**:

- Proprietary code documentation
- Internal API documentation
- Code that shouldn't leave your machine

## Cursor Commands

Two new commands have been added to `.cursor/commands.json`:

### `ai:review-with-ollama`

Review code using Ollama (privacy-preserving):

```
ai:review-with-ollama native/src/risk_calculator.cpp
```

### `ai:docs-with-ollama`

Generate documentation using Ollama:

```
ai:docs-with-ollama native/src/risk_calculator.cpp
```

## Workflow Integration

### Recommended Workflow

1. **For Sensitive Code**:
   - Use `ai:review-with-ollama` for code review
   - Use `ai:docs-with-ollama` for documentation
   - All analysis happens locally

2. **For General Code**:
   - Use Cursor AI for quick questions
   - Use Context7 for library documentation
   - Use Ollama for comprehensive analysis

3. **For Security Review**:
   - Use Ollama for initial analysis
   - Use Semgrep for security scanning
   - Combine both for comprehensive security review

### Model Selection

| Task            | Recommended Model | Why                                    |
| --------------- | ----------------- | -------------------------------------- |
| Code Review     | `codellama:7b`    | Code-specialized, better understanding |
| Documentation   | `mistral:7b`      | Better reasoning, comprehensive output |
| Quick Questions | `llama3.2`        | Fast, general purpose                  |

## Examples

### Example 1: Code Review

```bash
# Review risk calculator
python3 python/tools/ollama_code_review.py \
  --files native/src/risk_calculator.cpp \
  --model codellama:7b
```

**Output**: Actionable recommendations for security, correctness, and best practices.

### Example 2: Documentation Generation

```bash
# Generate API docs
python3 python/tools/ollama_documentation.py \
  --file native/src/risk_calculator.cpp \
  --model mistral:7b \
  --output docs/RISK_CALCULATOR_API.md
```

**Output**: Comprehensive markdown documentation with examples and notes.

## Benefits

1. **Privacy**: All code stays on your local machine
2. **Cost Savings**: No API costs for frequent analysis
3. **Offline Capability**: Works without internet connection
4. **Security**: Proprietary algorithms never transmitted to cloud
5. **Speed**: Fast analysis on local hardware (M4 optimized)

## Troubleshooting

### Ollama Not Running

```bash
# Check status
ollama list

# Start service
ollama serve
```

### Model Not Found

```bash
# List available models
ollama list

# Pull required model
ollama pull codellama:7b
ollama pull mistral:7b
```

### Script Errors

```bash
# Check Python dependencies
pip install requests

# Or with uv
uv pip install requests
```

## Integration with Other Tools

- **Before Semgrep**: Use Ollama for code analysis, then Semgrep for security scanning
- **With Context7**: Ollama for proprietary code, Context7 for library docs
- **With Cursor AI**: Ollama for sensitive code, Cursor AI for general questions
- **With MLX**: Ollama for quick reviews, MLX for deeper analysis

## Next Steps

1. ✅ Code review tool created
2. ✅ Documentation tool created
3. ✅ Cursor commands added
4. ⏭️ Test with actual code files
5. ⏭️ Integrate into pre-commit hooks (optional)
6. ⏭️ Add to CI/CD pipeline (optional)

---

**Reference**: See `.cursor/rules/ollama.mdc` for detailed usage guidelines.
