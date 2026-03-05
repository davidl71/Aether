"""
preflight.py - Deployment pre-flight checklist inspired by QuantConnect Lean CLI.
"""
from __future__ import annotations

import logging
import socket
from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple

logger = logging.getLogger(__name__)


@dataclass
class PreflightResult:
    passed: bool
    warnings: List[str]
    errors: List[str]


class PreflightChecklist:
    """Runs deployment-time validations before strategy startup."""

    def __init__(
        self,
        config: Dict,
        nautilus_data_config: Dict,
        nautilus_exec_config: Dict,
        connection_config: Dict,
        notifications_config: Optional[Dict] = None,
        data_provider_config: Optional[Dict] = None,
        questdb_config: Optional[Dict] = None,
        portal_config: Optional[Dict] = None,
    ) -> None:
        self.config = config
        self.data_cfg = nautilus_data_config
        self.exec_cfg = nautilus_exec_config
        self.connection_cfg = connection_config
        self.notifications_cfg = notifications_config or {}
        self.data_provider_cfg = data_provider_config or {}
        self.questdb_cfg = questdb_config or {}
        self.portal_cfg = portal_config or {}

    def run(self) -> PreflightResult:
        warnings: List[str] = []
        errors: List[str] = []

        self._log_heading("Pre-flight Checklist")

        # 1. Validate configuration
        is_valid, validation_errors = self._validate_config()
        if not is_valid:
            errors.extend(validation_errors)

        # 2. Verify IB host reachability
        if not self._check_host_reachable(self.exec_cfg.get("host"), self.exec_cfg.get("port")):
            errors.append(
                "Interactive Brokers host/port is unreachable. Ensure TWS or IB Gateway is running."
            )

        # 3. Summarise trading mode
        dry_run = self.config.get("dry_run", True)
        logger.info("Trading mode: %s", "DRY-RUN" if dry_run else "LIVE")
        if not dry_run:
            warnings.append(
                "Live trading enabled. Confirm capital, risk limits, and market data subscriptions."
            )

        # 4. ORATS readiness
        orats_cfg = self.config.get("orats", {})
        if orats_cfg.get("enabled", False) and not orats_cfg.get("api_token"):
            errors.append("ORATS integration enabled but api_token missing")

        # 5. Connection management summary
        weekly_cfg = self.connection_cfg.get("weekly_reauth", {})
        if weekly_cfg.get("enabled", False):
            logger.info(
                "Weekly IB re-authentication window configured for %s at %s UTC",
                weekly_cfg.get("day_of_week", "sunday").title(),
                weekly_cfg.get("time_utc", "21:00"),
            )
        else:
            warnings.append("Weekly IB re-authentication workflow disabled")

        # 6. Data client/execution client parity check
        if (self.data_cfg.get("host"), self.data_cfg.get("port")) != (
            self.exec_cfg.get("host"),
            self.exec_cfg.get("port"),
        ):
            warnings.append(
                "Data and execution clients target different IB endpoints. Verify this is intentional."
            )

        # 7. Notification readiness
        if self.notifications_cfg.get("enabled", False):
            channels = self.notifications_cfg.get("channels", [])
            logger.info("Notifications enabled (%d channels configured)", len(channels))
            if not channels:
                errors.append("Notifications enabled but no channels configured")
        else:
            warnings.append("Notifications disabled; critical events will only be logged")

        # 8. Data provider order summary
        primary = self.data_provider_cfg.get("primary", "ib")
        fallbacks = self.data_provider_cfg.get("fallbacks", [])
        provider_chain = [primary] + fallbacks if fallbacks else [primary]
        logger.info("Market data provider chain: %s", " -> ".join(provider_chain))
        if primary.lower() != "ib":
            warnings.append(
                "Primary data provider is not IB; ensure NautilusTrader subscriptions match selection."
            )

        # 9. QuestDB ingestion summary
        if self.questdb_cfg.get("enabled", False):
            logger.info(
                "QuestDB ingestion enabled (%s:%s)",
                self.questdb_cfg.get("ilp_host", "127.0.0.1"),
                self.questdb_cfg.get("ilp_port", 9009),
            )
        else:
            warnings.append("QuestDB ingestion disabled; market data will not be archived")

        # 10. Client Portal availability
        if self.portal_cfg.get("enabled", False):
            logger.info(
                "IBKR Client Portal API enabled (%s)",
                self.portal_cfg.get("base_url", "https://localhost:5001/v1/portal"),
            )
        else:
            warnings.append("IBKR Client Portal integration disabled; account snapshots require TWS")

        passed = not errors

        if passed:
            self._log_heading("Pre-flight checks passed")
        else:
            self._log_heading("Pre-flight checks failed")

        for item in warnings:
            logger.warning("PRE-FLIGHT WARNING: %s", item)

        for item in errors:
            logger.error("PRE-FLIGHT ERROR: %s", item)

        return PreflightResult(passed=passed, warnings=warnings, errors=errors)

    def _validate_config(self) -> Tuple[bool, List[str]]:
        try:
            from python.config_adapter import ConfigAdapter  # type: ignore
        except ImportError:  # pragma: no cover - fallback when running as script
            from config_adapter import ConfigAdapter  # type: ignore

        try:
            is_valid, errors = ConfigAdapter.validate_config(self.config)
            if is_valid:
                logger.info("Configuration validation successful")
            return is_valid, errors
        except Exception as exc:  # pragma: no cover - defensive
            logger.exception("Configuration validation raised unexpected error: %s", exc)
            return False, [f"Unexpected error during validation: {exc}"]

    @staticmethod
    def _check_host_reachable(host: Optional[str], port: Optional[int]) -> bool:
        if not host or not port:
            return False

        try:
            with socket.create_connection((host, port), timeout=2.0):
                logger.info("IB host %s:%s reachable", host, port)
                return True
        except OSError as exc:
            logger.warning("Unable to reach IB host %s:%s (%s)", host, port, exc)
            return False

    @staticmethod
    def _log_heading(title: str) -> None:
        logger.info("=" * 60)
        logger.info(title)
        logger.info("=" * 60)


