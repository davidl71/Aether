# Jupyter Notebooks Development Workflow

## Overview

Jupyter notebooks enhance our development workflow by providing:
- **Interactive analysis**: Explore data and test ideas quickly
- **Context preservation**: Document findings with code + results
- **Knowledge sharing**: Share analysis with team members
- **Reproducible research**: Combine code, data, and documentation

## Quick Start

```bash
# Install dependencies
pip install -r requirements-notebooks.txt

# Start JupyterLab
jupyter lab

# Or use VS Code
code notebooks/
```

## Workflow Integration

### 1. Research & Exploration

**When to use**: Exploring new data sources, testing hypotheses, understanding patterns

**Workflow**:
1. Create notebook in `01-data-exploration/`
2. Load data using utilities from `notebooks.utils`
3. Analyze and visualize
4. Document findings in `06-dev-workflow/research_log.ipynb`

**Example**:
```python
# notebooks/01-data-exploration/orats_data_exploration.ipynb
from notebooks.utils.data_loaders import load_orats_data
from notebooks.utils.visualizations import plot_correlation_matrix

df = load_orats_data("SPY")
# ... analysis ...
```

### 2. Strategy Development

**When to use**: Prototyping strategies, testing parameters, visualizing opportunities

**Workflow**:
1. Create notebook in `04-strategy-dev/`
2. Use existing Python modules (e.g., `python.integration.orats_client`)
3. Test strategy logic interactively
4. Document decisions in `06-dev-workflow/decision_log.ipynb`

**Example**:
```python
# notebooks/04-strategy-dev/box_spread_analysis.ipynb
from python.integration.orats_client import ORATSClient
from python.ml.feature_engineering import FeatureExtractor

# Test strategy logic
client = ORATSClient(api_token=token)
opportunities = find_box_spreads(client, "SPY")
```

### 3. Model Development

**When to use**: Feature engineering, model training, evaluation

**Workflow**:
1. Create notebook in `03-ml-development/`
2. Use `python.ml` modules
3. Experiment with features and hyperparameters
4. Document model performance and insights

**Example**:
```python
# notebooks/03-ml-development/model_training.ipynb
from python.ml.feature_engineering import FeatureExtractor
from python.ml.train_models import train_model

# Engineer features
extractor = FeatureExtractor()
features = extractor.extract_features(legs, market_data)

# Train model
model = train_model(features, targets)
```

### 4. Backtesting

**When to use**: Validating strategies, optimizing parameters, analyzing performance

**Workflow**:
1. Create notebook in `02-backtesting/`
2. Load historical data from QuestDB or ORATS
3. Simulate strategy on historical data
4. Analyze results and document findings

**Example**:
```python
# notebooks/02-backtesting/historical_backtest.ipynb
from notebooks.utils.data_loaders import load_questdb_data
from notebooks.utils.visualizations import plot_performance_metrics

# Load historical data
df = load_questdb_data("SPY", "2025-01-01", "2025-01-27")

# Run backtest
results = backtest_strategy(df)

# Visualize
plot_performance_metrics(results)
```

## Context Sharing

### Research Log

Document findings in `notebooks/06-dev-workflow/research_log.ipynb`:

```python
from notebooks.utils.notebook_helpers import (
    create_research_log_entry,
    save_research_log_entry
)

entry = create_research_log_entry(
    title="ORATS Data Quality Analysis",
    findings="""
    - ORATS provides comprehensive options data
    - Liquidity scores correlate with execution success
    - IV percentile useful for volatility filtering
    """,
    next_steps="""
    - Integrate liquidity scores into filtering
    - Test IV percentile thresholds
    """,
    tags=["orats", "data-quality"]
)

save_research_log_entry(entry)
```

### Decision Log

Record important decisions in `notebooks/06-dev-workflow/decision_log.ipynb`:

```python
from notebooks.utils.notebook_helpers import (
    create_decision_log_entry,
    save_decision_log_entry
)

entry = create_decision_log_entry(
    decision="Use ORATS liquidity scores for option filtering",
    context="Need to filter low-liquidity options to improve execution",
    rationale="""
    - ORATS liquidity scores correlate with execution success
    - Reduces failed orders and slippage
    - Simple threshold-based filtering
    """,
    alternatives_considered=[
        "Volume-based filtering",
        "Bid-ask spread filtering",
        "Combination approach"
    ],
    impact="Expected 20% reduction in failed orders",
    tags=["filtering", "execution", "orats"]
)

save_decision_log_entry(entry)
```

### Exporting for Sharing

```bash
# Export notebook as HTML
jupyter nbconvert --to html notebooks/analysis.ipynb

# Export as PDF (requires LaTeX)
jupyter nbconvert --to pdf notebooks/analysis.ipynb

# Export as Markdown
jupyter nbconvert --to markdown notebooks/analysis.ipynb
```

## Integration with Cursor/VS Code

### Semantic Search

Cursor can index notebooks for semantic search:
- Reference findings: `@notebooks research_log`
- Search for analysis: "How did we analyze ORATS data?"
- Find decisions: "Why did we choose ORATS over other providers?"

### Code References

Link notebooks in code comments:

```python
# See notebooks/01-data-exploration/orats_data_exploration.ipynb
# for analysis of ORATS data quality

def filter_by_liquidity(options, min_score=70):
    """
    Filter options by ORATS liquidity score.

    Decision: See notebooks/06-dev-workflow/decision_log.ipynb
    Analysis: See notebooks/01-data-exploration/orats_data_exploration.ipynb
    """
    return [opt for opt in options if opt.liquidity_score >= min_score]
```

### Documentation Links

Reference notebooks in markdown docs:

```markdown
## ORATS Integration

See [ORATS Data Exploration](../notebooks/01-data-exploration/orats_data_exploration.ipynb)
for detailed analysis of ORATS data structure and quality.

Decision rationale documented in [Decision Log](../notebooks/06-dev-workflow/decision_log.ipynb).
```

## Best Practices

### 1. Notebook Organization

- **One notebook per analysis**: Keep notebooks focused
- **Clear naming**: Use descriptive names with dates if needed
- **Version control**: Commit notebooks regularly
- **Clear outputs**: Include outputs for reproducibility

### 2. Code Reusability

- **Extract to modules**: Move reusable code to `notebooks/utils/`
- **Import existing modules**: Use existing Python packages
- **Avoid duplication**: Check `utils/` before writing new code

### 3. Documentation

- **Markdown cells**: Explain what and why, not just how
- **Clear structure**: Use headers to organize sections
- **Results interpretation**: Explain what results mean
- **Next steps**: Document follow-up actions

### 4. Development Workflow

- **Research log**: Document findings in `06-dev-workflow/research_log.ipynb`
- **Decision log**: Record important decisions
- **Experiment tracking**: Track experiments and results
- **Share context**: Export notebooks as HTML/PDF for sharing

## Troubleshooting

### Import Errors

```python
# Add project root to path
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))
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
    process_chunk(chunk)
```

## Examples

See the following notebooks for examples:

- **Data Exploration**: `notebooks/01-data-exploration/orats_data_exploration.ipynb`
- **Research Log**: `notebooks/06-dev-workflow/research_log.ipynb`
- **Template**: `notebooks/templates/analysis_template.ipynb`

## Resources

- [JupyterLab Documentation](https://jupyterlab.readthedocs.io/)
- [VS Code Jupyter Extension](https://code.visualstudio.com/docs/datascience/jupyter-notebooks)
- [Pandas Documentation](https://pandas.pydata.org/)
- [Matplotlib Gallery](https://matplotlib.org/stable/gallery/)
- [Plotly Documentation](https://plotly.com/python/)
