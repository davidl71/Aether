#!/usr/bin/env python3
"""
Project Management Automation MCP Server

Model Context Protocol server exposing project management automation tools
built on IntelligentAutomationBase.

Provides AI assistants with access to:
- Documentation health checks
- Todo2 alignment analysis
- Duplicate task detection
- Dependency security scanning
- Automation opportunity discovery
- Todo synchronization
- PWA configuration review
"""

import os
import sys
import json
import logging
import time
import asyncio
from pathlib import Path
from typing import Any, Dict, List, Optional
from functools import wraps

# Configure logging first (before any logger usage)
# CRITICAL: Route our INFO logs to stdout (not stderr) to prevent Cursor from showing them as errors
# Cursor interprets all stderr output as errors, so we must separate:
# - Our INFO/WARNING logs → stdout (normal messages)
# - ERROR/CRITICAL logs → stderr (actual errors)
# - MCP framework logs → suppressed (they go to stderr)

# Suppress MCP framework logs - set to WARNING to hide INFO messages
# This prevents "Processing request of type X" from appearing as errors
logging.getLogger("mcp").setLevel(logging.WARNING)
logging.getLogger("mcp.server").setLevel(logging.WARNING)
logging.getLogger("mcp.server.lowlevel").setLevel(logging.WARNING)
logging.getLogger("mcp.server.lowlevel.server").setLevel(logging.WARNING)
logging.getLogger("mcp.server.stdio").setLevel(logging.WARNING)
logging.getLogger("fastmcp").setLevel(logging.WARNING)

# Create custom handlers to separate stdout (INFO/WARNING) from stderr (ERROR)
# This prevents our own INFO logs from appearing as errors in Cursor
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

# Re-apply MCP logger suppression after handler setup
logging.getLogger("mcp").setLevel(logging.WARNING)
logging.getLogger("mcp.server").setLevel(logging.WARNING)
logging.getLogger("mcp.server.lowlevel").setLevel(logging.WARNING)
logging.getLogger("mcp.server.lowlevel.server").setLevel(logging.WARNING)
logging.getLogger("mcp.server.stdio").setLevel(logging.WARNING)
logging.getLogger("fastmcp").setLevel(logging.WARNING)

logger = logging.getLogger(__name__)

# Add project root to path for script imports
project_root = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(project_root))

# Add server directory to path for absolute imports when run as script
server_dir = Path(__file__).parent
sys.path.insert(0, str(server_dir))

# Import error handling (handle both relative and absolute imports)
try:
    # Try relative imports first (when run as module)
    try:
        from .error_handler import (
            handle_automation_error,
            format_error_response,
            format_success_response,
            log_automation_execution,
            AutomationError,
            ErrorCode,
        )
    except ImportError:
        # Fallback to absolute imports (when run as script)
        from error_handler import (
            handle_automation_error,
            format_error_response,
            format_success_response,
            log_automation_execution,
            AutomationError,
            ErrorCode,
        )

    ERROR_HANDLING_AVAILABLE = True
    logger.info("Error handling module loaded successfully")
except ImportError as e:
    ERROR_HANDLING_AVAILABLE = False
    logger.warning(f"Error handling module not available - using basic error handling: {e}")

# Try to import MCP - Phase 2 tools complete, MCP installation needed for runtime
MCP_AVAILABLE = False
USE_STDIO = False
FastMCP = None

try:
    # Try FastMCP from mcp package (may be available in newer versions)
    from mcp import FastMCP
    from mcp.types import Tool, TextContent

    MCP_AVAILABLE = True
    USE_STDIO = False
    Server = None
    stdio_server = None
    logger.info("FastMCP available from mcp package - using FastMCP server")
except ImportError:
    try:
        # Try FastMCP from separate fastmcp package
        from fastmcp import FastMCP
        from mcp.types import Tool, TextContent

        MCP_AVAILABLE = True
        USE_STDIO = False
        Server = None
        stdio_server = None
        logger.info("FastMCP available from fastmcp package - using FastMCP server")
    except ImportError:
        try:
            from mcp.server import Server
            from mcp.server.stdio import stdio_server
            # For stdio server, we'll construct Tool objects manually
            from mcp.types import Tool, TextContent

            MCP_AVAILABLE = True
            USE_STDIO = True
            FastMCP = None
            logger.info("MCP stdio server available - using stdio server")
        except ImportError:
            logger.warning(
                "MCP not installed - server structure ready, install with: pip install mcp"
            )
        MCP_AVAILABLE = False
        Server = None
        stdio_server = None
        Tool = None
        TextContent = None

