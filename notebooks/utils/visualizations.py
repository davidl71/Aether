"""
Visualization utilities for notebooks.

Provides plotting functions for box spread analysis, performance metrics,
and other trading-related visualizations.
"""

import matplotlib.pyplot as plt
import matplotlib.dates as mdates
import seaborn as sns
import pandas as pd
import numpy as np
from typing import Optional, List, Dict, Tuple
import logging

logger = logging.getLogger(__name__)

# Set style
sns.set_style("darkgrid")
plt.rcParams["figure.figsize"] = (12, 6)
plt.rcParams["font.size"] = 10


def plot_box_spread_opportunities(
    opportunities_df: pd.DataFrame,
    figsize: Tuple[int, int] = (14, 8),
    save_path: Optional[str] = None,
) -> plt.Figure:
    """
    Plot box spread opportunities over time.

    Args:
        opportunities_df: DataFrame with columns: timestamp, symbol, roi, profit, etc.
        figsize: Figure size
        save_path: Optional path to save figure

    Returns:
        Matplotlib figure
    """
    fig, axes = plt.subplots(2, 2, figsize=figsize)
    fig.suptitle("Box Spread Opportunities Analysis", fontsize=16, fontweight="bold")

    # 1. ROI distribution
    ax1 = axes[0, 0]
    if "roi" in opportunities_df.columns:
        ax1.hist(opportunities_df["roi"], bins=50, edgecolor="black", alpha=0.7)
        ax1.axvline(opportunities_df["roi"].mean(), color="red", linestyle="--", label="Mean")
        ax1.set_xlabel("ROI (%)")
        ax1.set_ylabel("Frequency")
        ax1.set_title("ROI Distribution")
        ax1.legend()
        ax1.grid(True, alpha=0.3)

    # 2. Opportunities over time
    ax2 = axes[0, 1]
    if "timestamp" in opportunities_df.columns:
        opportunities_df["timestamp"] = pd.to_datetime(opportunities_df["timestamp"])
        daily_opps = opportunities_df.groupby(opportunities_df["timestamp"].dt.date).size()
        ax2.plot(daily_opps.index, daily_opps.values, marker="o", linewidth=2)
        ax2.set_xlabel("Date")
        ax2.set_ylabel("Number of Opportunities")
        ax2.set_title("Opportunities Over Time")
        ax2.grid(True, alpha=0.3)
        ax2.xaxis.set_major_formatter(mdates.DateFormatter("%Y-%m-%d"))
        plt.setp(ax2.xaxis.get_majorticklabels(), rotation=45, ha="right")

    # 3. Profit vs ROI scatter
    ax3 = axes[1, 0]
    if "profit" in opportunities_df.columns and "roi" in opportunities_df.columns:
        ax3.scatter(opportunities_df["roi"], opportunities_df["profit"], alpha=0.6, s=50)
        ax3.set_xlabel("ROI (%)")
        ax3.set_ylabel("Profit ($)")
        ax3.set_title("Profit vs ROI")
        ax3.grid(True, alpha=0.3)

    # 4. Symbol distribution
    ax4 = axes[1, 1]
    if "symbol" in opportunities_df.columns:
        symbol_counts = opportunities_df["symbol"].value_counts().head(10)
        ax4.barh(range(len(symbol_counts)), symbol_counts.values)
        ax4.set_yticks(range(len(symbol_counts)))
        ax4.set_yticklabels(symbol_counts.index)
        ax4.set_xlabel("Number of Opportunities")
        ax4.set_title("Top 10 Symbols by Opportunities")
        ax4.grid(True, alpha=0.3, axis="x")

    plt.tight_layout()

    if save_path:
        fig.savefig(save_path, dpi=300, bbox_inches="tight")
        logger.info(f"Saved figure to {save_path}")

    return fig


def plot_performance_metrics(
    performance_df: pd.DataFrame,
    metrics: Optional[List[str]] = None,
    figsize: Tuple[int, int] = (14, 8),
    save_path: Optional[str] = None,
) -> plt.Figure:
    """
    Plot performance metrics over time.

    Args:
        performance_df: DataFrame with timestamp and performance metrics
        metrics: List of metric columns to plot (defaults to all numeric columns)
        figsize: Figure size
        save_path: Optional path to save figure

    Returns:
        Matplotlib figure
    """
    if metrics is None:
        metrics = [col for col in performance_df.columns if performance_df[col].dtype in [np.float64, np.int64]]
        metrics = [m for m in metrics if m != "timestamp"]

    n_metrics = len(metrics)
    n_cols = 2
    n_rows = (n_metrics + 1) // 2

    fig, axes = plt.subplots(n_rows, n_cols, figsize=figsize)
    if n_metrics == 1:
        axes = [axes]
    else:
        axes = axes.flatten()

    fig.suptitle("Performance Metrics Over Time", fontsize=16, fontweight="bold")

    if "timestamp" in performance_df.columns:
        performance_df["timestamp"] = pd.to_datetime(performance_df["timestamp"])
        x = performance_df["timestamp"]
    else:
        x = range(len(performance_df))

    for i, metric in enumerate(metrics[:n_rows * n_cols]):
        ax = axes[i]
        ax.plot(x, performance_df[metric], linewidth=2, marker="o", markersize=3)
        ax.set_xlabel("Time")
        ax.set_ylabel(metric.replace("_", " ").title())
        ax.set_title(f"{metric.replace('_', ' ').title()} Over Time")
        ax.grid(True, alpha=0.3)

        if "timestamp" in performance_df.columns:
            ax.xaxis.set_major_formatter(mdates.DateFormatter("%Y-%m-%d"))
            plt.setp(ax.xaxis.get_majorticklabels(), rotation=45, ha="right")

    # Hide unused subplots
    for i in range(n_metrics, len(axes)):
        axes[i].set_visible(False)

    plt.tight_layout()

    if save_path:
        fig.savefig(save_path, dpi=300, bbox_inches="tight")
        logger.info(f"Saved figure to {save_path}")

    return fig


