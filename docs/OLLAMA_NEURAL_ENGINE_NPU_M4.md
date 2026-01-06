# Neural Engine (NPU) Support for Local LLMs on Apple M4

**Date**: 2025-12-24
**Hardware**: Apple M4 (16-core Neural Engine, 38 TOPS)
**Status**: ✅ **MLX Framework Available** | ❌ **Ollama Does Not Use NPU**

## Current Status

### Ollama: ❌ No Neural Engine Support

**Ollama does NOT use the Neural Engine (NPU) on Apple Silicon.**

**Current Ollama Status:**

- ✅ Uses **Metal GPU** acceleration (automatic, working)
- ❌ Does **not** use Neural Engine (NPU)
- 🔮 Future: Neural Engine support may come in future updates

**Why GPU is Still Excellent:**

- Metal GPU provides 50-70 tokens/second for 7B models
- Excellent performance for code analysis and documentation
- No configuration needed

### MLX Framework: ✅ Neural Engine Support

**MLX (Machine Learning for Apple Silicon) DOES use Neural Engine!**

**MLX Status:**

- ✅ Uses **Neural Engine** (NPU) for acceleration
- ✅ Uses **Metal GPU** as well (hybrid approach)
- ✅ Optimized specifically for Apple Silicon
- ✅ Available in your project

## What is MLX?

**MLX** is Apple's open-source array framework optimized for machine learning on Apple Silicon. It's designed to leverage:

- **Neural Engine (NPU)**: 16-core, 38 TOPS on M4
- **Metal GPU**: Unified memory architecture
- **CPU**: Fallback and coordination

**Key Features:**

- Efficient LLM inference and fine-tuning
- Automatic hardware optimization
- Python-based, easy to use
- Supports popular models (Mistral, Llama, Phi, etc.)

## MLX vs Ollama Comparison

| Feature                 | Ollama                   | MLX                           |
| ----------------------- | ------------------------ | ----------------------------- |
| **Neural Engine (NPU)** | ❌ No                     | ✅ Yes                         |
| **Metal GPU**           | ✅ Yes                    | ✅ Yes                         |
| **Ease of Use**         | ⭐⭐⭐⭐⭐ Excellent          | ⭐⭐⭐⭐ Very Good                |
| **Model Support**       | ⭐⭐⭐⭐⭐ Excellent          | ⭐⭐⭐⭐ Very Good                |
| **Performance**         | ⭐⭐⭐⭐ Very Good           | ⭐⭐⭐⭐⭐ Excellent               |
| **Installation**        | ✅ Installed              | ✅ Available                   |
| **Best For**            | General use, quick setup | Maximum performance, research |

## Installing MLX

**Check if MLX is installed:**

```bash
python3 -c "import mlx, mlx_lm; print('MLX installed')"
```

**Install MLX (if needed):**

```bash
# Using the project script
./scripts/install_mlx.sh

# Or manually
python3 -m pip install --user --upgrade mlx mlx-lm
```

**Verify installation:**

```bash
python3 -c "import mlx, mlx_lm; print('OK')"
```

## Using MLX for LLM Inference

### Basic Usage

**Python script example:**

```python
import mlx.core as mx
import mlx.nn as nn
from mlx_lm import load, generate

# Load a model (uses Neural Engine + GPU)
model, tokenizer = load("mlx-community/Mistral-7B-Instruct-v0.2")

# Generate text (automatically uses Neural Engine)
response = generate(
    model,
    tokenizer,
    prompt="Review this C++ code for bugs: [code]",
    max_tokens=256
)
print(response)
```

### Available MLX Models

**Popular models available for MLX:**

- `mlx-community/Mistral-7B-Instruct-v0.2`
- `mlx-community/Llama-3.2-3B-Instruct`
- `mlx-community/Phi-3-mini-4k-instruct`
- `mlx-community/Qwen2.5-7B-Instruct`

**List available models:**

```bash
# Search Hugging Face for MLX models
# https://huggingface.co/models?library=mlx
```

## Performance Comparison (M4)

### Expected Performance with Neural Engine

| Framework  | Hardware Used       | Tokens/Second (7B) | First Token |
| ---------- | ------------------- | ------------------ | ----------- |
| **Ollama** | Metal GPU only      | 50-70              | 2-3s        |
| **MLX**    | Neural Engine + GPU | 60-90+             | 1-2s        |

**MLX Benefits:**

- ✅ **Faster inference**: Neural Engine optimized for AI workloads
- ✅ **Lower latency**: Specialized hardware for matrix operations
- ✅ **Better efficiency**: Uses both NPU and GPU together
- ✅ **Higher throughput**: 20-30% faster than GPU-only

### Neural Engine Specifications (M4)

**Apple M4 Neural Engine:**

