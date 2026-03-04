"""
Continuous learning pipeline for XGBoost models.

This module handles:
- Data collection from live trading
- Periodic model retraining
- Model versioning and A/B testing
- Performance monitoring
"""

import argparse
import json
import logging
import sqlite3
from pathlib import Path
from datetime import datetime, timedelta
from typing import Dict, List, Optional
from dataclasses import dataclass

from .train_models import ModelTrainer

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


@dataclass
class TradingRecord:
  """Record of a trading opportunity and its outcome."""
  timestamp: datetime
  symbol: str
  legs: List[Dict]  # Box spread legs data
  market_data: Dict
  features: List[float]
  predicted_profitable: bool
  predicted_risk: float
  actual_profitable: Optional[bool] = None
  actual_risk: Optional[float] = None
  execution_time: Optional[float] = None
  position_size: Optional[float] = None
  actual_profit: Optional[float] = None
  outcome: Optional[str] = None  # 'executed', 'skipped', 'failed', 'profitable', 'loss'


class DataCollector:
  """Collects trading data for model retraining."""

  def __init__(self, db_path: Path):
    self.db_path = Path(db_path)
    self.db_path.parent.mkdir(parents=True, exist_ok=True)
    self._init_database()

  def _init_database(self):
    """Initialize SQLite database for storing trading records."""
    conn = sqlite3.connect(self.db_path)
    cursor = conn.cursor()

    cursor.execute('''
      CREATE TABLE IF NOT EXISTS trading_records (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp TEXT NOT NULL,
        symbol TEXT NOT NULL,
        legs TEXT NOT NULL,
        market_data TEXT NOT NULL,
        features TEXT NOT NULL,
        predicted_profitable INTEGER NOT NULL,
        predicted_risk REAL NOT NULL,
        actual_profitable INTEGER,
        actual_risk REAL,
        execution_time REAL,
        position_size REAL,
        actual_profit REAL,
        outcome TEXT,
        model_version TEXT
      )
    ''')

    cursor.execute('''
      CREATE INDEX IF NOT EXISTS idx_timestamp ON trading_records(timestamp)
    ''')

    cursor.execute('''
      CREATE INDEX IF NOT EXISTS idx_symbol ON trading_records(symbol)
    ''')

    cursor.execute('''
      CREATE INDEX IF NOT EXISTS idx_outcome ON trading_records(outcome)
    ''')

    conn.commit()
    conn.close()

  def record_opportunity(
    self,
    symbol: str,
    legs: List[Dict],
    market_data: Dict,
    features: List[float],
    predicted_profitable: bool,
    predicted_risk: float,
    model_version: str = "unknown"
  ):
    """Record a trading opportunity (before execution)."""
    conn = sqlite3.connect(self.db_path)
    cursor = conn.cursor()

    cursor.execute('''
      INSERT INTO trading_records
      (timestamp, symbol, legs, market_data, features, predicted_profitable,
       predicted_risk, model_version)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    ''', (
      datetime.utcnow().isoformat(),
      symbol,
      json.dumps(legs),
      json.dumps(market_data),
      json.dumps(features),
      1 if predicted_profitable else 0,
      predicted_risk,
      model_version
    ))

    conn.commit()
    conn.close()

  def update_outcome(
    self,
    record_id: int,
    actual_profitable: Optional[bool] = None,
    actual_risk: Optional[float] = None,
    execution_time: Optional[float] = None,
    position_size: Optional[float] = None,
    actual_profit: Optional[float] = None,
    outcome: Optional[str] = None
  ):
    """Update a trading record with actual outcomes."""
    conn = sqlite3.connect(self.db_path)
    cursor = conn.cursor()

    updates = []
    values = []

    if actual_profitable is not None:
      updates.append("actual_profitable = ?")
      values.append(1 if actual_profitable else 0)

    if actual_risk is not None:
      updates.append("actual_risk = ?")
      values.append(actual_risk)

    if execution_time is not None:
      updates.append("execution_time = ?")
      values.append(execution_time)

    if position_size is not None:
      updates.append("position_size = ?")
      values.append(position_size)

    if actual_profit is not None:
      updates.append("actual_profit = ?")
      values.append(actual_profit)

    if outcome is not None:
      updates.append("outcome = ?")
      values.append(outcome)

    if updates:
      values.append(record_id)
      cursor.execute(
        f"UPDATE trading_records SET {', '.join(updates)} WHERE id = ?",
        values
      )
      conn.commit()

    conn.close()

  def get_training_data(
    self,
    start_date: Optional[datetime] = None,
    end_date: Optional[datetime] = None,
    min_records: int = 100
  ) -> List[Dict]:
    """Retrieve training data from database."""
    conn = sqlite3.connect(self.db_path)
    cursor = conn.cursor()

    query = '''
      SELECT * FROM trading_records
      WHERE actual_profitable IS NOT NULL
    '''
    params = []

    if start_date:
      query += " AND timestamp >= ?"
      params.append(start_date.isoformat())

    if end_date:
      query += " AND timestamp <= ?"
      params.append(end_date.isoformat())

    query += " ORDER BY timestamp"

    cursor.execute(query, params)
    rows = cursor.fetchall()

    if len(rows) < min_records:
      logger.warning(
        f"Only {len(rows)} records found, minimum {min_records} required"
      )
      conn.close()
      return []

    # Convert to training data format
    training_data = []
    for row in rows:
      record = {
        'legs': json.loads(row[3]),
        'market_data': json.loads(row[4]),
        'features': json.loads(row[5]),
        'targets': {
          'profitable': bool(row[8]),
          'high_risk': row[9] > 0.5 if row[9] is not None else False,
          'execution_time': row[10] if row[10] else 0.0,
          'position_size': row[11] if row[11] else 0.0,
        }
      }
      training_data.append(record)

    conn.close()
    return training_data


