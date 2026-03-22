# Ollama MCP Server Integration

**Status**: ✅ Configured and Ready
**Date**: 2025-01-27
**MCP Server**: `mcp-ollama` (via `uvx`)

## Overview

Ollama MCP server integration enables local LLM capabilities within Cursor IDE for privacy-sensitive code analysis, documentation generation, and trading strategy research.

## Benefits for Trading Software Development

- **Privacy**: Keep proprietary trading strategies and code analysis completely local
- **Cost Savings**: No API costs for frequent code analysis or documentation tasks
- **Offline Capability**: Work with AI assistance even without internet connection
- **Hybrid Approach**: Use Ollama for sensitive/proprietary analysis, Cursor AI for general assistance

## Installation

### Prerequisites

1. **Ollama Installed**: ✅ Already installed via Homebrew

   ```bash
   brew services start ollama
   ```

2. **Model Available**: ✅ llama3.2 model installed

   ```bash
   ollama list
   ```

### MCP Configuration

The Ollama MCP server is configured in `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "ollama": {
      "command": "uvx",
      "args": [
        "mcp-ollama"
      ],
      "env": {
        "OLLAMA_BASE_URL": "http://localhost:11434"
      }
    }
  }
}
```

## Available Tools

The Ollama MCP server provides the following tools:

### Model Management

- **List Models**: View all available Ollama models
- **Pull Model**: Download new models from Ollama library
- **Model Details**: Get information about specific models

### Chat & Analysis

- **Chat with Model**: Interact with local LLM models
- **Code Analysis**: Analyze code for bugs, security issues, or optimization
- **Documentation Generation**: Generate documentation from code
- **Strategy Research**: Research trading strategies (with local data)

## Usage Examples

### In Cursor Chat

Once configured, you can use Ollama through Cursor's AI chat:

```
"Use Ollama to analyze this trading code for security issues"
"Generate documentation for the box spread calculator using Ollama"
"Review this risk management code with Ollama for potential bugs"
```

### Direct Ollama Usage

You can also use Ollama directly via CLI:

```bash
# Chat with model
ollama run llama3.2 "Explain box spread arbitrage"

# Analyze code
ollama run llama3.2 "Review this C++ code for memory leaks: [code]"

# Generate documentation
ollama run llama3.2 "Generate documentation for this function: [code]"
```

## Configuration Details

### Environment Variables

- `OLLAMA_BASE_URL`: Ollama API endpoint (default: `http://localhost:11434`)
  - Configured to use local Ollama instance
  - Can be changed if Ollama runs on different host/port
  - Uses `uvx` to run the `mcp-ollama` package (Python-based, requires Python 3.10+)

### Model Selection

Currently installed model:

- **llama3.2**: 2.0 GB, fast and efficient for code analysis

To install additional models:

```bash
ollama pull codellama    # Code-specific model
ollama pull mistral       # General purpose
ollama pull phi3          # Small, efficient model
```

## Integration with Other MCP Servers

Ollama works alongside other MCP servers:

- **Filesystem**: Read code files for analysis
- **Git**: Analyze code changes and commits
- **Semgrep**: Combine with security scanning
- **Context7**: Use alongside documentation lookup

## Troubleshooting

### Ollama Not Responding

1. **Check Ollama Service**:

   ```bash
   brew services list | grep ollama
   ollama list
   ```

2. **Restart Ollama**:

   ```bash
   brew services restart ollama
   ```

3. **Verify API Endpoint**:

   ```bash
   curl http://localhost:11434/api/tags
   ```

### MCP Server Not Appearing

1. **Restart Cursor**: Completely quit and restart Cursor (not just reload)
2. **Check MCP Logs**: Look for errors in Cursor's MCP server logs
3. **Verify Configuration**: Ensure `.cursor/mcp.json` syntax is correct

### Model Not Found

1. **List Available Models**:

   ```bash
   ollama list
   ```

2. **Pull Missing Model**:

   ```bash
   ollama pull llama3.2
   ```

## Best Practices

### When to Use Ollama

✅ **Use Ollama for:**

- Privacy-sensitive code analysis
- Proprietary trading strategy review
- Frequent code analysis (cost savings)
- Offline development work
- Custom model fine-tuning

✅ **Use Cursor AI for:**

- General coding assistance
- Quick questions and answers
- Real-time collaboration
- Cloud-based features

### Model Selection

- **Code Analysis**: `codellama` or `llama3.2`
- **Documentation**: `llama3.2` or `mistral`
- **General Chat**: `llama3.2` or `phi3` (faster)
- **Complex Analysis**: `llama3.1` or `mistral` (more capable)

## Performance Considerations

- **Hardware Requirements**: Requires sufficient RAM (2-8GB per model)
- **Inference Speed**: Local inference may be slower than cloud APIs
- **Model Size**: Larger models provide better results but require more resources
- **GPU Acceleration**: Ollama automatically uses GPU if available

## Security Notes

- **Local Execution**: All data stays on your machine
- **No Cloud Transmission**: Code never leaves your local environment
- **API Key Free**: No API keys or authentication required
- **Privacy First**: Perfect for proprietary trading algorithms

## References

- **Ollama Website**: https://ollama.ai/
- **Ollama GitHub**: https://github.com/ollama/ollama
- **MCP Server Package**: https://pypi.org/project/mcp-ollama/ (Python package, run via `uvx`)
- **Project Documentation**: See `docs/API_DOCUMENTATION_INDEX.md` for more details

## Next Steps

1. ✅ Ollama installed and running
2. ✅ llama3.2 model downloaded
3. ✅ MCP server configured
4. 🔄 **Restart Cursor** to activate the integration
5. 📝 Test the integration with a code analysis request

---

**Last Updated**: 2025-01-27
**Status**: Ready for use after Cursor restart
