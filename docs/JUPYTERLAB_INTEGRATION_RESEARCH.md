# JupyterLab Integration Research

**Date**: 2025-11-19
**Task**: T-122 - Research JupyterLab integration patterns
**Status**: Research Complete

---

## Executive Summary

JupyterLab can significantly enhance this IBKR box spread trading project by providing an interactive environment for strategy development, backtesting, data analysis, and visualization. The project's existing Python infrastructure (FastAPI services, Cython bindings, QuestDB integration) makes JupyterLab integration highly feasible.

---

## Local Codebase Analysis

### Existing Python Infrastructure

**Python Services Architecture:**

- Multiple FastAPI services running on separate ports:
  - Alpaca service (port 8000)
  - IB service (port 8002)
  - TradeStation service (port 8001)
  - Discount Bank service (port 8003)
  - Tastytrade service (port 8004)

- All services follow similar patterns with `/api/health` endpoints
- Configuration system via `config/config.json` with service port definitions

**Python Integration Points:**

- **Cython Bindings**: `python/bindings/` - Exposes C++ calculations to Python
  - `PyBoxSpreadStrategy`, `PyBoxSpreadLeg`, `PyOptionContract`
  - `calculate_arbitrage_profit()`, `calculate_roi()`

- **NautilusTrader Integration**: `python/integration/nautilus_client.py`
- **Data Clients**: Multiple market data providers (ORATS, IBKR Portal, QuestDB)
- **Strategy Runner**: `python/integration/strategy_runner.py` - Main strategy loop

**Data Sources:**

- **QuestDB**: Time-series database for quotes and trades (`questdb_client.py`)
- **ORATS**: Options data API (`orats_client.py`)
- **IBKR Portal**: Account and portfolio snapshots (`ibkr_portal_client.py`)
- **Ledger System**: Rust-based ledger for transaction tracking

**Visualization:**

- Web PWA has basic sparkline visualizations (SVG-based)
- No advanced charting or analysis tools currently
- Data displayed in tabular format with simple overlays

### Key Integration Opportunities

1. **Direct Python Module Access**: All integration modules are importable
2. **Service API Access**: Services expose REST endpoints for data access
3. **QuestDB Query Interface**: Can query historical market data
4. **Configuration System**: Shared config file for all services

---

## Internet Research (2025)

### JupyterLab Core Capabilities

