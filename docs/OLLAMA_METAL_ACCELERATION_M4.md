# Ollama Metal GPU Acceleration on Apple M4

**Date**: 2025-12-24
**Hardware**: Apple M4
**Status**: ✅ **Metal GPU Acceleration Active**

## Current Status

**Your Ollama is already using Metal GPU acceleration!**

Verify with:
```bash
ollama ps
```

**Example Output:**
```
NAME          ID              SIZE      PROCESSOR    CONTEXT    UNTIL
mistral:7b    6577803aa9a0    4.9 GB    100% GPU     4096       3 minutes from now
```

The **`100% GPU`** indicator confirms Metal acceleration is active.

## How Metal Acceleration Works

### Automatic Detection

Ollama **automatically detects and uses Metal** on Apple Silicon (M1, M2, M3, M4) devices. **No configuration required!**

**What happens:**
1. Ollama detects Apple Silicon on startup
2. Automatically enables Metal GPU acceleration
3. Uses GPU for model inference (much faster than CPU)
4. Falls back to CPU if GPU unavailable

### M4 GPU Architecture

**Apple M4 GPU:**
- **10-core GPU** (M4 Pro/Max have more cores)
- **Metal Performance Shaders (MPS)** backend
- **Unified Memory Architecture** (GPU and CPU share memory)
- **Automatic optimization** for neural network operations

**Benefits:**
- ✅ **50-100+ tokens/second** for 7B models
- ✅ **Lower latency** than CPU-only inference
- ✅ **Efficient memory usage** (unified memory)
- ✅ **No configuration needed** (automatic)

## Verifying GPU Acceleration

### Method 1: Check Ollama Status

```bash
ollama ps
```

**Look for:**
- `PROCESSOR` column showing `100% GPU` or `GPU`
- If it shows `CPU`, GPU acceleration is not active

### Method 2: Activity Monitor (macOS)

1. **Open Activity Monitor** (Applications → Utilities)
2. **Window → GPU History** (or press `Cmd+4`)
3. **Run a model:**
   ```bash
   ollama run codellama:7b "Hello, how are you?"
   ```
4. **Observe GPU usage:**
   - GPU graph should show activity
   - CPU usage should be relatively low
   - Memory usage will increase (model loading)

**Expected Behavior:**
- **GPU Usage**: 50-100% during inference
- **CPU Usage**: 10-30% (coordination, not computation)
- **Memory**: 4-6 GB per 7B model

### Method 3: System Information

```bash
# Check Metal support
system_profiler SPDisplaysDataType | grep -i metal
```

**Expected Output:**
```
Metal: Supported, feature set macOS GPUFamily2 v1
```

### Method 4: Performance Monitoring

**During model inference, you should see:**
- **Fast response times**: 10-30 seconds for 7B models
- **High GPU utilization**: 80-100% during inference
- **Low CPU usage**: <30% during inference

**If GPU is NOT working:**
- Response times: 60+ seconds for 7B models
- High CPU usage: 80-100%
- Low GPU usage: <10%

## Optimization Tips

### 1. Keep Models Loaded

**Pre-load frequently used models:**
```bash
# Load model into GPU memory
ollama run codellama:7b "test"
# Keep terminal open or run in background
```

**Benefits:**
- ✅ Faster first response (no loading time)
- ✅ GPU memory stays allocated
- ✅ Better performance for repeated queries

**Check loaded models:**
```bash
ollama ps
```

### 2. Use Appropriate Model Sizes

**For M4 (10-core GPU):**
- ✅ **7B models**: Optimal (codellama:7b, mistral:7b)
- ✅ **3-4B models**: Very fast (llama3.2, phi3:mini)
- ⚠️ **13B+ models**: May be slower, use more memory
- ❌ **70B models**: Not recommended (too large for M4)

**Current Setup (Optimal):**
- codellama:7b (3.8 GB) - Code analysis
- mistral:7b (4.4 GB) - Documentation
- llama3.2 (2.0 GB) - Quick tasks

### 3. Monitor GPU Memory

**Check GPU memory usage:**
```bash
# Activity Monitor → GPU History
# Or use system_profiler
system_profiler SPDisplaysDataType
```

**M4 GPU Memory:**
- **Shared with system RAM** (unified memory)
- **Typical allocation**: 4-6 GB per 7B model
- **Total system RAM**: Check with `sysctl hw.memsize`

**Optimization:**
- Unload unused models: `ollama ps` → let models timeout
- Use smaller models for quick tasks
- Close other GPU-intensive apps if needed

### 4. Environment Variables (Advanced)

