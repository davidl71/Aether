# Jupyter Notebooks for IB Box Spread Development

This directory contains Jupyter notebooks for interactive analysis, strategy development, and enhanced development workflow.

## Quick Start

```bash
# Install notebook dependencies
pip install -r ../requirements-notebooks.txt

# Start JupyterLab
jupyter lab

# Or use VS Code with Jupyter extension
code notebooks/
```

## Directory Structure

```
notebooks/
├── 00-setup/              # Setup and configuration notebooks
│   └── environment_setup.ipynb
├── 01-data-exploration/  # Data exploration and analysis
│   ├── questdb_analysis.ipynb
│   ├── orats_data_exploration.ipynb
│   └── market_data_quality.ipynb
├── 02-backtesting/       # Backtesting and strategy validation
│   ├── historical_backtest.ipynb
│   ├── parameter_optimization.ipynb
│   └── strategy_comparison.ipynb
├── 03-ml-development/    # Machine learning model development
│   ├── feature_engineering.ipynb
│   ├── model_training.ipynb
│   ├── model_evaluation.ipynb
│   └── shap_analysis.ipynb
├── 04-strategy-dev/      # Strategy development and testing
│   ├── box_spread_analysis.ipynb
│   ├── opportunity_detection.ipynb
│   └── risk_analysis.ipynb
├── 05-performance/       # Performance monitoring and analysis
│   ├── execution_analysis.ipynb
│   ├── portfolio_performance.ipynb
│   └── latency_analysis.ipynb
├── 06-dev-workflow/      # Development workflow and context sharing
│   ├── research_log.ipynb
│   ├── decision_log.ipynb
│   └── experiment_tracking.ipynb
├── utils/                # Helper utilities
│   ├── __init__.py
│   ├── data_loaders.py
│   ├── visualizations.py
│   └── notebook_helpers.py
└── templates/            # Notebook templates
    ├── analysis_template.ipynb
    └── experiment_template.ipynb
```

## Development Workflow Integration

### 1. Research & Exploration

Use notebooks to:
- Explore new data sources
- Analyze market patterns
- Test hypothesis quickly
- Document findings with code + results

**Example**: `01-data-exploration/orats_data_exploration.ipynb`

### 2. Strategy Development

Use notebooks to:
- Prototype new strategies
- Visualize box spread opportunities
- Test parameter combinations
- Document strategy logic

**Example**: `04-strategy-dev/box_spread_analysis.ipynb`

### 3. Model Development

Use notebooks to:
- Engineer features interactively
- Train and evaluate models
- Analyze feature importance
- Compare model variants

**Example**: `03-ml-development/model_training.ipynb`

### 4. Context Sharing

Use notebooks to:
- Document research findings
- Share analysis with team
- Track decisions and rationale
- Maintain experiment history

**Example**: `06-dev-workflow/research_log.ipynb`

## Key Features

### Data Access Utilities

```python
from notebooks.utils.data_loaders import (
    load_questdb_data,
    load_orats_data,
    load_trading_logs
)

# Load QuestDB time-series data
df = load_questdb_data(
    symbol="SPY",
    start_date="2025-01-01",
    end_date="2025-01-27"
)

# Load ORATS options data
orats_data = load_orats_data(ticker="SPY", trade_date="2025-01-27")
```

### Visualization Helpers

```python
from notebooks.utils.visualizations import (
    plot_box_spread_opportunities,
    plot_performance_metrics,
    plot_feature_importance
)

# Visualize box spread opportunities
plot_box_spread_opportunities(opportunities_df)
```

### Integration with Existing Code

```python
# Use existing Python modules
from python.integration.orats_client import ORATSClient
from python.ml.feature_engineering import FeatureExtractor
from python.integration.questdb_client import QuestDBClient

# Use C++ bindings
from python.bindings import calculate_box_spread_value
```

## Best Practices

### 1. Notebook Organization

- **One notebook per analysis**: Keep notebooks focused
- **Clear naming**: Use descriptive names with dates if needed
- **Version control**: Commit notebooks regularly
- **Clear outputs**: Include outputs for reproducibility

### 2. Code Reusability

- **Extract to modules**: Move reusable code to `utils/`
- **Import existing modules**: Use existing Python packages
- **Avoid duplication**: Check `utils/` before writing new code

### 3. Documentation

- **Markdown cells**: Explain what and why, not just how
- **Clear structure**: Use headers to organize sections
- **Results interpretation**: Explain what results mean
- **Next steps**: Document follow-up actions

### 4. Development Workflow

- **Research log**: Document findings in `06-dev-workflow/research_log.ipynb`
- **Decision log**: Record important decisions in `06-dev-workflow/decision_log.ipynb`
- **Experiment tracking**: Track experiments in `06-dev-workflow/experiment_tracking.ipynb`
- **Share context**: Export notebooks as HTML/PDF for sharing

## Exporting Notebooks

### For Documentation

```bash
# Export to HTML
jupyter nbconvert --to html notebooks/01-data-exploration/orats_data_exploration.ipynb

# Export to PDF (requires LaTeX)
jupyter nbconvert --to pdf notebooks/01-data-exploration/orats_data_exploration.ipynb

# Export to Markdown
jupyter nbconvert --to markdown notebooks/01-data-exploration/orats_data_exploration.ipynb
```

### For Sharing

```bash
# Create shareable HTML with all outputs
jupyter nbconvert --to html --embed-images notebooks/analysis.ipynb

# Create presentation
jupyter nbconvert --to slides notebooks/presentation.ipynb
```

## Integration with Cursor/VS Code

### VS Code Jupyter Extension

1. Install Jupyter extension in VS Code
2. Open `.ipynb` files directly
3. Run cells with `Shift+Enter`
4. Use IntelliSense for code completion
5. Debug notebooks with breakpoints

### Cursor Integration

- Notebooks are indexed for semantic search
- Use `@notebooks` to reference notebook findings
- Link notebooks in documentation
- Use notebooks as context for AI assistance

## Troubleshooting

### Import Errors

```python
# Add project root to path
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))
```

### Data Connection Issues

```python
# Check QuestDB connection
from notebooks.utils.data_loaders import check_questdb_connection
check_questdb_connection()

# Check ORATS API
from python.integration.orats_client import ORATSClient
client = ORATSClient(api_token="your-token")
client.get_core_data("SPY")
```

### Memory Issues

```python
# Use chunked loading for large datasets
from notebooks.utils.data_loaders import load_questdb_data_chunked

for chunk in load_questdb_data_chunked(symbol="SPY", chunk_size=10000):
    # Process chunk
    process_chunk(chunk)
```

## Resources

- [JupyterLab Documentation](https://jupyterlab.readthedocs.io/)
- [VS Code Jupyter Extension](https://code.visualstudio.com/docs/datascience/jupyter-notebooks)
- [Pandas Documentation](https://pandas.pydata.org/)
- [Matplotlib Gallery](https://matplotlib.org/stable/gallery/)
- [Plotly Documentation](https://plotly.com/python/)

## Contributing

When adding new notebooks:

1. Follow the directory structure
2. Use templates from `templates/`
3. Document data sources and assumptions
4. Include example outputs
5. Update this README if adding new categories