🔗 **[Jupyter.org - JupyterLab Overview](https://jupyter.org/)**

- **Found via web search**: Official JupyterLab documentation
- **Key Insights**:
  - Web-based interactive development environment
  - Supports 40+ programming languages via kernels
  - Modular extension system for customization
  - Built-in file browser, terminal, and notebook interface

- **Applicable to Task**: Core platform for integration

🔗 **[AWS Machine Learning Blog - Jupyter AI Extensions](https://aws.amazon.com/blogs/machine-learning/announcing-new-jupyter-contributions-by-aws-to-democratize-generative-ai-and-scale-ml-workloads/)**

- **Found via web search**: AWS contributions to Jupyter ecosystem
- **Key Insights**:
  - Jupyter AI extension for generative AI capabilities
  - Code generation and debugging assistance
  - Integration with cloud ML services

- **Applicable to Task**: Could enhance strategy development workflow

🔗 **[D4Science - JupyterLab Shared Workspaces](https://www.d4science.org/services/jupyterlab)**

- **Found via web search**: Multi-user JupyterLab deployment
- **Key Insights**:
  - Shared cloud storage capabilities
  - Single sign-on (SSO) integration
  - Collaborative development features

- **Applicable to Task**: Multi-user deployment patterns

🔗 **[SemanticGIS - JupyterLab Visualization](https://semanticgis.org/Geospatial-Technology-Stack/Platforms--and--Applications/Interactive-Computing-Environments/JupyterLab)**

- **Found via web search**: JupyterLab visualization capabilities
- **Key Insights**:
  - Integration with Matplotlib, Plotly, Folium
  - Interactive charts and maps
  - Rich data visualization ecosystem

- **Applicable to Task**: Critical for trading data visualization

### Integration Patterns

**Standalone Service Pattern:**

- Deploy JupyterLab as separate service (similar to existing FastAPI services)
- Access via dedicated port (e.g., 8888)
- Can share Python environment with existing services
- Benefits: Isolation, independent scaling, easy deployment

**Embedded Pattern:**

- Integrate JupyterLab into existing PWA
- Use JupyterLab's iframe embedding capabilities
- Share authentication with main application
- Benefits: Unified interface, shared session

**Kernel-Based Pattern:**

- Configure Python kernel with project dependencies
- Access Cython bindings directly in notebooks
- Import integration modules for live data
- Benefits: Direct code access, no API overhead

---

## Specific Use Cases for This Project

### 1. Strategy Backtesting and Analysis

**Current State**: ORATS integration document mentions backtesting as "Low Priority" (Phase 4)
**JupyterLab Value**:

- Interactive backtesting notebooks
- Historical data analysis from QuestDB
- Parameter optimization with visual feedback
- Performance metrics visualization

**Implementation**:

```python

# Example notebook cell

from python.integration.questdb_client import QuestDBClient
from python.integration.orats_client import ORATSClient
from python.bindings.box_spread_bindings import calculate_arbitrage_profit

# Query historical data

questdb = QuestDBClient()

# Analyze and visualize
```

### 2. Market Data Visualization

**Current State**: Web PWA has basic sparklines, no advanced charts
**JupyterLab Value**:

- Interactive candlestick charts (Plotly)
- Volatility surface visualization
- Greeks analysis (delta, gamma, theta, vega)
- Real-time data streaming visualization

**Libraries**: Plotly, Matplotlib, Bokeh

### 3. Risk Analysis and Portfolio Optimization

**Current State**: Risk management config exists, but no analysis tools
**JupyterLab Value**:

- Portfolio risk metrics calculation
- Correlation analysis
- Position sizing optimization
- Drawdown analysis

### 4. Research and Development

**Current State**: Strategy logic in C++ and Python, hard to experiment
**JupyterLab Value**:

- Interactive strategy prototyping
- Test new calculation methods
- Compare different approaches side-by-side
- Document research findings inline

### 5. Data Exploration

**Current State**: QuestDB stores data but no query interface
**JupyterLab Value**:

- SQL queries against QuestDB
- Data quality checks
- Anomaly detection
- Pattern recognition

### 6. Box Spread Opportunity Analysis

**Current State**: Scenarios displayed in tables
**JupyterLab Value**:

- Interactive scenario explorer
- Profit/loss heatmaps
- Strike width analysis
- Expiration date optimization

---

## Technical Integration Approaches

### Approach 1: Standalone JupyterLab Service (Recommended)

**Architecture**:

- New service: `python/integration/jupyterlab_service.py`
- Port: 8888 (configurable via `config.json`)
- Launch script: `scripts/run-jupyterlab-service.sh`
- Integration with `launch-all-pwa-services.sh`

**Benefits**:

- Matches existing service pattern
- Easy to add to current infrastructure
- Independent scaling and management
- Can share Python virtual environment

**Configuration Addition**:

```json
"services": {
  "jupyterlab": {
    "port": 8888
  }
}
```

### Approach 2: Embedded in PWA

**Architecture**:

- JupyterLab iframe in React app
- Shared authentication
- Proxy requests through main web server

**Benefits**:

- Unified user experience
- Single entry point
- Shared session management

**Challenges**:

- More complex integration
- CORS configuration
- Session management complexity

### Approach 3: Development-Only Installation

**Architecture**:

- Local JupyterLab installation
- Developers run manually
- Not part of production deployment

**Benefits**:

- Simplest implementation
- No infrastructure changes
- Developer-focused tool

**Limitations**:

- Not accessible to non-developers
- No shared notebooks
- Manual setup required

---

## Python Kernel Configuration

### Required Dependencies

**Core JupyterLab**:

```bash
pip install jupyterlab
```

**Trading-Specific Extensions**:

```bash
pip install jupyterlab-git  # Version control
pip install jupyterlab-widgets  # Interactive widgets
```

**Data Analysis Libraries** (likely already installed):

```bash
pip install pandas numpy matplotlib plotly
```

**Project Integration**:

- Cython bindings: `python/bindings/` (already built)
- Integration modules: `python/integration/` (already available)
- Configuration loader: `python/integration/config_loader.py`

### Kernel Setup

**Custom Kernel Configuration**:

```python

# .ipython/profile_default/startup/00-project-init.py

import sys
from pathlib import Path

# Add project to path

project_root = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(project_root / "python"))

# Import common modules

from integration.config_loader import ConfigLoader
from integration.questdb_client import QuestDBClient

# Load config

config = ConfigLoader.load()
```

---

## Security Considerations

### Trading Data Security

- **Authentication**: Require login for JupyterLab access
- **Network Isolation**: Run on localhost or VPN-only access
- **Credential Management**: Never store API keys in notebooks
- **Read-Only Mode**: Option for read-only notebooks in production

### Best Practices

- Use environment variables for sensitive data
- Implement notebook signing/validation
- Audit notebook execution logs
- Restrict file system access

---

## Deployment Options

### Option 1: Development Tool

- Install locally for developers
- Not part of production deployment
- Manual launch: `jupyter lab`

### Option 2: Service Integration

- Add to service launch script
- Accessible at `http://localhost:8888`
- Integrated with existing service management

### Option 3: Docker Container

- Containerized JupyterLab instance
- Consistent environment across deployments
- Easy scaling and management

---

## Recommended Implementation Path

### Phase 1: Proof of Concept (Week 1)

1. Install JupyterLab locally
2. Create sample notebook accessing Cython bindings
3. Test QuestDB data querying
4. Create basic visualization example

### Phase 2: Service Integration (Week 2)

1. Create JupyterLab service wrapper
2. Add to configuration system
3. Integrate with launch scripts
4. Test multi-service deployment

### Phase 3: Notebook Templates (Week 3)

1. Strategy backtesting template
2. Market data visualization template
3. Risk analysis template
4. Box spread opportunity explorer

### Phase 4: Advanced Features (Week 4+)

1. Real-time data streaming
2. Interactive widgets
3. Custom extensions
4. Collaboration features

---

## Synthesis & Recommendation

**Recommended Approach**: **Standalone JupyterLab Service**

**Justification**:

1. **Matches Existing Architecture**: Follows same pattern as other Python services
2. **Easy Integration**: Minimal changes to existing infrastructure
3. **Flexible Deployment**: Can be enabled/disabled via configuration
4. **Developer Experience**: Familiar workflow for data scientists
5. **Scalability**: Can be deployed separately if needed

**Key Benefits**:

- Interactive strategy development and testing
- Rich data visualization capabilities
- Historical data analysis from QuestDB
- Research and documentation in one place
- Easy sharing of analysis notebooks

**Next Steps**:

1. Design detailed architecture (Task T-123)
2. Create service wrapper implementation
3. Add configuration support
4. Build sample notebooks
5. Integrate with launch scripts

---

## Relevant JupyterLab Extensions & Frameworks

### Essential Extensions for Trading/Finance

**1. JupyterLab Git** (`jupyterlab-git`)

- **Purpose**: Version control integration for notebooks
- **Installation**: `pip install jupyterlab-git`
- **Use Case**: Track changes to strategy notebooks, collaborate on analysis
- **Reference**: [JupyterLab Git Extension](https://github.com/jupyterlab/jupyterlab-git)

**2. JupyterLab Widgets** (`ipywidgets`, `jupyterlab-widgets`)

- **Purpose**: Interactive widgets for real-time data visualization
- **Installation**: `pip install ipywidgets jupyterlab-widgets`
- **Use Case**: Interactive parameter tuning, real-time market data displays
- **Reference**: [IPyWidgets Documentation](https://ipywidgets.readthedocs.io/)

**3. JupyterLab Resource Usage** (`jupyter-resource-usage`)

- **Purpose**: Monitor CPU and memory usage
- **Installation**: `pip install jupyter-resource-usage`
- **Use Case**: Monitor resource consumption during backtesting
- **Reference**: [Jupyter Resource Usage](https://github.com/jupyter-server/jupyter-resource-usage)

**4. JupyterLab Scheduler** (`jupyterlab-scheduler`)

- **Purpose**: Schedule notebook executions
- **Installation**: `pip install jupyterlab-scheduler`
- **Use Case**: Automated daily backtesting, scheduled data analysis
- **Reference**: [JupyterLab Scheduler](https://github.com/jupyterlab/jupyterlab-scheduler)

### Data Visualization Extensions

**5. bqplot** (`bqplot`)

- **Purpose**: Interactive plotting library for Jupyter
- **Installation**: `pip install bqplot`
- **Use Case**: Interactive financial charts, real-time updates
- **Reference**: [bqplot GitHub](https://github.com/bqplot/bqplot)

**6. PyGWalker** (`pygwalker`)

- **Purpose**: Transform pandas DataFrames into interactive visual analytics
- **Installation**: `pip install pygwalker`
- **Use Case**: Quick data exploration of market data, option chains
- **Reference**: [PyGWalker Documentation](https://docs.kanaries.net/pygwalker)

**7. Plotly** (`plotly`, `jupyterlab-plotly`)

- **Purpose**: Interactive, publication-quality graphs
- **Installation**: `pip install plotly jupyterlab-plotly`
- **Use Case**: Candlestick charts, volatility surfaces, performance dashboards
- **Reference**: [Plotly for Financial Charts](https://plotly.com/python/financial-charts/)

### Trading-Specific Tools

**8. Trading Strategy Notebooks** (`notebooks_invest`)

- **Purpose**: Collection of Jupyter notebooks for market analysis
- **Installation**: Clone from [GitHub](https://github.com/danielsobrado/notebooks_invest)
- **Use Case**: Reference implementations, learning examples
- **Reference**: [notebooks_invest GitHub](https://github.com/danielsobrado/notebooks_invest)

**9. Trading Strategy Framework** (TradingStrategy.ai)

- **Purpose**: Backtesting framework with Jupyter integration
- **Installation**: `pip install trading-strategy`
- **Use Case**: Professional backtesting infrastructure
- **Reference**: [TradingStrategy.ai Documentation](https://tradingstrategy.ai/docs/programming/learn.html)

### Integration Extensions

**10. JupyterLab APIBaker** (`jupyterlab-apibaker`)

- **Purpose**: Create secured APIs from notebook functions
- **Installation**: `pip install jupyterlab_apibaker`
- **Use Case**: Expose FastAPI service functions interactively
- **Reference**: [JupyterLab APIBaker](https://pypi.org/project/jupyterlab-apibaker/)

**11. QuestDB Integration**

- **Purpose**: Direct QuestDB query interface
- **Installation**: `pip install questdb`
- **Use Case**: Query historical market data from notebooks
- **Reference**: [QuestDB Python Client](https://questdb.io/docs/reference/api/python/)

### Recommended Extension Installation

Create `requirements-jupyterlab.txt`:

```txt

# Core JupyterLab

jupyterlab>=4.0.0

# Essential Extensions

jupyterlab-git>=0.45.0
ipywidgets>=8.1.0
jupyterlab-widgets>=3.0.0
jupyter-resource-usage>=0.6.0

# Visualization

plotly>=5.18.0
jupyterlab-plotly>=5.18.0
bqplot>=0.12.0
pygwalker>=0.3.0

# Data Analysis

pandas>=2.0.0
numpy>=1.24.0
matplotlib>=3.7.0

# Trading-Specific

questdb>=1.0.0

# Optional: Advanced

jupyterlab-scheduler>=0.1.0
jupyterlab-apibaker>=0.1.0
```

Install with:

```bash
pip install -r requirements-jupyterlab.txt
```

## References

- [JupyterLab Official Documentation](https://jupyter.org/)
- [JupyterLab Extensions](https://jupyterlab.readthedocs.io/en/stable/user/extensions.html)
- [Plotly for Financial Charts](https://plotly.com/python/financial-charts/)
- [Pandas for Financial Data](https://pandas.pydata.org/docs/user_guide/timeseries.html)
- [QuestDB Python Client](https://questdb.io/docs/reference/api/python/)
- [bqplot Interactive Plotting](https://github.com/bqplot/bqplot)
- [IPyWidgets Documentation](https://ipywidgets.readthedocs.io/)