**Ollama automatically uses Metal, but you can verify:**

```bash
# Check Ollama version (should be recent for best Metal support)
ollama --version

# Verify Metal backend
# (Ollama doesn't expose this directly, but Activity Monitor confirms)
```

**No environment variables needed** - Metal is automatic!

### 5. Update Ollama Regularly

**Keep Ollama updated for best Metal performance:**
```bash
brew upgrade ollama
```

**Recent updates improve:**
- Metal backend optimization
- Memory efficiency
- Model loading speed

## Troubleshooting

### GPU Not Being Used

**Symptoms:**
- `ollama ps` shows `CPU` instead of `GPU`
- Slow inference times (60+ seconds)
- High CPU usage, low GPU usage

**Solutions:**

1. **Check Ollama version:**
   ```bash
   ollama --version
   # Should be 0.1.0+ for Metal support
   ```

2. **Restart Ollama:**
   ```bash
   brew services restart ollama
   ```

3. **Verify macOS version:**
   - Metal requires macOS 12.0+ (Monterey)
   - M4 requires macOS 14.0+ (Sonoma)
   - Check: `sw_vers`

4. **Check for conflicts:**
   - Close other GPU-intensive apps
   - Check Activity Monitor for GPU usage

5. **Reinstall Ollama:**
   ```bash
   brew uninstall ollama
   brew install ollama
   brew services start ollama
   ```

### Slow Performance

**If GPU is active but still slow:**

1. **Check model size:**
   - 7B models should be fast (10-30s)
   - 13B+ models will be slower

2. **Monitor system resources:**
   - Close other apps
   - Check available RAM
   - Check thermal throttling (Activity Monitor → Energy)

3. **Use smaller models for quick tasks:**
   - llama3.2 (2.0 GB) for fast responses
   - codellama:7b (3.8 GB) for code analysis

### Memory Issues

**If models fail to load:**

1. **Check available RAM:**
   ```bash
   sysctl hw.memsize
   # Divide by 1024^3 for GB
   ```

2. **Unload other models:**
   ```bash
   ollama ps
   # Let models timeout or restart Ollama
   ```

3. **Use smaller models:**
   - llama3.2 instead of larger models
   - phi3:mini for very quick tasks

## Performance Benchmarks (M4)

### Expected Performance

| Model            | Size   | Tokens/Second | First Token | Full Response (100 tokens) |
| ---------------- | ------ | ------------- | ----------- | -------------------------- |
| **llama3.2**     | 2.0 GB | 80-100+       | <1s         | 1-2s                       |
| **codellama:7b** | 3.8 GB | 50-70         | 2-3s        | 10-20s                     |
| **mistral:7b**   | 4.4 GB | 50-70         | 2-3s        | 10-20s                     |

**Note:** Performance varies based on:
- Query complexity
- Context length
- System load
- Thermal state

### GPU Utilization

**During inference:**
- **GPU Usage**: 80-100%
- **CPU Usage**: 10-30%
- **Memory**: 4-6 GB per 7B model

**Idle (model loaded):**
- **GPU Usage**: 0-5%
- **CPU Usage**: <5%
- **Memory**: 4-6 GB (model in memory)

## Advanced: Neural Engine

**Apple M4 also has Neural Engine (16-core):**

**Current Status:**
- ❌ Ollama does **not** use Neural Engine (as of 2025-12-24)
- ✅ Uses **Metal GPU** instead (which is excellent)
- 🔮 Future: Neural Engine support may come in future Ollama updates

**Why GPU is Better for LLMs:**
- **More flexible**: Supports all model architectures
- **Better performance**: Optimized for large matrix operations
- **Proven**: Metal GPU acceleration is mature and stable

## Summary

### ✅ What's Working

- **Metal GPU acceleration**: ✅ Active (100% GPU)
- **Automatic detection**: ✅ No configuration needed
- **Optimal models**: ✅ 7B models work great on M4
- **Performance**: ✅ 50-70 tokens/second for 7B models

### 🎯 Recommendations

1. **Keep current setup**: All models using GPU acceleration
2. **Monitor with Activity Monitor**: Verify GPU usage visually
3. **Pre-load models**: Keep frequently used models in memory
4. **Update regularly**: `brew upgrade ollama` for latest optimizations

### 📊 Verification Commands

```bash
# Check GPU usage
ollama ps

# Monitor GPU activity
# Activity Monitor → Window → GPU History

# Check system info
system_profiler SPDisplaysDataType | grep -i metal
```

---

**Last Updated**: 2025-12-24
**Hardware**: Apple M4
**Status**: Metal GPU Acceleration Active and Optimized
