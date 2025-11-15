"""
Helper utilities for notebook development workflow.

Provides functions for environment setup, output management, and context sharing.
"""

import sys
import os
import json
from pathlib import Path
from datetime import datetime
from typing import Dict, Optional, Any
import logging

logger = logging.getLogger(__name__)

# Project root
project_root = Path(__file__).parent.parent.parent


def setup_notebook_environment(
    add_project_root: bool = True,
    set_logging_level: str = "INFO",
    configure_matplotlib: bool = True,
) -> Dict[str, Any]:
    """
    Setup notebook environment with common configurations.

    Args:
        add_project_root: Add project root to Python path
        set_logging_level: Logging level (DEBUG, INFO, WARNING, ERROR)
        configure_matplotlib: Configure matplotlib for inline display

    Returns:
        Dictionary with environment info
    """
    env_info = {
        "project_root": str(project_root),
        "python_path": sys.executable,
        "timestamp": datetime.now().isoformat(),
    }

    # Add project root to path
    if add_project_root and str(project_root) not in sys.path:
        sys.path.insert(0, str(project_root))
        env_info["project_root_added"] = True

    # Configure logging
    logging.basicConfig(
        level=getattr(logging, set_logging_level.upper()),
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    )
    env_info["logging_level"] = set_logging_level

    # Configure matplotlib
    if configure_matplotlib:
        try:
            import matplotlib
            matplotlib.use("inline")  # For Jupyter
            matplotlib.rcParams["figure.figsize"] = (12, 6)
            matplotlib.rcParams["font.size"] = 10
            env_info["matplotlib_configured"] = True
        except ImportError:
            env_info["matplotlib_configured"] = False

    # Load environment variables
    env_file = project_root / ".env"
    if env_file.exists():
        try:
            from dotenv import load_dotenv
            load_dotenv(env_file)
            env_info["env_file_loaded"] = True
        except ImportError:
            env_info["env_file_loaded"] = False
        except Exception as e:
            logger.warning(f"Failed to load .env file: {e}")
            env_info["env_file_loaded"] = False

    logger.info("Notebook environment setup complete")
    return env_info


def save_notebook_output(
    output_data: Dict[str, Any],
    output_dir: Optional[str] = None,
    filename: Optional[str] = None,
) -> Path:
    """
    Save notebook output (figures, data, results) to file.

    Args:
        output_data: Dictionary with output data
        output_dir: Output directory (defaults to notebooks/output/)
        filename: Output filename (defaults to timestamp-based)

    Returns:
        Path to saved file
    """
    if output_dir is None:
        output_dir = project_root / "notebooks" / "output"
    else:
        output_dir = Path(output_dir)

    output_dir.mkdir(parents=True, exist_ok=True)

    if filename is None:
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"notebook_output_{timestamp}.json"

    output_path = output_dir / filename

    # Add metadata
    output_data["metadata"] = {
        "timestamp": datetime.now().isoformat(),
        "notebook": os.environ.get("JUPYTER_NOTEBOOK_PATH", "unknown"),
    }

    # Save as JSON
    with open(output_path, "w") as f:
        json.dump(output_data, f, indent=2, default=str)

    logger.info(f"Saved notebook output to {output_path}")
    return output_path


def load_config(config_path: Optional[str] = None) -> Dict[str, Any]:
    """
    Load configuration from JSON file.

    Args:
        config_path: Path to config file (defaults to config/config.json)

    Returns:
        Configuration dictionary
    """
    if config_path is None:
        config_path = project_root / "config" / "config.json"
    else:
        config_path = Path(config_path)

    if not config_path.exists():
        logger.warning(f"Config file not found: {config_path}")
        return {}

    try:
        with open(config_path, "r") as f:
            config = json.load(f)
        logger.info(f"Loaded config from {config_path}")
        return config
    except Exception as e:
        logger.error(f"Failed to load config: {e}")
        return {}


