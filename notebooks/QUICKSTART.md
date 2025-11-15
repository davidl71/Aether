# Notebooks Quick Start

Get started with Jupyter notebooks in 3 steps:

## 1. Install Dependencies

```bash
# Run setup script
./scripts/setup_notebooks.sh

# Or manually
pip install -r requirements-notebooks.txt
```

## 2. Start Jupyter

```bash
# Option A: JupyterLab (recommended)
jupyter lab

# Option B: VS Code
code notebooks/
```

## 3. Open a Notebook

Start with one of these:

- **Data Exploration**: `01-data-exploration/orats_data_exploration.ipynb`
- **Research Log**: `06-dev-workflow/research_log.ipynb`
- **Template**: `templates/analysis_template.ipynb`

## Example: Quick Analysis

```python
# In a notebook cell:
from notebooks.utils.notebook_helpers import setup_notebook_environment
from notebooks.utils.data_loaders import load_orats_data, load_config

# Setup
env_info = setup_notebook_environment()
config = load_config()

# Load data
df = load_orats_data("SPY", api_token=config.get('orats', {}).get('api_token'))

# Explore
print(df.head())
print(df.describe())
```

## Next Steps

- Read [README.md](README.md) for detailed usage
- See [docs/NOTEBOOKS_WORKFLOW.md](../docs/NOTEBOOKS_WORKFLOW.md) for workflow integration
- Check [templates/](templates/) for notebook templates

## Troubleshooting

**Import errors?**
```python
import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd().parent))
```

**Can't connect to QuestDB/ORATS?**
- Check that services are running
- Verify API tokens in `config/config.json` or environment variables
- See `notebooks/utils/data_loaders.py` for connection helpers
