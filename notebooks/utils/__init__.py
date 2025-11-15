"""
Utility modules for Jupyter notebooks.

Provides helpers for data loading, visualization, and integration with
the IB Box Spread codebase.
"""

from notebooks.utils.data_loaders import (
    load_questdb_data,
    load_orats_data,
    load_trading_logs,
    check_questdb_connection,
    load_questdb_data_chunked,
)

from notebooks.utils.visualizations import (
    plot_box_spread_opportunities,
    plot_performance_metrics,
    plot_feature_importance,
    plot_time_series,
    plot_correlation_matrix,
)

from notebooks.utils.notebook_helpers import (
    setup_notebook_environment,
    save_notebook_output,
    load_config,
)

__all__ = [
    # Data loaders
    "load_questdb_data",
    "load_orats_data",
    "load_trading_logs",
    "check_questdb_connection",
    "load_questdb_data_chunked",
    # Visualizations
    "plot_box_spread_opportunities",
    "plot_performance_metrics",
    "plot_feature_importance",
    "plot_time_series",
    "plot_correlation_matrix",
    # Helpers
    "setup_notebook_environment",
    "save_notebook_output",
    "load_config",
]
