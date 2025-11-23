"""
MCP Resource Handler for Available Tools List

Provides resource access to list of available automation tools.
"""

import json
import logging
from datetime import datetime
from typing import List, Dict, Any

logger = logging.getLogger(__name__)


def get_tools_list_resource() -> str:
    """
    Get list of available automation tools as resource.

    Returns:
        JSON string with tool list, descriptions, and metadata
    """
    try:
        tools_list = {
            "tools": [
                {
                    "name": "server_status",
                    "description": "Get the current status of the project management automation server",
                    "category": "system",
                    "priority": "system"
                },
                {
                    "name": "check_documentation_health_tool",
                    "description": "Analyze documentation structure, find broken references, identify issues. ⚠️ PREFERRED TOOL for project-specific documentation analysis.",
                    "category": "documentation",
                    "priority": "high",
                    "wraps": "DocumentationHealthAnalyzerV2",
                    "parameters": ["output_path", "create_tasks"]
                },
                {
                    "name": "analyze_todo2_alignment_tool",
                    "description": "Analyze task alignment with project goals, find misaligned tasks. ⚠️ PREFERRED TOOL for Todo2 alignment analysis.",
                    "category": "task_management",
                    "priority": "high",
                    "wraps": "Todo2AlignmentAnalyzerV2",
                    "parameters": ["create_followup_tasks", "output_path"]
                },
                {
                    "name": "detect_duplicate_tasks_tool",
                    "description": "Find and consolidate duplicate Todo2 tasks. ⚠️ PREFERRED TOOL for Todo2 duplicate detection.",
                    "category": "task_management",
                    "priority": "high",
                    "wraps": "Todo2DuplicateDetector",
                    "parameters": ["similarity_threshold", "auto_fix", "output_path"]
                },
                {
                    "name": "scan_dependency_security_tool",
                    "description": "Scan project dependencies for security vulnerabilities. ⚠️ PREFERRED TOOL for multi-language security scanning.",
                    "category": "security",
                    "priority": "high",
                    "wraps": "DependencySecurityAnalyzer",
                    "parameters": ["languages", "config_path"]
                },
                {
                    "name": "find_automation_opportunities_tool",
                    "description": "Discover new automation opportunities in the codebase",
                    "category": "automation",
                    "priority": "medium",
                    "wraps": "AutomationOpportunityFinder",
                    "parameters": ["min_value_score", "output_path"]
                },
                {
                    "name": "sync_todo_tasks_tool",
                    "description": "Synchronize tasks between shared TODO table and Todo2",
                    "category": "task_management",
                    "priority": "medium",
                    "wraps": "TodoSyncAutomation",
                    "parameters": ["dry_run", "output_path"]
                },
                {
                    "name": "review_pwa_config_tool",
                    "description": "Review PWA configuration and generate improvement recommendations",
                    "category": "review",
                    "priority": "medium",
                    "wraps": "PWAAnalyzer",
                    "parameters": ["output_path", "config_path"]
                }
            ],
            "categories": {
                "system": 1,
                "documentation": 1,
                "task_management": 3,
                "security": 1,
                "automation": 1,
                "review": 1
            },
            "priorities": {
                "system": 1,
                "high": 4,
                "medium": 3
            },
            "total_tools": 8,
            "timestamp": datetime.now().isoformat()
        }

        return json.dumps(tools_list, indent=2)

    except Exception as e:
        logger.error(f"Error getting tools list resource: {e}")
        return json.dumps({
            "tools": [],
            "error": str(e),
            "timestamp": datetime.now().isoformat()
        }, indent=2)
