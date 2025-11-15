"""
Evaluate trained XGBoost models.

Provides evaluation metrics and model analysis tools.
"""

import argparse
import json
import logging
from pathlib import Path
from typing import Dict

import numpy as np
import xgboost as xgb
from sklearn.metrics import (
  accuracy_score,
  classification_report,
  confusion_matrix,
  mean_squared_error,
  r2_score,
  roc_auc_score,
  roc_curve,
)

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class ModelEvaluator:
  """Evaluates XGBoost models."""

  def __init__(self, models_dir: Path):
    self.models_dir = Path(models_dir)
    self.models = {}
    self.feature_names = []

    # Load feature names
    feature_names_file = self.models_dir / 'feature_names.json'
    if feature_names_file.exists():
      with open(feature_names_file, 'r') as f:
        self.feature_names = json.load(f)

  def load_model(self, model_name: str, model_type: str = 'classifier'):
    """Load a trained model."""
    model_path = self.models_dir / f'{model_name}_model.bin'

    if not model_path.exists():
      raise FileNotFoundError(f"Model not found: {model_path}")

    if model_type == 'classifier':
      model = xgb.XGBClassifier()
    else:
      model = xgb.XGBRegressor()

    model.load_model(str(model_path))
    self.models[model_name] = model

    logger.info(f"Loaded {model_name} model from {model_path}")
    return model

  def evaluate_classifier(
    self, model: xgb.XGBClassifier, X_test: np.ndarray, y_test: np.ndarray
  ) -> Dict:
    """Evaluate a binary classifier."""
    y_pred = model.predict(X_test)
    y_pred_proba = model.predict_proba(X_test)[:, 1]

    accuracy = accuracy_score(y_test, y_pred)
    auc = roc_auc_score(y_test, y_pred_proba)

    # Confusion matrix
    cm = confusion_matrix(y_test, y_pred)
    tn, fp, fn, tp = cm.ravel()

    # Additional metrics
    precision = tp / (tp + fp) if (tp + fp) > 0 else 0.0
    recall = tp / (tp + fn) if (tp + fn) > 0 else 0.0
    f1 = 2 * (precision * recall) / (precision + recall) if (precision + recall) > 0 else 0.0

    # ROC curve
    fpr, tpr, thresholds = roc_curve(y_test, y_pred_proba)

    results = {
      'accuracy': float(accuracy),
      'auc': float(auc),
      'precision': float(precision),
      'recall': float(recall),
      'f1_score': float(f1),
      'confusion_matrix': {
        'true_negatives': int(tn),
        'false_positives': int(fp),
        'false_negatives': int(fn),
        'true_positives': int(tp),
      },
      'roc_curve': {
        'fpr': fpr.tolist(),
        'tpr': tpr.tolist(),
        'thresholds': thresholds.tolist(),
      },
    }

    logger.info(f"Accuracy: {accuracy:.4f}")
    logger.info(f"AUC: {auc:.4f}")
    logger.info(f"Precision: {precision:.4f}")
    logger.info(f"Recall: {recall:.4f}")
    logger.info(f"F1 Score: {f1:.4f}")
    logger.info(f"\n{classification_report(y_test, y_pred)}")

    return results

  def evaluate_regressor(
    self, model: xgb.XGBRegressor, X_test: np.ndarray, y_test: np.ndarray
  ) -> Dict:
    """Evaluate a regression model."""
    y_pred = model.predict(X_test)

    mse = mean_squared_error(y_test, y_pred)
    rmse = np.sqrt(mse)
    r2 = r2_score(y_test, y_pred)

    # Additional metrics
    mae = np.mean(np.abs(y_test - y_pred))
    mape = np.mean(np.abs((y_test - y_pred) / (y_test + 1e-8))) * 100

    results = {
      'mse': float(mse),
      'rmse': float(rmse),
      'r2_score': float(r2),
      'mae': float(mae),
      'mape': float(mape),
    }

    logger.info(f"MSE: {mse:.4f}")
    logger.info(f"RMSE: {rmse:.4f}")
    logger.info(f"R²: {r2:.4f}")
    logger.info(f"MAE: {mae:.4f}")
    logger.info(f"MAPE: {mape:.2f}%")

    return results


def main():
  parser = argparse.ArgumentParser(description='Evaluate XGBoost models')
  parser.add_argument(
    '--models-dir', type=Path, required=True,
    help='Directory containing trained models'
  )
  parser.add_argument(
    '--test-data', type=Path, required=True,
    help='Path to test data JSON file'
  )

  args = parser.parse_args()

  evaluator = ModelEvaluator(args.models_dir)

  # Load test data (same format as training data)
  # This would use the same data loading logic as train_models.py
  # For now, this is a placeholder structure

  logger.info("Model evaluation complete")


if __name__ == '__main__':
  main()