- **16 cores** dedicated to AI/ML tasks
- **38 TOPS** (trillions of operations per second)
- **Optimized for**: Matrix multiplication, neural network inference
- **Memory**: Shared with system (unified memory architecture)

**M5 Improvements (for reference):**

- **Neural Accelerators** in GPU (additional to Neural Engine)
- **27% performance boost** over M4
- **Higher memory bandwidth**

## MLX Tools in Your Project

**Existing MLX tools:**

1. **`python/tools/summarize_log_mlx.py`** - Summarize build/test logs
2. **`python/tools/summarize_file_mlx.py`** - Summarize files
3. **`python/tools/draft_plan_mlx.py`** - Draft plans using MLX

**Usage example:**

```bash
# Summarize a log file using MLX (Neural Engine)
python3 python/tools/summarize_log_mlx.py --path logs/build_latest.log

# Summarize a file
python3 python/tools/summarize_file_mlx.py --path docs/ARCHITECTURE.md
```

## When to Use MLX vs Ollama

### Use Ollama When

- ✅ **Quick setup**: Already installed and working
- ✅ **General tasks**: Code review, documentation, quick questions
- ✅ **Ease of use**: Simple CLI interface
- ✅ **Model variety**: Easy model switching
- ✅ **MCP integration**: Already integrated with Cursor

### Use MLX When

- ✅ **Maximum performance**: Need fastest inference
- ✅ **Research/development**: Custom model fine-tuning
- ✅ **Python integration**: Building Python-based tools
- ✅ **Neural Engine**: Want to leverage NPU acceleration
- ✅ **Advanced use cases**: Custom inference pipelines

## Hybrid Approach (Recommended)

**Best of both worlds:**

1. **Ollama for daily use:**
   - Code review via MCP integration
   - Quick documentation generation
   - General AI assistance

2. **MLX for performance-critical tasks:**
   - Large batch processing
   - Custom inference pipelines
   - Research and experimentation

**Example workflow:**

```bash
# Quick code review (Ollama via MCP)
"Use Ollama to review this code for bugs"

# Batch log summarization (MLX for performance)
python3 python/tools/summarize_log_mlx.py --path logs/*.log
```

## Verifying Neural Engine Usage

### Check MLX Hardware Usage

**Python script:**

```python
import mlx.core as mx

# Check available devices
print(f"Default device: {mx.default_device()}")
print(f"Available devices: {mx.metal.is_available()}")

# MLX automatically uses Neural Engine when available
# No explicit configuration needed
```

### Monitor with Activity Monitor

**During MLX inference:**

1. Open **Activity Monitor** → **Window** → **GPU History**
2. Run MLX model inference
3. Observe:
   - **Neural Engine activity**: May show as GPU activity
   - **Memory usage**: Model loading
   - **CPU usage**: Lower than GPU-only (coordination only)

**Note:** Neural Engine activity may appear as GPU activity in Activity Monitor, as both use unified memory architecture.

## Future: Neural Engine Support in Ollama

**Current Status:**

- ❌ Ollama does not use Neural Engine (as of 2025-12-24)
- 🔮 Feature requests submitted (GitHub issues #3898, #4817)
- 🔮 Core ML format support requested
- ⏳ No official timeline announced

**What to Watch:**

- Ollama GitHub issues for Neural Engine support
- Core ML integration announcements
- Performance improvements in future releases

**In the meantime:**

- ✅ Use **Ollama** for convenience (Metal GPU is excellent)
- ✅ Use **MLX** for maximum performance (Neural Engine + GPU)

## Summary

### Current Setup

**Ollama (Current):**

- ✅ Metal GPU acceleration (100% GPU)
- ✅ Excellent performance (50-70 tokens/sec)
- ✅ Easy to use, MCP integrated
- ❌ No Neural Engine support

**MLX (Available):**

- ✅ Neural Engine + Metal GPU
- ✅ Better performance (60-90+ tokens/sec)
- ✅ Python-based, more flexible
- ✅ Already in your project

### Recommendations

1. **Keep using Ollama** for daily tasks (already optimized)
2. **Try MLX** for performance-critical tasks
3. **Monitor Ollama updates** for future Neural Engine support
4. **Use both** - Ollama for convenience, MLX for performance

### Quick Start with MLX

```bash
# Install MLX (if not already installed)
./scripts/install_mlx.sh

# Test MLX
python3 -c "import mlx, mlx_lm; print('MLX ready!')"

# Use existing MLX tools
python3 python/tools/summarize_log_mlx.py --path logs/build_latest.log
```

---

**Last Updated**: 2025-12-24
**Hardware**: Apple M4 (16-core Neural Engine, 38 TOPS)
**Status**: MLX Available for Neural Engine Acceleration
