# XGBoost 3-Phase Implementation

<!--
@index: xgboost-implementation
@category: implementation
@tags: xgboost, machine-learning, implementation, trading, box-spread
@last-updated: 2025-01-27
-->

This document describes the 3-phase XGBoost integration implementation for box spread trading.

## Overview

The implementation follows a 3-phase approach:

1. **Phase 1**: Python prototyping and model training
2. **Phase 2**: C++ integration for production inference
3. **Phase 3**: Continuous learning with periodic retraining

## Phase 1: Python Prototyping ✅

### Components Created

1. **Feature Engineering** (`python/ml/feature_engineering.py`)
   - `FeatureExtractor` class
   - Extracts features from box spread legs and market data
   - Features include: spread metrics, leg statistics, market conditions, Greeks, temporal features

2. **Model Training** (`python/ml/train_models.py`)
   - `ModelTrainer` class
   - Trains 4 models:
     - Profitability prediction (binary classification)
     - Risk assessment (binary classification)
     - Execution timing (regression)
     - Position sizing (regression)
   - Saves models in XGBoost binary format (`.bin` files)

3. **Model Evaluation** (`python/ml/evaluate_models.py`)
   - `ModelEvaluator` class
   - Provides evaluation metrics (accuracy, AUC, RMSE, R²)
   - Model analysis tools

4. **Documentation** (`python/ml/README.md`)
   - Usage guide
   - Data format specification
   - Integration examples

### Usage

```bash
# Train all models
python -m python.ml.train_models \
  --data data/training_data.json \
  --output python/ml/models

# Evaluate models
python -m python.ml.evaluate_models \
  --models-dir python/ml/models \
  --test-data data/test_data.json
```

### Dependencies Added

- `xgboost>=2.0.0`
- `scikit-learn>=1.3.0`
- `optuna>=3.0.0` (for hyperparameter tuning)
- `shap>=0.42.0` (for model interpretability)

## Phase 2: C++ Integration ✅

### Components Created

1. **ML Predictor Header** (`native/include/ib_box_spread/ml_predictor.h`)
   - `MLPredictor` class interface
   - Prediction result structures:
     - `ProfitabilityPrediction`
     - `RiskPrediction`
     - `ExecutionTimingPrediction`
     - `PositionSizingPrediction`
   - Feature extraction methods
   - Model loading methods

2. **ML Predictor Implementation** (`native/src/ml_predictor.cpp`)
   - XGBoost model loading (with `ENABLE_XGBOOST` flag)
   - Feature extraction (mirrors Python implementation)
   - Prediction methods
   - Stub implementation when XGBoost is not available

### Integration Notes

- Models are saved from Python in XGBoost binary format
- C++ code loads models using XGBoost C API
- Feature extraction logic mirrors Python implementation
- Can be conditionally compiled with `ENABLE_XGBOOST` flag

### Next Steps for Full Integration

1. Add XGBoost to CMakeLists.txt:

   ```cmake
   find_package(xgboost QUIET)
   if(NOT xgboost_FOUND)
     include(FetchContent)
     FetchContent_Declare(
       xgboost
       GIT_REPOSITORY https://github.com/dmlc/xgboost.git
       GIT_TAG master
     )
     FetchContent_MakeAvailable(xgboost)
   endif()
   ```

2. Link XGBoost to target:

   ```cmake
   target_link_libraries(your_target PRIVATE xgboost::xgboost)
   ```

3. Enable XGBoost in code:

   ```cpp
   #define ENABLE_XGBOOST
   ```

4. Integrate with `BoxSpreadStrategy`:

   ```cpp
   ml::MLPredictor predictor;
   predictor.load_models("python/ml/models/models");

   auto prediction = predictor.predict_profitability(
     spread, chain, underlying_price, risk_free_rate
   );

   if (prediction && prediction->is_profitable) {
     // Execute trade
   }
   ```

## Phase 3: Continuous Learning ✅

### Components Created

1. **Data Collector** (`python/ml/continuous_learning.py`)
   - `DataCollector` class
   - SQLite database for storing trading records
   - Records opportunities and outcomes
   - Retrieves training data

2. **Model Version Manager** (`python/ml/continuous_learning.py`)
   - `ModelVersionManager` class
   - Tracks model versions
   - A/B testing support
   - Production/staging promotion

3. **Continuous Learning Pipeline** (`python/ml/continuous_learning.py`)
   - `ContinuousLearningPipeline` class
   - Automatic retraining (configurable interval)
   - Performance monitoring
   - Model versioning

