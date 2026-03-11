"""
Python TUI for IB Box Spread Trading

This module provides a terminal user interface (TUI) for monitoring and controlling
the IB box spread trading system. It uses the Textual library for the UI and shares
snapshot/config contracts with the rest of the platform.

MIGRATION NOTES FOR FUTURE C++ MIGRATION (pybind11):
- All data models in tui/models.py can be exposed via pybind11
- Provider classes in tui/providers/ can be wrapped as C++ classes
- UI rendering logic in tui/app.py should remain in Python (or use C++ for data processing only)
- Consider keeping Python TUI as reference implementation during migration
"""

__version__ = "0.1.0"