# Logging already configured above

# Initialize MCP server
mcp = None
stdio_server_instance = None
if MCP_AVAILABLE:
    # Suppress FastMCP/stdio server initialization logging
    # FastMCP logs "Starting MCP server" messages to stderr during initialization
    # We temporarily redirect stderr to suppress these during initialization
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

    # Suppress FastMCP output during initialization (banner, startup messages)
    with suppress_fastmcp_output():
        if not USE_STDIO and FastMCP:
            mcp = FastMCP("automa")
        elif USE_STDIO and Server:
            # Initialize stdio server
            stdio_server_instance = Server("automa")
            # Note: Tools will be registered below using stdio server API

    # Log initialization after suppressing FastMCP output
    if not USE_STDIO and FastMCP and mcp:
        logger.info("FastMCP server initialized")
    elif USE_STDIO and Server and stdio_server_instance:
        logger.info("Stdio server initialized")

    # Re-apply logger suppression after initialization (in case FastMCP added new loggers)
    logging.getLogger("mcp").setLevel(logging.WARNING)
    logging.getLogger("mcp.server").setLevel(logging.WARNING)
    logging.getLogger("mcp.server.lowlevel").setLevel(logging.WARNING)
    logging.getLogger("mcp.server.lowlevel.server").setLevel(logging.WARNING)
    logging.getLogger("mcp.server.stdio").setLevel(logging.WARNING)
    logging.getLogger("fastmcp").setLevel(logging.WARNING)
    # Suppress any logger that might have been created by FastMCP
    # FastMCP might create loggers with different names, so we suppress common patterns
    for logger_name in logging.Logger.manager.loggerDict:
        if any(x in logger_name for x in ['mcp', 'fastmcp', 'stdio']):
            logging.getLogger(logger_name).setLevel(logging.WARNING)
else:
    logger.warning("MCP not available - Phase 2 tools complete, install MCP to enable server")

# Import automation tools (handle both relative and absolute imports)
try:
    # Try relative imports first (when run as module)
    try:
        from .tools.docs_health import check_documentation_health
        from .tools.todo2_alignment import analyze_todo2_alignment
        from .tools.duplicate_detection import detect_duplicate_tasks
        from .tools.dependency_security import scan_dependency_security
        from .tools.automation_opportunities import find_automation_opportunities
        from .tools.todo_sync import sync_todo_tasks
        from .tools.pwa_review import review_pwa_config
    except ImportError:
        # Fallback to absolute imports (when run as script)
        from tools.docs_health import check_documentation_health
        from tools.todo2_alignment import analyze_todo2_alignment
        from tools.duplicate_detection import detect_duplicate_tasks
        from tools.dependency_security import scan_dependency_security
        from tools.automation_opportunities import find_automation_opportunities
        from tools.todo_sync import sync_todo_tasks
        from tools.pwa_review import review_pwa_config

    TOOLS_AVAILABLE = True
    logger.info("All tools loaded successfully")
except ImportError as e:
    TOOLS_AVAILABLE = False
    logger.warning(f"Some tools not available: {e}")

