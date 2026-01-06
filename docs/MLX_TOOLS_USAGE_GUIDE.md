# MLX Tools Usage Guide

**Date**: 2025-12-30
**Status**: ✅ MLX Installed and Available
**Hardware**: Apple M4 with Neural Engine + Metal GPU

## Overview

MLX (Machine Learning for Apple Silicon) provides on-device AI capabilities using Apple's Neural Engine and Metal GPU. This guide shows how to use the MLX tools available in this project.

## Available MLX Tools

### 1. Log Summarization (`summarize_log_mlx.py`)

**Purpose**: Summarize build/test logs using MLX

**Usage**:

```bash
# Summarize build log
python3 python/tools/summarize_log_mlx.py --path logs/build_latest.log

# Summarize test log
python3 python/tools/summarize_log_mlx.py --path logs/tests_latest.log

# Custom model and token limit
python3 python/tools/summarize_log_mlx.py \
  --path logs/build_latest.log \
  --model mlx-community/Mistral-7B-Instruct-v0.2 \
  --max_tokens 512
```

**Features**:

- Automatically detects test vs build logs
- Falls back to heuristic summary if MLX unavailable
- Highlights errors, warnings, and failing tests
- Provides actionable next steps

**Example Output**:

```
Heuristic Summary
==================
Total lines: 240
Errors: 1
Warnings: 21

Top error lines:
- ninja: error: missing libtwsapi.dylib

Top warning lines:
- CMake Warning (dev)...
```

### 2. File Summarization (`summarize_file_mlx.py`)

**Purpose**: Summarize any text/markdown file using MLX

**Usage**:

```bash
# Summarize a file
python3 python/tools/summarize_file_mlx.py --path README.md

# Read from stdin
cat docs/ARCHITECTURE.md | python3 python/tools/summarize_file_mlx.py

# Custom settings
python3 python/tools/summarize_file_mlx.py \
  --path docs/API_DOCUMENTATION_INDEX.md \
  --max_tokens 256
```

**Use Cases**:

- Quick documentation summaries
- Understanding large code files
- Extracting key points from research documents

### 3. Code Review (`diffucode_review.py`)

**Purpose**: Review C++/CMake code using MLX models (e.g., DiffuCode-7B-cpGRPO)

**Usage**:

```bash
# Review single file
python3 python/tools/diffucode_review.py \
  --files native/src/box_spread_calc.cpp

# Review multiple files
python3 python/tools/diffucode_review.py \
  --files native/src/box_spread_strategy.cpp \
           native/src/order_manager.cpp \
           native/src/tws_client.cpp \
           CMakePresets.json

# Custom model
export MLX_MODEL=DiffuCode-7B-cpGRPO
python3 python/tools/diffucode_review.py \
  --files native/src/risk_calculator.cpp
```

**Review Focus Areas**:

- Correctness and race conditions
- Error handling patterns
- Performance optimizations
- Cross-platform build hygiene
- Code style alignment
- Security and validation

**Output Format**:

- Top 5-10 Recommendations
- Quick Wins
- Potential Risks

### 4. Plan Drafting (`draft_plan_mlx.py`)

**Purpose**: Generate implementation plans from specifications

**Usage**:

```bash
# From file
python3 python/tools/draft_plan_mlx.py --path spec.md

# From clipboard (macOS)
pbpaste | python3 python/tools/draft_plan_mlx.py

# Custom model
python3 python/tools/draft_plan_mlx.py \
  --path docs/research/architecture/BANK_LOAN_POSITION_SYSTEM_DESIGN.md \
  --max_tokens 512
```

**Output Structure**:

- Objectives (bullets)
- Constraints/Risks (bullets)
- Milestones (3-6 items)
- Tasks (up to 10, short imperative sentences)
- Validation/Success Criteria (bullets)

## MLX via MCP (mcp-stdio-tools)

After restarting Cursor, you can use MLX through the MCP server:

### Available Actions

1. **Status Check**:

   ```
   mcp_mcp-stdio-tools_mlx(action="status")
   ```

2. **Hardware Info**:

   ```
   mcp_mcp-stdio-tools_mlx(action="hardware")
   ```

3. **List Models**:

   ```
   mcp_mcp-stdio-tools_mlx(action="models")
   ```

4. **Generate Text**:

   ```
   mcp_mcp-stdio-tools_mlx(
     action="generate",
     prompt="Review this C++ code for potential bugs...",
     model="mlx-community/Mistral-7B-Instruct-v0.2",
     max_tokens=512
   )
   ```

## Available MLX Models

### Recommended Models

1. **Mistral-7B-Instruct-v0.2** (Default)
   - Good for: General tasks, summarization, code review
   - Size: ~4.4 GB
   - Best for: Most use cases

