"""
Train XGBoost models for box spread trading.

This script trains multiple models:
1. Profitability prediction (binary classification)
2. Risk assessment (binary classification)
3. Execution timing (regression)
4. Position sizing (regression)
"""

import argparse
import json
import logging
from pathlib import Path
from typing import Dict, Tuple

import numpy as np
import xgboost as xgb
from sklearn.metrics import (
    accuracy_score,
    classification_report,
    mean_squared_error,
    r2_score,
    roc_auc_score,
)
from sklearn.model_selection import train_test_split

from .feature_engineering import BoxSpreadLeg, FeatureExtractor, MarketData

logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


class ModelTrainer:
    """Trains XGBoost models for box spread trading."""

    def __init__(self, output_dir: Path):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.feature_extractor = FeatureExtractor()
        self.models = {}
        self.feature_names = []

    def prepare_data(self, data_file: Path) -> Tuple[np.ndarray, Dict[str, np.ndarray]]:
        """
        Load and prepare training data.

        Args:
          data_file: Path to JSON file with training data

        Returns:
          Tuple of (features, targets_dict) where targets_dict contains:
            - profitability: Binary (1 = profitable, 0 = not)
            - risk: Binary (1 = high risk, 0 = low risk)
            - execution_time: Float (seconds to execute)
            - position_size: Float (normalized position size)
        """
        logger.info(f"Loading data from {data_file}")

        with open(data_file, "r") as f:
            data = json.load(f)

        features_list = []
        profitability = []
        risk = []
        execution_time = []
        position_size = []

        for sample in data:
            # Extract features
            legs = [BoxSpreadLeg(**leg_data) for leg_data in sample["legs"]]
            market_data = MarketData(**sample["market_data"])
            historical = sample.get("historical_data")

            try:
                features = self.feature_extractor.extract_features(
                    legs, market_data, historical
                )
                features_list.append(features)

                # Extract targets
                profitability.append(sample["targets"]["profitable"])
                risk.append(sample["targets"]["high_risk"])
                execution_time.append(sample["targets"].get("execution_time", 0.0))
                position_size.append(sample["targets"].get("position_size", 0.0))
            except Exception as e:
                logger.warning(f"Skipping sample due to error: {e}")
                continue

        X = np.array(features_list)
        self.feature_names = self.feature_extractor.get_feature_names()

        targets = {
            "profitability": np.array(profitability),
            "risk": np.array(risk),
            "execution_time": np.array(execution_time),
            "position_size": np.array(position_size),
        }

        logger.info(f"Loaded {len(X)} samples with {len(self.feature_names)} features")
        return X, targets

    def train_profitability_model(
        self, X: np.ndarray, y: np.ndarray, test_size: float = 0.2
    ) -> xgb.XGBClassifier:
        """Train binary classifier for profitability prediction."""
        logger.info("Training profitability model...")

        X_train, X_test, y_train, y_test = train_test_split(
            X, y, test_size=test_size, random_state=42, stratify=y
        )

        # XGBoost parameters optimized for binary classification
        params = {
            "objective": "binary:logistic",
            "eval_metric": "auc",
            "max_depth": 6,
            "learning_rate": 0.1,
            "n_estimators": 200,
            "subsample": 0.8,
            "colsample_bytree": 0.8,
            "min_child_weight": 3,
            "gamma": 0.1,
            "reg_alpha": 0.1,  # L1
            "reg_lambda": 1.0,  # L2
            "random_state": 42,
            "n_jobs": -1,
        }

        model = xgb.XGBClassifier(**params)
        model.fit(
            X_train,
            y_train,
            eval_set=[(X_test, y_test)],
            early_stopping_rounds=20,
            verbose=False,
        )

        # Evaluate
        y_pred = model.predict(X_test)
        y_pred_proba = model.predict_proba(X_test)[:, 1]

        accuracy = accuracy_score(y_test, y_pred)
        auc = roc_auc_score(y_test, y_pred_proba)

        logger.info(f"Profitability model - Accuracy: {accuracy:.4f}, AUC: {auc:.4f}")
        logger.info(f"\n{classification_report(y_test, y_pred)}")

        # Save feature importance
        self._save_feature_importance(
            model,
            "profitability",
            self.output_dir / "profitability_feature_importance.json",
        )

        return model

    def train_risk_model(
        self, X: np.ndarray, y: np.ndarray, test_size: float = 0.2
    ) -> xgb.XGBClassifier:
        """Train binary classifier for risk assessment."""
        logger.info("Training risk model...")

        X_train, X_test, y_train, y_test = train_test_split(
            X, y, test_size=test_size, random_state=42, stratify=y
        )

        params = {
            "objective": "binary:logistic",
            "eval_metric": "auc",
            "max_depth": 5,
            "learning_rate": 0.1,
            "n_estimators": 200,
            "subsample": 0.8,
            "colsample_bytree": 0.8,
            "min_child_weight": 5,  # More conservative for risk
            "gamma": 0.2,
            "reg_alpha": 0.1,
            "reg_lambda": 1.0,
            "random_state": 42,
            "n_jobs": -1,
        }

        model = xgb.XGBClassifier(**params)
        model.fit(
            X_train,
            y_train,
            eval_set=[(X_test, y_test)],
            early_stopping_rounds=20,
            verbose=False,
        )

        # Evaluate
        y_pred = model.predict(X_test)
        y_pred_proba = model.predict_proba(X_test)[:, 1]

        accuracy = accuracy_score(y_test, y_pred)
        auc = roc_auc_score(y_test, y_pred_proba)

        logger.info(f"Risk model - Accuracy: {accuracy:.4f}, AUC: {auc:.4f}")
        logger.info(f"\n{classification_report(y_test, y_pred)}")

        self._save_feature_importance(
            model, "risk", self.output_dir / "risk_feature_importance.json"
        )

        return model

    def train_execution_timing_model(
        self, X: np.ndarray, y: np.ndarray, test_size: float = 0.2
    ) -> xgb.XGBRegressor:
        """Train regression model for execution timing."""
        logger.info("Training execution timing model...")

        X_train, X_test, y_train, y_test = train_test_split(
            X, y, test_size=test_size, random_state=42
        )

        params = {
            "objective": "reg:squarederror",
            "eval_metric": "rmse",
            "max_depth": 6,
            "learning_rate": 0.1,
            "n_estimators": 200,
            "subsample": 0.8,
            "colsample_bytree": 0.8,
            "min_child_weight": 3,
            "gamma": 0.1,
            "reg_alpha": 0.1,
            "reg_lambda": 1.0,
            "random_state": 42,
            "n_jobs": -1,
        }

        model = xgb.XGBRegressor(**params)
        model.fit(
            X_train,
            y_train,
            eval_set=[(X_test, y_test)],
            early_stopping_rounds=20,
            verbose=False,
        )

        # Evaluate
        y_pred = model.predict(X_test)
        rmse = np.sqrt(mean_squared_error(y_test, y_pred))
        r2 = r2_score(y_test, y_pred)

        logger.info(f"Execution timing model - RMSE: {rmse:.4f}, R²: {r2:.4f}")

        self._save_feature_importance(
            model,
            "execution_timing",
            self.output_dir / "execution_timing_feature_importance.json",
        )

        return model

    def train_position_sizing_model(
        self, X: np.ndarray, y: np.ndarray, test_size: float = 0.2
    ) -> xgb.XGBRegressor:
        """Train regression model for position sizing."""
        logger.info("Training position sizing model...")

        X_train, X_test, y_train, y_test = train_test_split(
            X, y, test_size=test_size, random_state=42
        )

        params = {
            "objective": "reg:squarederror",
            "eval_metric": "rmse",
            "max_depth": 5,
            "learning_rate": 0.1,
            "n_estimators": 200,
            "subsample": 0.8,
            "colsample_bytree": 0.8,
            "min_child_weight": 5,
            "gamma": 0.2,
            "reg_alpha": 0.1,
            "reg_lambda": 1.0,
            "random_state": 42,
            "n_jobs": -1,
        }

        model = xgb.XGBRegressor(**params)
        model.fit(
            X_train,
            y_train,
            eval_set=[(X_test, y_test)],
            early_stopping_rounds=20,
            verbose=False,
        )

        # Evaluate
        y_pred = model.predict(X_test)
        rmse = np.sqrt(mean_squared_error(y_test, y_pred))
        r2 = r2_score(y_test, y_pred)

        logger.info(f"Position sizing model - RMSE: {rmse:.4f}, R²: {r2:.4f}")

        self._save_feature_importance(
            model,
            "position_sizing",
            self.output_dir / "position_sizing_feature_importance.json",
        )

        return model

    def _save_feature_importance(
        self, model: xgb.XGBModel, model_name: str, output_file: Path
    ):
        """Save feature importance to JSON file."""
        importance = model.get_booster().get_score(importance_type="gain")

        # Map feature indices to names
        feature_importance = {}
        for idx, name in enumerate(self.feature_names):
            feature_idx = f"f{idx}"
            if feature_idx in importance:
                feature_importance[name] = float(importance[feature_idx])

        # Sort by importance
        sorted_importance = dict(
            sorted(feature_importance.items(), key=lambda x: x[1], reverse=True)
        )

        with open(output_file, "w") as f:
            json.dump(sorted_importance, f, indent=2)

        logger.info(f"Saved feature importance for {model_name} to {output_file}")

    def save_models(self):
        """Save all trained models."""
        models_dir = self.output_dir / "models"
        models_dir.mkdir(exist_ok=True)

        for name, model in self.models.items():
            model_path = models_dir / f"{name}_model.bin"
            model.save_model(str(model_path))
            logger.info(f"Saved {name} model to {model_path}")

        # Save feature names
        with open(models_dir / "feature_names.json", "w") as f:
            json.dump(self.feature_names, f, indent=2)

        logger.info(f"Saved feature names to {models_dir / 'feature_names.json'}")