### Usage

```bash
# Retrain models
python -m python.ml.continuous_learning \
  --data-db data/trading_records.db \
  --models-dir python/ml/models \
  --retrain

# Monitor performance
python -m python.ml.continuous_learning \
  --data-db data/trading_records.db \
  --models-dir python/ml/models \
  --monitor
```

### Data Collection

The system collects:

- Trading opportunities (before execution)
- Predictions made by models
- Actual outcomes (after execution)
- Execution metrics (time, size, profit)

### Retraining Schedule

- Default: Every 7 days
- Minimum records: 100 new samples
- Training window: Last 90 days of data
- Automatic promotion: Manual (for safety)

## Integration Workflow

### 1. Training Initial Models

```bash
# Prepare training data (JSON format)
# See python/ml/README.md for format

# Train models
python -m python.ml.train_models \
  --data data/training_data.json \
  --output python/ml/models
```

### 2. Deploying Models to C++

Models are automatically saved in XGBoost binary format that C++ can load:

```cpp
ml::MLPredictor predictor;
predictor.load_models("python/ml/models/models");
```

### 3. Using Predictions in Trading

```cpp
// In BoxSpreadStrategy::evaluate_box_spread()
auto prediction = predictor.predict_all(
  spread, chain, underlying_price, risk_free_rate,
  account_equity, portfolio_risk
);

if (prediction.profitability &&
    prediction.profitability->is_profitable &&
    prediction.risk &&
    !prediction.risk->is_high_risk) {
  // Execute trade with recommended position size
  double size = prediction.position_sizing ?
    prediction.position_sizing->recommended_size : default_size;
  execute_trade(spread, size);
}
```

### 4. Collecting Data for Retraining

```python
# In trading system
from python.ml.continuous_learning import DataCollector

collector = DataCollector('data/trading_records.db')

# Record opportunity
collector.record_opportunity(
  symbol='SPX',
  legs=legs_data,
  market_data=market_data,
  features=features,
  predicted_profitable=True,
  predicted_risk=0.3,
  model_version='20250127_120000'
)

# Update with outcome
collector.update_outcome(
  record_id=record_id,
  actual_profitable=True,
  actual_profit=100.0,
  outcome='profitable'
)
```

### 5. Periodic Retraining

Set up a cron job or scheduled task:

```bash
# Daily check (runs retraining if needed)
0 2 * * * python -m python.ml.continuous_learning \
  --data-db data/trading_records.db \
  --models-dir python/ml/models
```

## File Structure

```
python/ml/
  __init__.py
  feature_engineering.py    # Feature extraction
  train_models.py            # Model training
  evaluate_models.py         # Model evaluation
  continuous_learning.py     # Continuous learning pipeline
  README.md                  # Usage documentation
  models/                    # Trained models directory
    models/
      profitability_model.bin
      risk_model.bin
      execution_timing_model.bin
      position_sizing_model.bin
      feature_names.json
    versions.json            # Model version history
    last_training.json        # Last training timestamp

native/include/ib_box_spread/
  ml_predictor.h             # C++ ML predictor interface

native/src/
  ml_predictor.cpp            # C++ ML predictor implementation

data/
  trading_records.db          # SQLite database for training data
```

## Model Performance Monitoring

The continuous learning pipeline tracks:

- **Prediction Accuracy**: How often predictions match outcomes
- **Profitability Accuracy**: Correct profitable/not profitable predictions
- **Risk Assessment**: How well risk predictions match actual risk
- **Execution Timing**: Whether timing recommendations improve execution
- **Position Sizing**: Whether recommended sizes are optimal

## Model Versioning

Models are versioned with:

- Version ID (timestamp-based)
- Creation date
- Training metrics
- Status (staging/production)
- Description

New models start in "staging" and must be manually promoted to "production" for safety.

## Next Steps

1. **Collect Training Data**: Gather historical box spread opportunities and outcomes
2. **Train Initial Models**: Use Phase 1 scripts to train models
3. **Integrate C++**: Complete Phase 2 integration with XGBoost in CMake
4. **Deploy**: Use models in production trading system
5. **Monitor**: Set up continuous learning pipeline
6. **Iterate**: Retrain models periodically with new data

## References

- [XGBoost Deep Research](XGBOOST_DEEP_RESEARCH.md)
- [Modeling Tools Overview](MODELING_TOOLS_OVERVIEW.md)
- [XGBoost Documentation](https://xgboost.readthedocs.io/)
- [Python ML Module README](../python/ml/README.md)