2. **DiffuCode-7B-cpGRPO** (Code Review)
   - Good for: Specialized code review
   - Size: ~7 GB
   - Best for: Detailed code analysis

3. **Llama-3.2-3B-Instruct**
   - Good for: Quick tasks, smaller context
   - Size: ~2 GB
   - Best for: Fast responses

### Model Installation

Models are automatically downloaded on first use. To pre-download:

```bash
python3 -c "from mlx_lm import load; load('mlx-community/Mistral-7B-Instruct-v0.2')"
```

## Performance

### Expected Performance (M4)

| Task | Tokens/Second | First Token |
|------|---------------|-------------|
| **Log Summarization** | 60-90 | 1-2s |
| **Code Review** | 50-70 | 2-3s |
| **File Summarization** | 60-90 | 1-2s |

### Hardware Usage

- **Neural Engine**: Automatically used for acceleration
- **Metal GPU**: Used for matrix operations
- **Memory**: Models loaded into unified memory
- **CPU**: Minimal (coordination only)

## Troubleshooting

### Model Download Issues

If models fail to download:

```bash
# Check Hugging Face cache
ls -la ~/.cache/huggingface/hub/models--mlx-community--*

# Clear cache and retry
rm -rf ~/.cache/huggingface/hub/models--mlx-community--*
python3 python/tools/summarize_log_mlx.py --path logs/build_latest.log
```

### MLX Not Available

If tools fall back to heuristic summaries:

1. **Check Installation**:

   ```bash
   python3 -c "import mlx; import mlx_lm; print('✅ MLX installed')"
   ```

2. **Check Metal GPU**:

   ```bash
   python3 -c "import mlx.core as mx; print(f'Metal: {mx.metal.is_available()}')"
   ```

3. **Reinstall if needed**:

   ```bash
   pip install --upgrade mlx mlx-lm
   ```

### MCP Server Not Detecting MLX

If `mcp_mcp-stdio-tools_mlx` reports MLX not installed:

1. **Restart Cursor completely** (Cmd+Q, then reopen)
2. **Verify mcp-stdio-tools has MLX**:

   ```bash
   cd /Users/davidl/Projects/mcp-stdio-tools
   uv run python -c "import mlx; print('✅ MLX in uv env')"
   ```

## Integration with Project Commands

### Cursor Commands

The following Cursor commands use MLX:

- `summarize:build-log` - Uses MLX to summarize build logs
- `summarize:test-log` - Uses MLX to summarize test logs
- `ai:review-with-mlx` - Uses MLX for code review

### Example Workflow

```bash
# 1. Build project
cmake --build --preset macos-arm64-debug

# 2. Summarize build log with MLX
cursor run summarize:build-log

# 3. Review code changes with MLX
cursor run ai:review-with-mlx --arg "native/src/box_spread_calc.cpp"

# 4. Generate plan from spec
python3 python/tools/draft_plan_mlx.py --path docs/TASK_SPEC.md
```

## Best Practices

1. **Use Appropriate Models**:
   - General tasks: Mistral-7B-Instruct
   - Code review: DiffuCode-7B-cpGRPO
   - Quick summaries: Llama-3.2-3B

2. **Token Limits**:
   - Log summaries: 256-512 tokens
   - Code reviews: 512-1024 tokens
   - Plans: 256-512 tokens

3. **File Size**:
   - Large files are automatically truncated
   - Focus on specific sections for better results

4. **Privacy**:
   - All processing is local (no data sent to cloud)
   - Perfect for proprietary trading code
   - Use for sensitive financial calculations

## Comparison: MLX vs Ollama

| Feature | MLX | Ollama |
|---------|-----|--------|
| **Neural Engine** | ✅ Yes | ❌ No |
| **Performance** | 60-90 tok/s | 50-70 tok/s |
| **Setup** | Requires model download | Pre-installed models |
| **Best For** | Performance-critical tasks | Quick tasks, convenience |
| **MCP Integration** | Via mcp-stdio-tools | Direct MCP server |

## Next Steps

1. **Test MLX Tools**:

   ```bash
   python3 python/tools/summarize_log_mlx.py --path logs/build_latest.log
   ```

2. **Restart Cursor** (to enable MCP MLX tool):
   - Quit Cursor (Cmd+Q)
   - Reopen Cursor
   - Test: `mcp_mcp-stdio-tools_mlx(action="status")`

3. **Download Models** (optional):

   ```bash
   python3 -c "from mlx_lm import load; load('mlx-community/Mistral-7B-Instruct-v0.2')"
   ```

---

**Last Updated**: 2025-12-30
**Status**: ✅ MLX Tools Ready for Use
