"""
Error Handling and Logging for Project Management Automation MCP Server

Provides centralized error handling, structured logging, and error response formatting.
"""

import json
import logging
import traceback
from typing import Any, Dict, Optional
from datetime import datetime
from enum import Enum

logger = logging.getLogger(__name__)


class ErrorCode(Enum):
    """Standard error codes for automation tools."""
    SUCCESS = "SUCCESS"
    INVALID_INPUT = "INVALID_INPUT"
    AUTOMATION_ERROR = "AUTOMATION_ERROR"
    CONFIGURATION_ERROR = "CONFIGURATION_ERROR"
    DEPENDENCY_ERROR = "DEPENDENCY_ERROR"
    TIMEOUT_ERROR = "TIMEOUT_ERROR"
    PERMISSION_ERROR = "PERMISSION_ERROR"
    UNKNOWN_ERROR = "UNKNOWN_ERROR"


class AutomationError(Exception):
    """Base exception for automation tool errors."""

    def __init__(
        self,
        message: str,
        error_code: ErrorCode = ErrorCode.AUTOMATION_ERROR,
        details: Optional[Dict[str, Any]] = None
    ):
        self.message = message
        self.error_code = error_code
        self.details = details or {}
        super().__init__(self.message)


def format_error_response(
    error: Exception,
    error_code: ErrorCode = ErrorCode.UNKNOWN_ERROR,
    include_traceback: bool = False
) -> Dict[str, Any]:
    """
    Format an error response for MCP tool output.

    Args:
        error: The exception that occurred
        error_code: Standard error code
        include_traceback: Whether to include full traceback (for debugging)

    Returns:
        Formatted error response dictionary
    """
    response = {
        "success": False,
        "error": {
            "code": error_code.value,
            "message": str(error),
            "timestamp": datetime.now().isoformat()
        }
    }

    if include_traceback:
        response["error"]["traceback"] = traceback.format_exc()

    if isinstance(error, AutomationError):
        response["error"]["code"] = error.error_code.value
        if error.details:
            response["error"]["details"] = error.details

    return response


def format_success_response(
    data: Dict[str, Any],
    message: Optional[str] = None
) -> Dict[str, Any]:
    """
    Format a success response for MCP tool output.

    Args:
        data: Result data to include
        message: Optional success message

    Returns:
        Formatted success response dictionary
    """
    response = {
        "success": True,
        "data": data,
        "timestamp": datetime.now().isoformat()
    }

    if message:
        response["message"] = message

    return response


def handle_automation_error(
    func,
    *args,
    **kwargs
) -> Dict[str, Any]:
    """
    Decorator/wrapper to handle automation errors gracefully.

    Args:
        func: Function to wrap
        *args: Function arguments
        **kwargs: Function keyword arguments

    Returns:
        Formatted response (success or error)
    """
    try:
        result = func(*args, **kwargs)

        if isinstance(result, dict) and "success" in result:
            return result

        return format_success_response(result if isinstance(result, dict) else {"result": result})

    except AutomationError as e:
        logger.error(f"Automation error in {func.__name__}: {e.message}")
        return format_error_response(e, e.error_code)

    except ValueError as e:
        logger.error(f"Invalid input in {func.__name__}: {e}")
        return format_error_response(e, ErrorCode.INVALID_INPUT)

    except FileNotFoundError as e:
        logger.error(f"File not found in {func.__name__}: {e}")
        return format_error_response(
            e,
            ErrorCode.CONFIGURATION_ERROR,
            details={"file": str(e)}
        )

    except PermissionError as e:
        logger.error(f"Permission error in {func.__name__}: {e}")
        return format_error_response(e, ErrorCode.PERMISSION_ERROR)

    except ImportError as e:
        logger.error(f"Dependency error in {func.__name__}: {e}")
        return format_error_response(
            e,
            ErrorCode.DEPENDENCY_ERROR,
            details={"missing_module": str(e)}
        )

    except Exception as e:
        logger.error(f"Unexpected error in {func.__name__}: {e}", exc_info=True)
        return format_error_response(
            e,
            ErrorCode.UNKNOWN_ERROR,
            include_traceback=True
        )


def log_automation_execution(
    tool_name: str,
    duration: float,
    success: bool,
    error: Optional[Exception] = None
) -> None:
    """
    Log automation tool execution for monitoring.

    Args:
        tool_name: Name of the tool executed
        duration: Execution duration in seconds
        success: Whether execution succeeded
        error: Exception if execution failed
    """
    log_data = {
        "tool": tool_name,
        "duration_seconds": duration,
        "success": success,
        "timestamp": datetime.now().isoformat()
    }

    if error:
        log_data["error"] = str(error)
        log_data["error_type"] = type(error).__name__

    if success:
        logger.info(f"Tool execution successful: {json.dumps(log_data)}")
    else:
        logger.warning(f"Tool execution failed: {json.dumps(log_data)}")