def plot_feature_importance(
    feature_importance: Dict[str, float],
    top_n: int = 20,
    figsize: Tuple[int, int] = (10, 8),
    save_path: Optional[str] = None,
) -> plt.Figure:
    """
    Plot feature importance from ML models.

    Args:
        feature_importance: Dictionary mapping feature names to importance scores
        top_n: Number of top features to display
        figsize: Figure size
        save_path: Optional path to save figure

    Returns:
        Matplotlib figure
    """
    # Sort by importance
    sorted_features = sorted(feature_importance.items(), key=lambda x: x[1], reverse=True)
    top_features = sorted_features[:top_n]

    features, importances = zip(*top_features)

    fig, ax = plt.subplots(figsize=figsize)

    y_pos = np.arange(len(features))
    ax.barh(y_pos, importances, align="center")
    ax.set_yticks(y_pos)
    ax.set_yticklabels([f.replace("_", " ").title() for f in features])
    ax.set_xlabel("Importance Score")
    ax.set_title(f"Top {top_n} Feature Importance")
    ax.grid(True, alpha=0.3, axis="x")

    plt.tight_layout()

    if save_path:
        fig.savefig(save_path, dpi=300, bbox_inches="tight")
        logger.info(f"Saved figure to {save_path}")

    return fig


def plot_time_series(
    df: pd.DataFrame,
    columns: List[str],
    timestamp_col: str = "timestamp",
    figsize: Tuple[int, int] = (14, 6),
    save_path: Optional[str] = None,
) -> plt.Figure:
    """
    Plot multiple time series on the same figure.

    Args:
        df: DataFrame with time series data
        columns: List of column names to plot
        timestamp_col: Name of timestamp column
        figsize: Figure size
        save_path: Optional path to save figure

    Returns:
        Matplotlib figure
    """
    fig, ax = plt.subplots(figsize=figsize)

    if timestamp_col in df.columns:
        df[timestamp_col] = pd.to_datetime(df[timestamp_col])
        x = df[timestamp_col]
    else:
        x = range(len(df))

    for col in columns:
        if col in df.columns:
            ax.plot(x, df[col], label=col.replace("_", " ").title(), linewidth=2)

    ax.set_xlabel("Time")
    ax.set_ylabel("Value")
    ax.set_title("Time Series Comparison")
    ax.legend()
    ax.grid(True, alpha=0.3)

    if timestamp_col in df.columns:
        ax.xaxis.set_major_formatter(mdates.DateFormatter("%Y-%m-%d"))
        plt.setp(ax.xaxis.get_majorticklabels(), rotation=45, ha="right")

    plt.tight_layout()

    if save_path:
        fig.savefig(save_path, dpi=300, bbox_inches="tight")
        logger.info(f"Saved figure to {save_path}")

    return fig


def plot_correlation_matrix(
    df: pd.DataFrame,
    columns: Optional[List[str]] = None,
    figsize: Tuple[int, int] = (12, 10),
    save_path: Optional[str] = None,
) -> plt.Figure:
    """
    Plot correlation matrix heatmap.

    Args:
        df: DataFrame with numeric columns
        columns: List of columns to include (defaults to all numeric)
        figsize: Figure size
        save_path: Optional path to save figure

    Returns:
        Matplotlib figure
    """
    if columns is None:
        columns = [col for col in df.columns if df[col].dtype in [np.float64, np.int64]]

    corr_df = df[columns].corr()

    fig, ax = plt.subplots(figsize=figsize)

    sns.heatmap(
        corr_df,
        annot=True,
        fmt=".2f",
        cmap="coolwarm",
        center=0,
        square=True,
        linewidths=1,
        cbar_kws={"shrink": 0.8},
        ax=ax,
    )

    ax.set_title("Correlation Matrix", fontsize=14, fontweight="bold")
    plt.tight_layout()

    if save_path:
        fig.savefig(save_path, dpi=300, bbox_inches="tight")
        logger.info(f"Saved figure to {save_path}")

    return fig
