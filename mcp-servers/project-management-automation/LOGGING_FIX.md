# MCP Server Logging Fix

## Issue

MCP server was logging INFO level messages that appeared with `[error]` severity in Cursor's MCP logs:

**Example 1:**
```
2025-11-23 16:54:31.861 [error] 2025-11-23 16:54:31,859 - mcp.server.lowlevel.server - INFO - Processing request of type ListResourcesRequest
```

**Example 2:**
```
2025-11-23 16:59:11.391 [error] [11/23/25 16:59:11] INFO     Starting MCP server 'Project Management Automation' with transport 'stdio'
```

**Example 3:**
```
2025-11-23 17:05:24.845 [error] 2025-11-23 17:05:24,845 - __main__ - INFO - FastMCP server initialized
2025-11-23 17:05:24.851 [error] 2025-11-23 17:05:24,851 - __main__ - INFO - All tools loaded successfully
```

**Example 4:**
```
2025-11-23 17:05:24.914 [error]
╭──────────────────────────────────────────────────────────────────────────────╮
│                         FastMCP 2.13.1 Banner                                │
╰──────────────────────────────────────────────────────────────────────────────╯
```

## Root Cause

**Cursor interprets ALL stderr output as errors**, even if it's just INFO level logs. The problem has multiple sources:

1. **MCP framework loggers** (`mcp.server.lowlevel.server`) - Standard Python logging to stderr
2. **FastMCP initialization** - Logs "Starting MCP server" messages to stderr
3. **Our own server logs** - Python's `logging.basicConfig()` sends all logs to stderr by default
4. **FastMCP banner** - Prints startup banner to stdout/stderr during initialization

**The Solution:** Route our INFO/WARNING logs to stdout, keep ERROR logs on stderr, and suppress FastMCP's initialization output.

## Solution

Updated logging configuration in `server.py` to:

1. **Suppress MCP framework INFO logs** - Set MCP loggers to WARNING level
2. **Keep our server logs at INFO** - Our own server messages remain visible
3. **Apply suppression before and after basicConfig** - Ensures it takes effect

## Changes Made

### 1. Route Our Logs to stdout (Not stderr)

**CRITICAL:** Python's `logging.basicConfig()` sends all logs to stderr by default. We need to route INFO/WARNING to stdout so Cursor doesn't treat them as errors.

```python
# Create custom handlers to separate stdout (INFO/WARNING) from stderr (ERROR)
class InfoWarningFilter(logging.Filter):
    """Filter to allow only INFO and WARNING level messages"""
    def filter(self, record):
        return record.levelno in (logging.DEBUG, logging.INFO, logging.WARNING)

class ErrorFilter(logging.Filter):
    """Filter to allow only ERROR and CRITICAL level messages"""
    def filter(self, record):
        return record.levelno >= logging.ERROR

# Create handlers
stdout_handler = logging.StreamHandler(sys.stdout)
stdout_handler.setLevel(logging.INFO)
stdout_handler.addFilter(InfoWarningFilter())
stdout_handler.setFormatter(logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s"))

stderr_handler = logging.StreamHandler(sys.stderr)
stderr_handler.setLevel(logging.ERROR)
stderr_handler.addFilter(ErrorFilter())
stderr_handler.setFormatter(logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s"))

# Configure root logger with our custom handlers
root_logger = logging.getLogger()
root_logger.setLevel(logging.INFO)
root_logger.handlers = []  # Clear any existing handlers
root_logger.addHandler(stdout_handler)
root_logger.addHandler(stderr_handler)
```

### 2. Suppress MCP Framework Loggers

```python
# Suppress MCP framework logs - set to WARNING to hide INFO messages
logging.getLogger("mcp").setLevel(logging.WARNING)
logging.getLogger("mcp.server").setLevel(logging.WARNING)
logging.getLogger("mcp.server.lowlevel").setLevel(logging.WARNING)
logging.getLogger("mcp.server.lowlevel.server").setLevel(logging.WARNING)
logging.getLogger("mcp.server.stdio").setLevel(logging.WARNING)
logging.getLogger("fastmcp").setLevel(logging.WARNING)

# Configure our logger
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    force=True
)

# Re-apply suppression (in case basicConfig reset it)
logging.getLogger("mcp").setLevel(logging.WARNING)
# ... (same loggers again)
```

### 3. Suppress FastMCP Initialization Logging

**FastMCP prints a banner and startup messages to stdout/stderr during initialization.** We suppress both:

```python
# Suppress stdout and stderr during FastMCP initialization
# FastMCP prints banner and "Starting MCP server" messages during initialization
import io
import contextlib

@contextlib.contextmanager
def suppress_fastmcp_output():
    """Temporarily suppress stdout and stderr during FastMCP initialization"""
    original_stdout = sys.stdout
    original_stderr = sys.stderr
    try:
        # Redirect both stdout and stderr to suppress FastMCP banner and startup messages
        sys.stdout = io.StringIO()
        sys.stderr = io.StringIO()
        yield
    finally:
        sys.stdout = original_stdout
        sys.stderr = original_stderr

# Use during FastMCP initialization
with suppress_fastmcp_output():
    mcp = FastMCP("Project Management Automation")

# Log initialization after suppressing FastMCP output
logger.info("FastMCP server initialized")
```

### 4. Suppress All MCP-Related Loggers

```python
# Suppress any logger that might have been created by FastMCP
for logger_name in logging.Logger.manager.loggerDict:
    if any(x in logger_name for x in ['mcp', 'fastmcp', 'stdio']):
        logging.getLogger(logger_name).setLevel(logging.WARNING)
```

## Result

- ✅ MCP framework INFO logs suppressed (no longer appear as errors)
- ✅ Our server INFO logs still visible
- ✅ WARNING and ERROR logs from MCP framework still visible (for actual issues)
- ✅ Cleaner Cursor MCP logs

## Verification

After restarting Cursor, you should see:
- ❌ No more `[error]` entries for INFO level MCP framework messages
- ✅ Only actual errors appear with `[error]` severity
- ✅ Our server INFO messages appear normally

## Affected Loggers

The following MCP framework loggers are suppressed at INFO level:
- `mcp`
- `mcp.server`
- `mcp.server.lowlevel`
- `mcp.server.lowlevel.server` ← The one causing the issue
- `mcp.server.stdio`
- `fastmcp`

---

**Status:** ✅ Fixed - MCP framework INFO logs suppressed