def main():
    parser = argparse.ArgumentParser(
        description="Train XGBoost models for box spread trading"
    )
    parser.add_argument(
        "--data", type=Path, required=True, help="Path to training data JSON file"
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("python/ml/models"),
        help="Output directory for models",
    )
    parser.add_argument(
        "--models",
        nargs="+",
        choices=["profitability", "risk", "execution_timing", "position_sizing", "all"],
        default=["all"],
        help="Which models to train",
    )

    args = parser.parse_args()

    trainer = ModelTrainer(args.output)

    # Load data
    X, targets = trainer.prepare_data(args.data)

    # Train models
    models_to_train = (
        args.models
        if "all" not in args.models
        else ["profitability", "risk", "execution_timing", "position_sizing"]
    )

    if "profitability" in models_to_train:
        trainer.models["profitability"] = trainer.train_profitability_model(
            X, targets["profitability"]
        )

    if "risk" in models_to_train:
        trainer.models["risk"] = trainer.train_risk_model(X, targets["risk"])

    if "execution_timing" in models_to_train:
        trainer.models["execution_timing"] = trainer.train_execution_timing_model(
            X, targets["execution_time"]
        )

    if "position_sizing" in models_to_train:
        trainer.models["position_sizing"] = trainer.train_position_sizing_model(
            X, targets["position_size"]
        )

    # Save models
    trainer.save_models()

    logger.info("Training complete!")


if __name__ == "__main__":
    main()