class ModelVersionManager:
  """Manages model versions and A/B testing."""

  def __init__(self, models_dir: Path):
    self.models_dir = Path(models_dir)
    self.versions_file = self.models_dir / 'versions.json'
    self._load_versions()

  def _load_versions(self):
    """Load version history."""
    if self.versions_file.exists():
      with open(self.versions_file, 'r') as f:
        self.versions = json.load(f)
    else:
      self.versions = {
        'versions': [],
        'current_production': None,
        'current_staging': None,
      }

  def _save_versions(self):
    """Save version history."""
    with open(self.versions_file, 'w') as f:
      json.dump(self.versions, f, indent=2)

  def create_version(
    self,
    model_name: str,
    model_path: Path,
    metrics: Dict,
    description: str = ""
  ) -> str:
    """Create a new model version."""
    version_id = datetime.utcnow().strftime("%Y%m%d_%H%M%S")

    version_info = {
      'version_id': version_id,
      'model_name': model_name,
      'model_path': str(model_path),
      'created_at': datetime.utcnow().isoformat(),
      'metrics': metrics,
      'description': description,
      'status': 'staging',  # New versions start in staging
    }

    self.versions['versions'].append(version_info)
    self.versions['current_staging'] = version_id
    self._save_versions()

    logger.info(f"Created new version {version_id} for {model_name}")
    return version_id

  def promote_to_production(self, version_id: str):
    """Promote a version to production."""
    for version in self.versions['versions']:
      if version['version_id'] == version_id:
        version['status'] = 'production'
        version['promoted_at'] = datetime.utcnow().isoformat()
        self.versions['current_production'] = version_id
        self._save_versions()
        logger.info(f"Promoted version {version_id} to production")
        return

    logger.error(f"Version {version_id} not found")

  def get_production_version(self, model_name: str) -> Optional[str]:
    """Get current production version for a model."""
    for version in self.versions['versions']:
      if (version['model_name'] == model_name and
          version['status'] == 'production'):
        return version['version_id']
    return None