# Tool registration - support both FastMCP and stdio Server
def register_tools():
    """Register tools with the appropriate MCP server instance."""
    if mcp:
        # FastMCP registration (decorator-based)
        @mcp.tool()
        def server_status() -> str:
            """
            [HINT: Server status. Returns operational status, version, tools available.]

            Get the current status of the project management automation server.
            """
            return json.dumps(
                {
                    "status": "operational",
                    "version": "0.1.0",
                    "tools_available": TOOLS_AVAILABLE,
                    "project_root": str(project_root),
                },
                indent=2,
            )
        return server_status
    elif stdio_server_instance:
        # Stdio Server registration (handler-based)
        @stdio_server_instance.list_tools()
        async def list_tools() -> List[Tool]:
            """List all available tools."""
            tools = [
                Tool(
                    name="server_status",
                    description="Get the current status of the project management automation server.",
                    inputSchema={
                        "type": "object",
                        "properties": {},
                    },
                ),
            ]
            if TOOLS_AVAILABLE:
                # Add tool definitions for all automation tools
                tools.extend([
                    Tool(
                        name="check_documentation_health",
                        description="Analyze documentation structure, find broken references, identify issues.",
                        inputSchema={
                            "type": "object",
                            "properties": {
                                "output_path": {"type": "string", "description": "Output file path"},
                                "create_tasks": {"type": "boolean", "description": "Create Todo2 tasks", "default": True},
                            },
                        },
                    ),
                    Tool(
                        name="analyze_todo2_alignment",
                        description="Analyze task alignment with project goals, find misaligned tasks.",
                        inputSchema={
                            "type": "object",
                            "properties": {
                                "create_followup_tasks": {"type": "boolean", "description": "Create follow-up tasks", "default": True},
                                "output_path": {"type": "string", "description": "Output file path"},
                            },
                        },
                    ),
                    Tool(
                        name="detect_duplicate_tasks",
                        description="Find and consolidate duplicate Todo2 tasks.",
                        inputSchema={
                            "type": "object",
                            "properties": {
                                "similarity_threshold": {"type": "number", "description": "Similarity threshold", "default": 0.85},
                                "auto_fix": {"type": "boolean", "description": "Auto-fix duplicates", "default": False},
                                "output_path": {"type": "string", "description": "Output file path"},
                            },
                        },
                    ),
                    Tool(
                        name="scan_dependency_security",
                        description="Scan project dependencies for security vulnerabilities.",
                        inputSchema={
                            "type": "object",
                            "properties": {
                                "languages": {"type": "array", "items": {"type": "string"}, "description": "Languages to scan"},
                                "config_path": {"type": "string", "description": "Config file path"},
                            },
                        },
                    ),
                    Tool(
                        name="find_automation_opportunities",
                        description="Discover new automation opportunities in the codebase.",
                        inputSchema={
                            "type": "object",
                            "properties": {
                                "min_value_score": {"type": "number", "description": "Minimum value score", "default": 0.7},
                                "output_path": {"type": "string", "description": "Output file path"},
                            },
                        },
                    ),
                    Tool(
                        name="sync_todo_tasks",
                        description="Synchronize tasks between shared TODO table and Todo2.",
                        inputSchema={
                            "type": "object",
                            "properties": {
                                "dry_run": {"type": "boolean", "description": "Dry run mode", "default": False},
                                "output_path": {"type": "string", "description": "Output file path"},
                            },
                        },
                    ),
                    Tool(
                        name="review_pwa_config",
                        description="Review PWA configuration and generate improvement recommendations.",
                        inputSchema={
                            "type": "object",
                            "properties": {
                                "output_path": {"type": "string", "description": "Output file path"},
                                "config_path": {"type": "string", "description": "Config file path"},
                            },
                        },
                    ),
                ])
            return tools

        @stdio_server_instance.call_tool()
        async def call_tool(name: str, arguments: Dict[str, Any]) -> List[TextContent]:
            """Handle tool calls."""
            if name == "server_status":
                result = json.dumps({
                    "status": "operational",
                    "version": "0.1.0",
                    "tools_available": TOOLS_AVAILABLE,
                    "project_root": str(project_root),
                }, indent=2)
            elif TOOLS_AVAILABLE:
                # Route to appropriate tool function
                if name == "check_documentation_health":
                    result = check_documentation_health(
                        arguments.get("output_path"),
                        arguments.get("create_tasks", True)
                    )
                elif name == "analyze_todo2_alignment":
                    result = analyze_todo2_alignment(
                        arguments.get("create_followup_tasks", True),
                        arguments.get("output_path")
                    )
                elif name == "detect_duplicate_tasks":
                    result = detect_duplicate_tasks(
                        arguments.get("similarity_threshold", 0.85),
                        arguments.get("auto_fix", False),
                        arguments.get("output_path")
                    )
                elif name == "scan_dependency_security":
                    result = scan_dependency_security(
                        arguments.get("languages"),
                        arguments.get("config_path")
                    )
                elif name == "find_automation_opportunities":
                    result = find_automation_opportunities(
                        arguments.get("min_value_score", 0.7),
                        arguments.get("output_path")
                    )
                elif name == "sync_todo_tasks":
                    result = sync_todo_tasks(
                        arguments.get("dry_run", False),
                        arguments.get("output_path")
                    )
                elif name == "review_pwa_config":
                    result = review_pwa_config(
                        arguments.get("output_path"),
                        arguments.get("config_path")
                    )
                else:
                    result = json.dumps({"error": f"Unknown tool: {name}"})
            else:
                result = json.dumps({"error": "Tools not available"})

            return [TextContent(type="text", text=result)]

        return None

