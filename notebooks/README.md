# JupyterLab Notebooks

This directory contains sample Jupyter notebooks for interactive analysis, backtesting, and visualization of box spread trading strategies.

## Available Notebooks

### 1. `box_spread_analysis.ipynb`
Interactive exploration of box spread opportunities:
- Load and analyze box spread scenarios
- Visualize profit/loss heatmaps
- Compare different strike widths
- Analyze expiration date impact

### 2. `market_data_visualization.ipynb`
Market data visualization and analysis:
- Real-time price charts (candlestick)
- Volatility surface visualization
- Greeks analysis (delta, gamma, theta, vega)
- Volume and open interest trends

### 3. `questdb_exploration.ipynb`
Historical data analysis from QuestDB:
- Query historical quotes and trades
- Time-series analysis
- Performance metrics
- Pattern recognition

### 4. `backtesting_template.ipynb`
Strategy backtesting framework:
- Historical strategy testing
- Parameter optimization
- Performance analysis
- Risk metrics calculation

## Getting Started

1. **Start JupyterLab**:
   ```bash
   ./scripts/run-jupyterlab-service.sh
   # Or use the unified launcher:
   ./web/scripts/launch-all-pwa-services.sh
   ```

2. **Access JupyterLab**:
   - Open browser to `http://127.0.0.1:8888`
   - Access token will be displayed in the terminal output

3. **Open a Notebook**:
   - Navigate to the `notebooks/` directory
   - Click on any `.ipynb` file to open it
   - Run cells with `Shift+Enter`

## Project Integration

All notebooks have access to:
- **Cython Bindings**: `python/bindings/box_spread_bindings`
- **Integration Modules**: `python/integration/*`
- **Configuration**: `config/config.json` via `ConfigLoader`
- **QuestDB Client**: `python/integration/questdb_client.py`
- **ORATS Client**: `python/integration/orats_client.py`

## Example Usage

```python
# Import project modules
import sys
from pathlib import Path

# Add project to path
project_root = Path().resolve().parent
sys.path.insert(0, str(project_root / "python"))

# Import Cython bindings
from bindings.box_spread_bindings import (
    calculate_arbitrage_profit,
    calculate_roi,
    PyBoxSpreadStrategy
)

# Import integration modules
from integration.config_loader import ConfigLoader
from integration.questdb_client import QuestDBClient

# Load configuration
config = ConfigLoader.load()

# Use QuestDB for historical data
questdb = QuestDBClient()
```

## Dependencies

Install required packages:
```bash
pip install jupyterlab plotly pandas numpy matplotlib
```

For full extension support:
```bash
pip install -r requirements-jupyterlab.txt
```

For all notebook dependencies (recommended):
```bash
pip install -r requirements-notebooks.txt
```

## Import Resolution in Cursor/VS Code

The project is configured for proper import resolution in notebooks:

1. **Python Analysis Paths**: Configured in `.vscode/settings.json` to include:
   - `${workspaceFolder}/python`
   - `${workspaceFolder}/python/bindings`
   - `${workspaceFolder}/python/integration`

2. **Pyright Configuration**: `pyrightconfig.json` provides additional import resolution for:
   - Project modules
   - Notebook utilities
   - Cython bindings

3. **Notebook Dependency Checks**: All notebooks include dependency check cells that:
   - Verify required packages are installed
   - Provide clear installation instructions if missing
   - Wrap imports in try/except for better error handling

If you see import warnings in notebooks:
- Ensure you've installed `requirements-notebooks.txt`
- Select the correct Python interpreter (Cmd+Shift+P → "Python: Select Interpreter")
- Restart the Cursor/VS Code window after installing packages

## Tips

- **Save frequently**: Notebooks auto-save, but commit important work to git
- **Use markdown cells**: Document your analysis and findings
- **Clear outputs**: Before committing, clear outputs to reduce file size
- **Version control**: Use `jupyterlab-git` extension for notebook versioning