class ContinuousLearningPipeline:
  """Main pipeline for continuous learning."""

  def __init__(
    self,
    data_db_path: Path,
    models_dir: Path,
    retrain_interval_days: int = 7,
    min_new_records: int = 100
  ):
    self.data_collector = DataCollector(data_db_path)
    self.version_manager = ModelVersionManager(models_dir)
    self.models_dir = Path(models_dir)
    self.retrain_interval_days = retrain_interval_days
    self.min_new_records = min_new_records

  def should_retrain(self) -> bool:
    """Check if models should be retrained."""
    # Get last training date
    last_training_file = self.models_dir / 'last_training.json'

    if not last_training_file.exists():
      return True  # Never trained

    with open(last_training_file, 'r') as f:
      last_training = json.load(f)

    last_date = datetime.fromisoformat(last_training['timestamp'])
    days_since = (datetime.utcnow() - last_date).days

    return days_since >= self.retrain_interval_days

  def retrain_models(self, force: bool = False):
    """Retrain all models with latest data."""
    if not force and not self.should_retrain():
      logger.info("Not time to retrain yet")
      return

    logger.info("Starting model retraining...")

    # Get training data
    end_date = datetime.utcnow()
    start_date = end_date - timedelta(days=90)  # Last 90 days

    training_data = self.data_collector.get_training_data(
      start_date, end_date, self.min_new_records
    )

    if not training_data:
      logger.warning("Not enough training data")
      return

    # Save training data to temporary file
    temp_data_file = self.models_dir / 'temp_training_data.json'
    with open(temp_data_file, 'w') as f:
      json.dump(training_data, f, indent=2)

    # Train models
    trainer = ModelTrainer(self.models_dir)
    X, targets = trainer.prepare_data(temp_data_file)

    # Train all models
    trainer.models['profitability'] = trainer.train_profitability_model(
      X, targets['profitability']
    )
    trainer.models['risk'] = trainer.train_risk_model(
      X, targets['risk']
    )
    trainer.models['execution_timing'] = trainer.train_execution_timing_model(
      X, targets['execution_time']
    )
    trainer.models['position_sizing'] = trainer.train_position_sizing_model(
      X, targets['position_size']
    )

    # Save models
    trainer.save_models()

    # Create new versions
    for model_name in ['profitability', 'risk', 'execution_timing', 'position_sizing']:
      model_path = self.models_dir / 'models' / f'{model_name}_model.bin'

      # Evaluate model (simplified - would use actual test set)
      metrics = {
        'training_samples': len(training_data),
        'retrained_at': datetime.utcnow().isoformat(),
      }

      self.version_manager.create_version(
        model_name, model_path, metrics, f"Retrained on {len(training_data)} samples"
      )

    # Update last training timestamp
    last_training_file = self.models_dir / 'last_training.json'
    with open(last_training_file, 'w') as f:
      json.dump({
        'timestamp': datetime.utcnow().isoformat(),
        'samples_used': len(training_data),
      }, f, indent=2)

    # Clean up
    temp_data_file.unlink()

    logger.info("Model retraining complete")

  def monitor_performance(self):
    """Monitor model performance in production."""
    # Get recent predictions and outcomes
    conn = sqlite3.connect(self.data_collector.db_path)
    cursor = conn.cursor()

    # Get predictions from last 24 hours
    yesterday = (datetime.utcnow() - timedelta(days=1)).isoformat()

    cursor.execute('''
      SELECT
        predicted_profitable,
        actual_profitable,
        predicted_risk,
        actual_risk,
        outcome
      FROM trading_records
      WHERE timestamp >= ? AND actual_profitable IS NOT NULL
    ''', (yesterday,))

    rows = cursor.fetchall()
    conn.close()

    if not rows:
      logger.info("No recent data for performance monitoring")
      return

    # Calculate metrics
    correct_profitable = sum(
      1 for row in rows
      if bool(row[0]) == bool(row[1])
    )
    accuracy = correct_profitable / len(rows) if rows else 0.0

    logger.info("Model Performance (last 24h):")
    logger.info(f"  Accuracy: {accuracy:.2%}")
    logger.info(f"  Total predictions: {len(rows)}")

    return {
      'accuracy': accuracy,
      'total_predictions': len(rows),
      'timestamp': datetime.utcnow().isoformat(),
    }


def main():
  parser = argparse.ArgumentParser(
    description='Continuous learning pipeline for XGBoost models'
  )
  parser.add_argument(
    '--data-db', type=Path, default=Path('data/trading_records.db'),
    help='Path to trading records database'
  )
  parser.add_argument(
    '--models-dir', type=Path, default=Path('python/ml/models'),
    help='Directory for models'
  )
  parser.add_argument(
    '--retrain', action='store_true',
    help='Force retraining of models'
  )
  parser.add_argument(
    '--monitor', action='store_true',
    help='Monitor model performance'
  )

  args = parser.parse_args()

  pipeline = ContinuousLearningPipeline(
    args.data_db,
    args.models_dir,
    retrain_interval_days=7,
    min_new_records=100
  )

  if args.retrain:
    pipeline.retrain_models(force=True)

  if args.monitor:
    pipeline.monitor_performance()

  # Auto-retrain if needed
  if pipeline.should_retrain():
    pipeline.retrain_models()


if __name__ == '__main__':
  main()