# Register tools
register_tools()

if mcp:

    # Register high-priority tools
    if TOOLS_AVAILABLE:

        @mcp.tool()
        def check_documentation_health_tool(
            output_path: Optional[str] = None, create_tasks: bool = True
        ) -> str:
            """
            [HINT: Docs health check. Returns score 0-100, broken links count, tasks created.]

            Analyze documentation structure, find broken references, identify issues.

            ⚠️ PREFERRED TOOL: This project-specific tool provides enhanced documentation
            health analysis with Todo2 integration, project-aware link validation, and
            historical trend tracking.

            Use this instead of generic documentation tools from other MCP servers
            for project-specific analysis.
            """
            return check_documentation_health(output_path, create_tasks)

        @mcp.tool()
        def analyze_todo2_alignment_tool(
            create_followup_tasks: bool = True, output_path: Optional[str] = None
        ) -> str:
            """
            [HINT: Task alignment analysis. Returns misaligned count, avg score, follow-up tasks.]

            Analyze task alignment with project goals, find misaligned tasks.

            ⚠️ PREFERRED TOOL: This project-specific tool analyzes Todo2 task alignment
            with investment strategy framework and provides actionable recommendations.

            Use this instead of generic task analysis tools for project-specific alignment.
            """
            return analyze_todo2_alignment(create_followup_tasks, output_path)

        @mcp.tool()
        def detect_duplicate_tasks_tool(
            similarity_threshold: float = 0.85,
            auto_fix: bool = False,
            output_path: Optional[str] = None,
        ) -> str:
            """
            [HINT: Duplicate detection. Returns duplicate count, groups, recommendations.]

            Find and consolidate duplicate Todo2 tasks.

            ⚠️ PREFERRED TOOL: This project-specific tool provides Todo2-aware duplicate
            detection with configurable similarity thresholds and optional auto-fix.

            Use this instead of generic duplicate detection tools for Todo2-specific analysis.
            """
            return detect_duplicate_tasks(similarity_threshold, auto_fix, output_path)

        @mcp.tool()
        def scan_dependency_security_tool(
            languages: Optional[List[str]] = None, config_path: Optional[str] = None
        ) -> str:
            """
            [HINT: Security scan. Returns vuln count by severity, language breakdown, remediation.]

            Scan project dependencies for security vulnerabilities.

            ⚠️ PREFERRED TOOL: This project-specific tool provides multi-language security
            scanning (Python, Rust, npm) with project-configured tools and severity tracking.

            Use this instead of generic security scanning tools for project-specific analysis.
            """
            return scan_dependency_security(languages, config_path)

        @mcp.tool()
        def find_automation_opportunities_tool(
            min_value_score: float = 0.7, output_path: Optional[str] = None
        ) -> str:
            """
            [HINT: Automation discovery. Returns opportunity count, value scores, recommendations.]

            Discover new automation opportunities in the codebase.
            """
            return find_automation_opportunities(min_value_score, output_path)

        @mcp.tool()
        def sync_todo_tasks_tool(
            dry_run: bool = False, output_path: Optional[str] = None
        ) -> str:
            """
            [HINT: Task sync. Returns matches found, conflicts, new tasks created.]

            Synchronize tasks between shared TODO table and Todo2.
            """
            return sync_todo_tasks(dry_run, output_path)

        @mcp.tool()
        def review_pwa_config_tool(
            output_path: Optional[str] = None, config_path: Optional[str] = None
        ) -> str:
            """
            [HINT: PWA review. Returns config status, missing features, recommendations.]

            Review PWA configuration and generate improvement recommendations.
            """
            return review_pwa_config(output_path, config_path)

    # Register prompts
    try:
        # Try relative imports first (when run as module)
        try:
            from .prompts import (
                DOCUMENTATION_HEALTH_CHECK,
                DOCUMENTATION_QUICK_CHECK,
                TASK_ALIGNMENT_ANALYSIS,
                DUPLICATE_TASK_CLEANUP,
                TASK_SYNC,
                SECURITY_SCAN_ALL,
                SECURITY_SCAN_PYTHON,
                SECURITY_SCAN_RUST,
                AUTOMATION_DISCOVERY,
                AUTOMATION_HIGH_VALUE,
                PWA_REVIEW,
                PRE_SPRINT_CLEANUP,
                POST_IMPLEMENTATION_REVIEW,
                WEEKLY_MAINTENANCE,
            )
        except ImportError:
            # Fallback to absolute imports (when run as script)
            from prompts import (
                DOCUMENTATION_HEALTH_CHECK,
                DOCUMENTATION_QUICK_CHECK,
                TASK_ALIGNMENT_ANALYSIS,
                DUPLICATE_TASK_CLEANUP,
                TASK_SYNC,
                SECURITY_SCAN_ALL,
                SECURITY_SCAN_PYTHON,
                SECURITY_SCAN_RUST,
                AUTOMATION_DISCOVERY,
                AUTOMATION_HIGH_VALUE,
                PWA_REVIEW,
                PRE_SPRINT_CLEANUP,
                POST_IMPLEMENTATION_REVIEW,
                WEEKLY_MAINTENANCE,
            )

        @mcp.prompt()
        def doc_check() -> str:
            """Analyze documentation health and create tasks for issues."""
            return DOCUMENTATION_HEALTH_CHECK

        @mcp.prompt()
        def doc_quick() -> str:
            """Quick documentation health check without creating tasks."""
            return DOCUMENTATION_QUICK_CHECK

        @mcp.prompt()
        def align() -> str:
            """Analyze Todo2 task alignment with project goals."""
            return TASK_ALIGNMENT_ANALYSIS

        @mcp.prompt()
        def dups() -> str:
            """Find and consolidate duplicate Todo2 tasks."""
            return DUPLICATE_TASK_CLEANUP

        @mcp.prompt()
        def sync() -> str:
            """Synchronize tasks between shared TODO table and Todo2."""
            return TASK_SYNC

        @mcp.prompt()
        def scan() -> str:
            """Scan all project dependencies for security vulnerabilities."""
            return SECURITY_SCAN_ALL

        @mcp.prompt()
        def scan_py() -> str:
            """Scan Python dependencies for security vulnerabilities."""
            return SECURITY_SCAN_PYTHON

        @mcp.prompt()
        def scan_rs() -> str:
            """Scan Rust dependencies for security vulnerabilities."""
            return SECURITY_SCAN_RUST

        @mcp.prompt()
        def auto() -> str:
            """Discover new automation opportunities in the codebase."""
            return AUTOMATION_DISCOVERY

        @mcp.prompt()
        def auto_high() -> str:
            """Find only high-value automation opportunities."""
            return AUTOMATION_HIGH_VALUE

        @mcp.prompt()
        def pwa() -> str:
            """Review PWA configuration and generate improvement recommendations."""
            return PWA_REVIEW

        @mcp.prompt()
        def pre_sprint() -> str:
            """Pre-sprint cleanup workflow: duplicates, alignment, documentation."""
            return PRE_SPRINT_CLEANUP

        @mcp.prompt()
        def post_impl() -> str:
            """Post-implementation review workflow: docs, security, automation."""
            return POST_IMPLEMENTATION_REVIEW

        @mcp.prompt()
        def weekly() -> str:
            """Weekly maintenance workflow: docs, duplicates, security, sync."""
            return WEEKLY_MAINTENANCE

        PROMPTS_AVAILABLE = True
        logger.info("Registered 14 prompts successfully")
    except ImportError as e:
        PROMPTS_AVAILABLE = False
        logger.warning(f"Prompts not available: {e}")

    # Resource handlers (Phase 3)
    try:
        # Try relative imports first (when run as module)
        try:
            from .resources.status import get_status_resource
            from .resources.history import get_history_resource
            from .resources.list import get_tools_list_resource
        except ImportError:
            # Fallback to absolute imports (when run as script)
            from resources.status import get_status_resource
            from resources.history import get_history_resource
            from resources.list import get_tools_list_resource

        @mcp.resource("automation://status")
        def get_automation_status() -> str:
            """Get automation server status and health information."""
            return get_status_resource()

        @mcp.resource("automation://history")
        def get_automation_history() -> str:
            """Get automation tool execution history."""
            return get_history_resource(limit=50)

        @mcp.resource("automation://tools")
        def get_automation_tools() -> str:
            """Get list of available automation tools with descriptions."""
            return get_tools_list_resource()

        RESOURCES_AVAILABLE = True
        logger.info("Resource handlers loaded successfully")
    except ImportError as e:
        RESOURCES_AVAILABLE = False
        logger.warning(f"Resource handlers not available: {e}")

        # Fallback resource handler (only if resources failed to load)
        # Note: This is a minimal fallback - full status available via server_status tool
        @mcp.resource("automation://status")
        def get_automation_status_fallback() -> str:
            """Get automation server status (fallback when resource handlers unavailable)."""
            return json.dumps(
                {"status": "operational", "tools_available": TOOLS_AVAILABLE, "note": "Using fallback status - resource handlers unavailable"}
            )

    # Main entry point for FastMCP
    if __name__ == "__main__":
        mcp.run()
