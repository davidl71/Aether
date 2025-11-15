# Machine Learning Module for Box Spread Trading

This module provides XGBoost-based machine learning models for enhancing box spread trading strategies.

## Overview

The ML module implements a 3-phase approach:

1. **Phase 1**: Python prototyping and model training
2. **Phase 2**: C++ integration for production inference
3. **Phase 3**: Continuous learning with periodic retraining

## Models

### 1. Profitability Prediction (Binary Classification)
Predicts whether a box spread opportunity will be profitable.

**Features**:
- Spread width and net premium
- Volume and open interest
- Implied volatility
- Market conditions (VIX, underlying price)
- Greeks (delta, gamma, theta, vega)
- Time to expiration

**Output**: Probability of profitability (0-1)

### 2. Risk Assessment (Binary Classification)
Predicts execution risk or early assignment probability.

**Features**:
- Order book depth
- Spread stability
- Time to expiration
- Dividend dates
- Earnings dates
- Historical assignment rates

**Output**: Risk score (0-1)

### 3. Execution Timing (Regression)
Determines optimal timing for order execution.

**Features**:
- Current spread width
- Recent spread history
- Volume patterns
- Time of day
- Market volatility

**Output**: Optimal wait time (seconds)

### 4. Position Sizing (Regression)
Determines optimal position size based on market conditions.

**Features**:
- Account equity
- Current portfolio risk
- Opportunity profitability
- Market volatility
- Historical success rate

**Output**: Normalized position size (0-1)

## Usage

### Training Models

```bash
# Train all models
python -m python.ml.train_models \
  --data data/training_data.json \
  --output python/ml/models

# Train specific models
python -m python.ml.train_models \
  --data data/training_data.json \
  --output python/ml/models \
  --models profitability risk
```

### Evaluating Models

```bash
python -m python.ml.evaluate_models \
  --models-dir python/ml/models \
  --test-data data/test_data.json
```

### Using Models in Python

```python
from python.ml.feature_engineering import FeatureExtractor, BoxSpreadLeg, MarketData
import xgboost as xgb
import numpy as np

# Load trained model
model = xgb.XGBClassifier()
model.load_model('python/ml/models/models/profitability_model.bin')

# Extract features
extractor = FeatureExtractor()
legs = [...]  # Your box spread legs
market_data = MarketData(...)

features = extractor.extract_features(legs, market_data)
features = features.reshape(1, -1)

# Predict
probability = model.predict_proba(features)[0, 1]
print(f"Profitability probability: {probability:.2%}")
```

## Data Format

Training data should be in JSON format:

```json
[
  {
    "legs": [
      {
        "strike": 100.0,
        "expiration": "2025-02-21T16:00:00",
        "option_type": "call",
        "bid": 2.50,
        "ask": 2.55,
        "volume": 1000,
        "open_interest": 5000,
        "implied_volatility": 0.20,
        "delta": 0.50,
        "gamma": 0.05,
        "theta": -0.10,
        "vega": 0.15
      },
      ...
    ],
    "market_data": {
      "underlying_price": 100.0,
      "risk_free_rate": 0.05,
      "current_time": "2025-01-27T10:00:00",
      "vix": 15.0,
      "market_volatility": 0.18
    },
    "historical_data": {
      "historical_profitability": 0.65,
      "success_rate": 0.70,
      "recent_spreads": [0.10, 0.12, 0.11]
    },
    "targets": {
      "profitable": 1,
      "high_risk": 0,
      "execution_time": 2.5,
      "position_size": 0.25
    }
  }
]
```

## Feature Engineering

The `FeatureExtractor` class extracts features from box spread legs and market data:

- **Spread Features**: Box value, net premium, spread width
- **Leg Features**: Volume, open interest, IV statistics
- **Market Features**: Underlying price, VIX, risk-free rate
- **Greeks Features**: Net delta, gamma, theta, vega
- **Temporal Features**: Time to expiration, time of day
- **Historical Features**: Past profitability, success rates

## Model Export

Trained models are saved in XGBoost binary format (`.bin` files) that can be loaded in C++:

```cpp
// C++ code (Phase 2)
#include <xgboost/c_api.h>

BoosterHandle booster;
XGBoosterCreate(nullptr, 0, &booster);
XGBoosterLoadModel(booster, "profitability_model.bin");
```

## Continuous Learning (Phase 3)

See `continuous_learning.py` for:
- Data collection from live trading
- Periodic model retraining
- Model versioning and A/B testing
- Performance monitoring

## Dependencies

- `xgboost>=2.0.0`: Gradient boosting library
- `scikit-learn>=1.3.0`: Model evaluation metrics
- `numpy>=1.24.0`: Numerical operations
- `optuna>=3.0.0`: Hyperparameter tuning (optional)
- `shap>=0.42.0`: Model interpretability (optional)

## Next Steps

1. **Collect Training Data**: Gather historical box spread opportunities and outcomes
2. **Train Models**: Use `train_models.py` to train initial models
3. **Evaluate**: Use `evaluate_models.py` to assess model performance
4. **Integrate C++**: Load models in C++ for production inference (Phase 2)
5. **Deploy Continuous Learning**: Set up retraining pipeline (Phase 3)

## References

- [XGBoost Documentation](https://xgboost.readthedocs.io/)
- [XGBoost Deep Research](../docs/XGBOOST_DEEP_RESEARCH.md)
- [Modeling Tools Overview](../docs/MODELING_TOOLS_OVERVIEW.md)
