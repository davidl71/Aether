#!/usr/bin/env python3
"""
nautilus_strategy.py - Main entry point for nautilus_trader integration
"""
import argparse
import logging
import sys
from pathlib import Path

# Try to import as package first, fallback to path manipulation
try:
    from python.integration.nautilus_client import NautilusClient
    from python.integration.strategy_runner import StrategyRunner
    from python.integration.connection_manager import (
        ConnectionSupervisor,
        ReauthScheduler,
    )
    from python.integration.preflight import PreflightChecklist
    from python.integration.notification_center import NotificationCenter
    from python.integration.ibkr_portal_client import IBKRPortalClient
    from python.config_adapter import ConfigAdapter
except ImportError:
    # Fallback: add project root to path
    project_root = Path(__file__).parent.parent
    sys.path.insert(0, str(project_root / "python"))
    from integration.nautilus_client import NautilusClient
    from integration.strategy_runner import StrategyRunner
    from integration.connection_manager import (
        ConnectionSupervisor,
        ReauthScheduler,
    )
    from integration.preflight import PreflightChecklist
    from integration.notification_center import NotificationCenter
    from integration.ibkr_portal_client import IBKRPortalClient
    from config_adapter import ConfigAdapter

# Imports handled above

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='[%(asctime)s] [%(levelname)s] [%(name)s] %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger(__name__)


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="IB Box Spread Strategy with NautilusTrader"
    )
    parser.add_argument(
        "--config",
        type=str,
        default="config/config.json",
        help="Path to configuration file"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Run in dry-run mode (no real trades)"
    )
    parser.add_argument(
        "--log-level",
        type=str,
        default="info",
        choices=["debug", "info", "warning", "error"],
        help="Logging level"
    )
    
    args = parser.parse_args()
    
    # Set log level
    log_level = getattr(logging, args.log_level.upper())
    logging.getLogger().setLevel(log_level)
    
    logger.info("=" * 60)
    logger.info("IB Box Spread Strategy - NautilusTrader Integration")
    logger.info("=" * 60)
    
    # Load configuration
    try:
        config = ConfigAdapter.load_config(args.config)
        logger.info(f"Loaded configuration from {args.config}")
    except Exception as e:
        logger.error(f"Failed to load configuration: {e}")
        return 1
    
    # Override dry_run if specified
    if args.dry_run:
        config["dry_run"] = True
        logger.info("Running in DRY-RUN mode")
    
    # Check if nautilus_trader is enabled
    nautilus_config = config.get("nautilus_trader", {})
    if not nautilus_config.get("enabled", True):
        logger.error("NautilusTrader integration is disabled in config")
        return 1
    
    # Initialize NautilusTrader client
    try:
        connection_config = ConfigAdapter.get_connection_management_config(config)
        reauth_config = connection_config.get("weekly_reauth", {})

        notifications_config = ConfigAdapter.get_notifications_config(config)
        notifier = NotificationCenter(notifications_config)

        data_config = ConfigAdapter.get_nautilus_data_config(config)
        exec_config = ConfigAdapter.get_nautilus_exec_config(config)
        data_provider_config = ConfigAdapter.get_data_provider_config(config)
        questdb_config = ConfigAdapter.get_questdb_config(config)
        ibkr_portal_config = ConfigAdapter.get_ibkr_portal_config(config)
        
        preflight = PreflightChecklist(
            config=config,
            nautilus_data_config=data_config,
            nautilus_exec_config=exec_config,
            connection_config=connection_config,
            notifications_config=notifications_config,
            data_provider_config=data_provider_config,
            questdb_config=questdb_config,
            portal_config=ibkr_portal_config,
        )

        preflight_result = preflight.run()
        if not preflight_result.passed:
            logger.error("Pre-flight checks failed. Resolve errors and retry deployment")
            return 1

        portal_client = None
        if ibkr_portal_config.get("enabled", False):
            try:
                portal_client = IBKRPortalClient(
                    base_url=ibkr_portal_config.get("base_url", "https://localhost:5000/v1/portal"),
                    verify_ssl=ibkr_portal_config.get("verify_ssl", False),
                    timeout_seconds=int(ibkr_portal_config.get("timeout_seconds", 5)),
                    preferred_accounts=ibkr_portal_config.get("preferred_accounts", []),
                )
                portal_client.ensure_session()
                logger.info("IBKR Client Portal session validated")
            except Exception as exc:
                logger.warning("Failed to initialise IBKR Client Portal client: %s", exc)
                portal_client = None
        else:
            portal_client = None

        client = NautilusClient(
            data_config=data_config,
            exec_config=exec_config,
            venue="IB",
            notification_center=notifier,
        )
        
        # Create data and exec clients
        client.create_data_client()
        client.create_exec_client()
        
        logger.info("NautilusTrader client initialized")
    except Exception as e:
        logger.error(f"Failed to initialize NautilusTrader client: {e}")
        return 1
    
    # Prepare connection supervisor
    reauth_scheduler = ReauthScheduler(ReauthScheduler.parse_config(reauth_config))
    connection_supervisor = ConnectionSupervisor(reauth_scheduler, notifier)
    if not reauth_scheduler.config.enabled:
        logger.info("Weekly IB re-authentication workflow disabled")

    # Connect to IB
    try:
        if not client.connect():
            logger.error("Failed to connect to Interactive Brokers")
            return 1
        logger.info("Connected to Interactive Brokers")
    except Exception as e:
        logger.error(f"Connection error: {e}")
        return 1
    
    # Initialize strategy runner
    try:
        strategy_config = ConfigAdapter.get_strategy_config(config)
        risk_config = ConfigAdapter.get_risk_config(config)
        orats_config = ConfigAdapter.get_orats_config(config)
        
        # Log ORATS status
        if orats_config:
            logger.info("ORATS integration enabled")
        else:
            logger.info("ORATS integration disabled")
        
        runner = StrategyRunner(
            nautilus_client=client,
            strategy_config=strategy_config,
            risk_config=risk_config,
            orats_config=orats_config,
            notification_center=notifier,
            data_provider_config=data_provider_config,
            questdb_config=questdb_config,
            portal_client=portal_client,
        )
        
        logger.info("Strategy runner initialized")
    except Exception as e:
        logger.error(f"Failed to initialize strategy runner: {e}")
        client.disconnect()
        return 1
    
    # Start strategy
    try:
        runner.start()
        logger.info("Strategy started - monitoring for opportunities")
        
        # Keep running until interrupted
        import signal
        import time

        shutdown_requested = False

        def signal_handler(sig, frame):
            nonlocal shutdown_requested
            logger.info("Received interrupt signal, shutting down...")
            shutdown_requested = True
            runner.stop()
            client.disconnect()
            sys.exit(0)
        
        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)
        
        # Main loop
        while True:
            connection_supervisor.run_housekeeping(client, runner)
            if shutdown_requested:
                break
            if not runner.is_running and not runner.is_paused:
                break
            time.sleep(1)
        
    except KeyboardInterrupt:
        logger.info("Interrupted by user")
    except Exception as e:
        logger.error(f"Error in strategy loop: {e}")
    finally:
        runner.stop()
        client.disconnect()
        logger.info("Shutdown complete")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())



