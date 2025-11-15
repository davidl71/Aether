"""
Feature engineering for box spread ML models.

Extracts and transforms raw market data into features suitable for XGBoost training.
"""

from dataclasses import dataclass
from datetime import datetime
from typing import Dict, List, Optional, Tuple

import numpy as np


@dataclass
class BoxSpreadLeg:
    """Represents a single leg of a box spread."""

    strike: float
    expiration: datetime
    option_type: str  # 'call' or 'put'
    bid: float
    ask: float
    volume: int
    open_interest: int
    implied_volatility: Optional[float] = None
    delta: Optional[float] = None
    gamma: Optional[float] = None
    theta: Optional[float] = None
    vega: Optional[float] = None


@dataclass
class MarketData:
    """Market data context for feature engineering."""

    underlying_price: float
    risk_free_rate: float
    current_time: datetime
    vix: Optional[float] = None
    market_volatility: Optional[float] = None


class FeatureExtractor:
    """Extracts features from box spread legs and market data."""

    def __init__(self):
        self.feature_names = []

    def extract_features(
        self,
        legs: List[BoxSpreadLeg],
        market_data: MarketData,
        historical_data: Optional[Dict] = None,
    ) -> np.ndarray:
        """
        Extract features for ML model.

        Args:
          legs: List of 4 box spread legs
          market_data: Current market context
          historical_data: Optional historical data for temporal features

        Returns:
          Feature vector as numpy array
        """
        features = []
        feature_names = []

        # Validate we have 4 legs
        if len(legs) != 4:
            raise ValueError(f"Box spread requires 4 legs, got {len(legs)}")

        # Sort legs by strike for consistent ordering
        sorted_legs = sorted(legs, key=lambda x: (x.strike, x.option_type))

        # === Spread Features ===
        spread_features, spread_names = self._extract_spread_features(sorted_legs)
        features.extend(spread_features)
        feature_names.extend(spread_names)

        # === Leg Features ===
        leg_features, leg_names = self._extract_leg_features(sorted_legs)
        features.extend(leg_features)
        feature_names.extend(leg_names)

        # === Market Features ===
        market_features, market_names = self._extract_market_features(
            sorted_legs, market_data
        )
        features.extend(market_features)
        feature_names.extend(market_names)

        # === Greeks Features ===
        greeks_features, greeks_names = self._extract_greeks_features(sorted_legs)
        features.extend(greeks_features)
        feature_names.extend(greeks_names)

        # === Temporal Features ===
        temporal_features, temporal_names = self._extract_temporal_features(
            sorted_legs, market_data
        )
        features.extend(temporal_features)
        feature_names.extend(temporal_names)

        # === Historical Features ===
        if historical_data:
            hist_features, hist_names = self._extract_historical_features(
                sorted_legs, historical_data
            )
            features.extend(hist_features)
            feature_names.extend(hist_names)

        self.feature_names = feature_names
        return np.array(features, dtype=np.float32)

    def _extract_spread_features(
        self, legs: List[BoxSpreadLeg]
    ) -> Tuple[List[float], List[str]]:
        """Extract spread-level features."""
        features = []
        names = []

        # Calculate box spread value
        # Box spread = (K2 - K1) * multiplier
        strikes = [leg.strike for leg in legs]
        min_strike = min(strikes)
        max_strike = max(strikes)
        box_value = max_strike - min_strike

        # Net premium paid/received
        # Long legs (buy): negative premium
        # Short legs (sell): positive premium
        net_premium = 0.0
        for leg in legs:
            mid_price = (leg.bid + leg.ask) / 2.0
            # Determine if long or short based on position
            # For box spread: typically long lower strike, short higher strike
            # This is simplified - actual logic depends on strategy
            if leg.strike == min_strike:
                net_premium -= mid_price  # Long
            else:
                net_premium += mid_price  # Short

        # Spread width (bid-ask)
        total_spread = sum(leg.ask - leg.bid for leg in legs)

        # Average spread percentage
        avg_spread_pct = np.mean(
            [
                (
                    (leg.ask - leg.bid) / ((leg.bid + leg.ask) / 2.0)
                    if leg.bid > 0
                    else 0.0
                )
                for leg in legs
            ]
        )

        # Maximum spread percentage
        max_spread_pct = max(
            [
                (
                    (leg.ask - leg.bid) / ((leg.bid + leg.ask) / 2.0)
                    if leg.bid > 0
                    else 0.0
                )
                for leg in legs
            ],
            default=0.0,
        )

        features.extend(
            [
                box_value,
                net_premium,
                total_spread,
                avg_spread_pct,
                max_spread_pct,
            ]
        )
        names.extend(
            [
                "box_value",
                "net_premium",
                "total_spread",
                "avg_spread_pct",
                "max_spread_pct",
            ]
        )

        return features, names

    def _extract_leg_features(
        self, legs: List[BoxSpreadLeg]
    ) -> Tuple[List[float], List[str]]:
        """Extract features for each leg."""
        features = []
        names = []

        # Volume and open interest statistics
        volumes = [leg.volume for leg in legs]
        ois = [leg.open_interest for leg in legs]

        features.extend(
            [
                np.sum(volumes),  # Total volume
                np.mean(volumes),  # Average volume
                np.std(volumes) if len(volumes) > 1 else 0.0,  # Volume std
                np.min(volumes),  # Min volume
                np.max(volumes),  # Max volume
                np.sum(ois),  # Total open interest
                np.mean(ois),  # Average open interest
                np.std(ois) if len(ois) > 1 else 0.0,  # OI std
                np.min(ois),  # Min OI
                np.max(ois),  # Max OI
            ]
        )
        names.extend(
            [
                "total_volume",
                "avg_volume",
                "volume_std",
                "min_volume",
                "max_volume",
                "total_oi",
                "avg_oi",
                "oi_std",
                "min_oi",
                "max_oi",
            ]
        )

        # Implied volatility statistics
        ivs = [
            leg.implied_volatility for leg in legs if leg.implied_volatility is not None
        ]
        if ivs:
            features.extend(
                [
                    np.mean(ivs),
                    np.std(ivs) if len(ivs) > 1 else 0.0,
                    np.min(ivs),
                    np.max(ivs),
                ]
            )
            names.extend(
                [
                    "avg_iv",
                    "iv_std",
                    "min_iv",
                    "max_iv",
                ]
            )
        else:
            features.extend([0.0, 0.0, 0.0, 0.0])
            names.extend(["avg_iv", "iv_std", "min_iv", "max_iv"])

        # Bid-ask spread statistics per leg
        spreads = [leg.ask - leg.bid for leg in legs if leg.bid > 0]
        if spreads:
            features.extend(
                [
                    np.mean(spreads),
                    np.std(spreads) if len(spreads) > 1 else 0.0,
                    np.min(spreads),
                    np.max(spreads),
                ]
            )
            names.extend(
                [
                    "avg_leg_spread",
                    "leg_spread_std",
                    "min_leg_spread",
                    "max_leg_spread",
                ]
            )
        else:
            features.extend([0.0, 0.0, 0.0, 0.0])
            names.extend(
                ["avg_leg_spread", "leg_spread_std", "min_leg_spread", "max_leg_spread"]
            )

        return features, names

    def _extract_market_features(
        self, legs: List[BoxSpreadLeg], market_data: MarketData
    ) -> Tuple[List[float], List[str]]:
        """Extract market-level features."""
        features = []
        names = []

        # Underlying price features
        underlying = market_data.underlying_price
        strikes = [leg.strike for leg in legs]

        # Moneyness (distance from strike)
        avg_strike = np.mean(strikes)
        moneyness = (underlying - avg_strike) / underlying if underlying > 0 else 0.0

        features.extend(
            [
                underlying,
                market_data.risk_free_rate,
                moneyness,
                market_data.vix if market_data.vix else 0.0,
                market_data.market_volatility if market_data.market_volatility else 0.0,
            ]
        )
        names.extend(
            [
                "underlying_price",
                "risk_free_rate",
                "moneyness",
                "vix",
                "market_volatility",
            ]
        )

        return features, names

    def _extract_greeks_features(
        self, legs: List[BoxSpreadLeg]
    ) -> Tuple[List[float], List[str]]:
        """Extract Greeks-based features."""
        features = []
        names = []

        # Aggregate Greeks
        deltas = [leg.delta for leg in legs if leg.delta is not None]
        gammas = [leg.gamma for leg in legs if leg.gamma is not None]
        thetas = [leg.theta for leg in legs if leg.theta is not None]
        vegas = [leg.vega for leg in legs if leg.vega is not None]

        # Net Greeks (sum across legs)
        net_delta = sum(deltas) if deltas else 0.0
        net_gamma = sum(gammas) if gammas else 0.0
        net_theta = sum(thetas) if thetas else 0.0
        net_vega = sum(vegas) if vegas else 0.0

        # Greeks statistics
        if deltas:
            features.extend(
                [np.mean(deltas), np.std(deltas) if len(deltas) > 1 else 0.0]
            )
            names.extend(["avg_delta", "delta_std"])
        else:
            features.extend([0.0, 0.0])
            names.extend(["avg_delta", "delta_std"])

        if gammas:
            features.extend(
                [np.mean(gammas), np.std(gammas) if len(gammas) > 1 else 0.0]
            )
            names.extend(["avg_gamma", "gamma_std"])
        else:
            features.extend([0.0, 0.0])
            names.extend(["avg_gamma", "gamma_std"])

        features.extend([net_delta, net_gamma, net_theta, net_vega])
        names.extend(["net_delta", "net_gamma", "net_theta", "net_vega"])

        return features, names

    def _extract_temporal_features(
        self, legs: List[BoxSpreadLeg], market_data: MarketData
    ) -> Tuple[List[float], List[str]]:
        """Extract time-based features."""
        features = []
        names = []

        # Time to expiration (in days)
        current_time = market_data.current_time
        expirations = [leg.expiration for leg in legs]
        min_exp = min(expirations)
        max_exp = max(expirations)

        days_to_exp_min = (min_exp - current_time).total_seconds() / 86400.0
        days_to_exp_max = (max_exp - current_time).total_seconds() / 86400.0
        days_to_exp_avg = np.mean(
            [(exp - current_time).total_seconds() / 86400.0 for exp in expirations]
        )

        # Time of day features
        hour = current_time.hour
        minute = current_time.minute
        is_market_hours = 9 <= hour < 16  # Simplified market hours

        features.extend(
            [
                days_to_exp_min,
                days_to_exp_max,
                days_to_exp_avg,
                float(hour),
                float(minute),
                float(is_market_hours),
            ]
        )
        names.extend(
            [
                "days_to_exp_min",
                "days_to_exp_max",
                "days_to_exp_avg",
                "hour",
                "minute",
                "is_market_hours",
            ]
        )

        return features, names

    def _extract_historical_features(
        self, legs: List[BoxSpreadLeg], historical_data: Dict
    ) -> Tuple[List[float], List[str]]:
        """Extract features from historical data."""
        features = []
        names = []

        # Historical profitability (if available)
        if "historical_profitability" in historical_data:
            features.append(historical_data["historical_profitability"])
            names.append("historical_profitability")

        # Historical success rate
        if "success_rate" in historical_data:
            features.append(historical_data["success_rate"])
            names.append("historical_success_rate")

        # Recent spread trends
        if "recent_spreads" in historical_data:
            recent = historical_data["recent_spreads"]
            if recent:
                features.extend(
                    [
                        np.mean(recent),
                        np.std(recent) if len(recent) > 1 else 0.0,
                    ]
                )
                names.extend(["recent_spread_mean", "recent_spread_std"])
            else:
                features.extend([0.0, 0.0])
                names.extend(["recent_spread_mean", "recent_spread_std"])

        return features, names

    def get_feature_names(self) -> List[str]:
        """Get list of feature names."""
        return self.feature_names.copy()