def create_research_log_entry(
    title: str,
    findings: str,
    code_snippets: Optional[Dict[str, str]] = None,
    figures: Optional[Dict[str, str]] = None,
    next_steps: Optional[str] = None,
    tags: Optional[list] = None,
) -> Dict[str, Any]:
    """
    Create a structured research log entry.

    Args:
        title: Entry title
        findings: Research findings/observations
        code_snippets: Dictionary of code snippets (name -> code)
        figures: Dictionary of figure paths (name -> path)
        next_steps: Next steps or follow-up actions
        tags: List of tags for categorization

    Returns:
        Dictionary with research log entry
    """
    entry = {
        "timestamp": datetime.now().isoformat(),
        "title": title,
        "findings": findings,
        "code_snippets": code_snippets or {},
        "figures": figures or {},
        "next_steps": next_steps,
        "tags": tags or [],
    }

    return entry


def save_research_log_entry(
    entry: Dict[str, Any],
    log_file: Optional[str] = None,
) -> Path:
    """
    Save research log entry to file.

    Args:
        entry: Research log entry dictionary
        log_file: Log file path (defaults to notebooks/06-dev-workflow/research_log.json)

    Returns:
        Path to log file
    """
    if log_file is None:
        log_file = project_root / "notebooks" / "06-dev-workflow" / "research_log.json"
    else:
        log_file = Path(log_file)

    log_file.parent.mkdir(parents=True, exist_ok=True)

    # Load existing entries
    if log_file.exists():
        try:
            with open(log_file, "r") as f:
                log_data = json.load(f)
        except Exception:
            log_data = {"entries": []}
    else:
        log_data = {"entries": []}

    # Add new entry
    log_data["entries"].append(entry)
    log_data["last_updated"] = datetime.now().isoformat()

    # Save
    with open(log_file, "w") as f:
        json.dump(log_data, f, indent=2, default=str)

    logger.info(f"Saved research log entry to {log_file}")
    return log_file


def create_decision_log_entry(
    decision: str,
    context: str,
    rationale: str,
    alternatives_considered: Optional[list] = None,
    impact: Optional[str] = None,
    tags: Optional[list] = None,
) -> Dict[str, Any]:
    """
    Create a structured decision log entry.

    Args:
        decision: The decision made
        context: Context/situation that required the decision
        rationale: Reasoning behind the decision
        alternatives_considered: List of alternatives that were considered
        impact: Expected impact of the decision
        tags: List of tags for categorization

    Returns:
        Dictionary with decision log entry
    """
    entry = {
        "timestamp": datetime.now().isoformat(),
        "decision": decision,
        "context": context,
        "rationale": rationale,
        "alternatives_considered": alternatives_considered or [],
        "impact": impact,
        "tags": tags or [],
    }

    return entry


def save_decision_log_entry(
    entry: Dict[str, Any],
    log_file: Optional[str] = None,
) -> Path:
    """
    Save decision log entry to file.

    Args:
        entry: Decision log entry dictionary
        log_file: Log file path (defaults to notebooks/06-dev-workflow/decision_log.json)

    Returns:
        Path to log file
    """
    if log_file is None:
        log_file = project_root / "notebooks" / "06-dev-workflow" / "decision_log.json"
    else:
        log_file = Path(log_file)

    log_file.parent.mkdir(parents=True, exist_ok=True)

    # Load existing entries
    if log_file.exists():
        try:
            with open(log_file, "r") as f:
                log_data = json.load(f)
        except Exception:
            log_data = {"entries": []}
    else:
        log_data = {"entries": []}

    # Add new entry
    log_data["entries"].append(entry)
    log_data["last_updated"] = datetime.now().isoformat()

    # Save
    with open(log_file, "w") as f:
        json.dump(log_data, f, indent=2, default=str)

    logger.info(f"Saved decision log entry to {log_file}")
    return log_file