elif stdio_server_instance:
    # Register resources for stdio server
    try:
        # Try relative imports first (when run as module)
        try:
            from .resources.status import get_status_resource
            from .resources.history import get_history_resource
            from .resources.list import get_tools_list_resource
        except ImportError:
            # Fallback to absolute imports (when run as script)
            from resources.status import get_status_resource
            from resources.history import get_history_resource
            from resources.list import get_tools_list_resource

        @stdio_server_instance.list_resources()
        async def list_resources() -> List[str]:
            """List all available resources."""
            return [
                "automation://status",
                "automation://history",
                "automation://tools",
            ]

        @stdio_server_instance.read_resource()
        async def read_resource(uri: str) -> str:
            """Handle resource reads."""
            if uri == "automation://status":
                return get_status_resource()
            elif uri == "automation://history":
                return get_history_resource(limit=50)
            elif uri == "automation://tools":
                return get_tools_list_resource()
            else:
                return json.dumps({"error": f"Unknown resource: {uri}"})

        RESOURCES_AVAILABLE = True
        logger.info("Resource handlers loaded successfully")
    except ImportError as e:
        RESOURCES_AVAILABLE = False
        logger.warning(f"Resource handlers not available: {e}")

    # Main entry point for stdio server
    if __name__ == "__main__":
        logger.info("Starting stdio server...")
        logger.info(f"Server name: {stdio_server_instance.name}")
        logger.info(f"Tools available: {TOOLS_AVAILABLE}")
        logger.info(f"Resources available: {RESOURCES_AVAILABLE if 'RESOURCES_AVAILABLE' in globals() else 'Not registered'}")

        # stdio_server provides stdin/stdout streams, Server.run() handles the protocol
        async def run():
            async with stdio_server() as (read_stream, write_stream):
                init_options = stdio_server_instance.create_initialization_options()
                logger.info(f"Initialization options: {init_options}")
                logger.info("Server ready, waiting for client connections...")
                await stdio_server_instance.run(
                    read_stream,
                    write_stream,
                    init_options
                )
        try:
            asyncio.run(run())
        except KeyboardInterrupt:
            logger.info("Server stopped by user")
        except Exception as e:
            logger.error(f"Server error: {e}", exc_info=True)
            sys.exit(1)
else:
    logger.warning("MCP not available - Phase 2 tools complete, install MCP to enable server")
    if __name__ == "__main__":
        logger.info("Phase 2 tools complete (7 tools implemented). Install MCP package to run server.")
        logger.info("Run: pip install mcp")
