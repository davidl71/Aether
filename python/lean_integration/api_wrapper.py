"""
api_wrapper.py - FastAPI REST API Wrapper for LEAN

This module provides a FastAPI application that exposes LEAN's internal state
via REST endpoints matching the API contract defined in agents/shared/API_CONTRACT.md.
"""

import os
import logging
from typing import Optional
from datetime import datetime, timezone

from fastapi import FastAPI, Depends, HTTPException, status, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse

from .lean_client import LeanClient
from .api_converter import ApiConverter
from .websocket_manager import websocket_manager
from .event_bridge import get_event_bridge, event_bridge
from .algorithm_hooks import hook_algorithm_events
from .api_models import (
    SnapshotResponse,
    StrategyStartRequest,
    StrategyStopRequest,
    CancelOrderRequest,
    ComboOrderRequest,
    HealthResponse,
    ErrorResponse
)

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Create FastAPI application
app = FastAPI(
    title="LEAN REST API Wrapper",
    description="REST API wrapper for QuantConnect LEAN algorithmic trading engine",
    version="1.0.0",
    docs_url="/docs",
    redoc_url="/redoc"
)

# CORS configuration
cors_origins = os.getenv(
    "CORS_ORIGINS",
    "http://localhost:3000,http://localhost:5173,http://localhost:8080"
).split(",")

app.add_middleware(
    CORSMiddleware,
    allow_origins=cors_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Global instances
lean_client = LeanClient()
api_converter = ApiConverter()


def get_lean_client() -> LeanClient:
    """
    Dependency to get LEAN client.

    Raises:
        HTTPException: If LEAN algorithm is not running
    """
    if not lean_client.is_running:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="LEAN algorithm is not running"
        )
    return lean_client


@app.get("/health", response_model=HealthResponse)
async def health():
    """Health check endpoint."""
    return HealthResponse(
        status="ok",
        lean_running=lean_client.is_running,
        timestamp=datetime.now(timezone.utc)
    )


@app.get("/api/v1/snapshot", response_model=SnapshotResponse)
async def get_snapshot(client: LeanClient = Depends(get_lean_client)):
    """
    Get system snapshot matching API contract.

    Returns complete system state including portfolio, positions, orders, and metrics.
    """
    try:
        # Query LEAN internal state
        portfolio = client.get_portfolio()
        positions = client.get_positions()
        orders = client.get_orders()
        metrics = client.get_metrics()
        symbols = client.get_symbols()

        # Convert to API contract format
        snapshot = api_converter.build_snapshot(
            portfolio=portfolio,
            positions=positions,
            orders=orders,
            metrics=metrics,
            symbols=symbols,
            algorithm=client.algorithm
        )

        return snapshot

    except RuntimeError as e:
        logger.error(f"Runtime error getting snapshot: {e}")
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail=str(e)
        )
    except Exception as e:
        logger.error(f"Unexpected error getting snapshot: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to get snapshot: {str(e)}"
        )


@app.post("/api/v1/strategy/start", status_code=status.HTTP_204_NO_CONTENT)
async def strategy_start(
    request: StrategyStartRequest,
    client: LeanClient = Depends(get_lean_client)
):
    """
    Start LEAN strategy.

    Requires confirmation flag in request body.
    """
    if not request.confirm:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Confirmation required to start strategy"
        )

    try:
        client.start_algorithm()
        logger.info("Strategy started via REST API")
    except RuntimeError as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=str(e)
        )
    except NotImplementedError:
        raise HTTPException(
            status_code=status.HTTP_501_NOT_IMPLEMENTED,
            detail="Strategy start must be implemented via LEAN launcher"
        )


@app.post("/api/v1/strategy/stop", status_code=status.HTTP_204_NO_CONTENT)
async def strategy_stop(
    request: StrategyStopRequest,
    client: LeanClient = Depends(get_lean_client)
):
    """
    Stop LEAN strategy.

    Requires confirmation flag in request body.
    """
    if not request.confirm:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Confirmation required to stop strategy"
        )

    try:
        client.stop_algorithm()
        logger.info("Strategy stopped via REST API")
    except RuntimeError as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=str(e)
        )
    except NotImplementedError:
        raise HTTPException(
            status_code=status.HTTP_501_NOT_IMPLEMENTED,
            detail="Strategy stop must be implemented via LEAN launcher"
        )


@app.post("/api/v1/orders/cancel", status_code=status.HTTP_204_NO_CONTENT)
async def cancel_order(
    request: CancelOrderRequest,
    client: LeanClient = Depends(get_lean_client)
):
    """
    Cancel a specific order.

    Requires confirmation flag in request body.
    """
    if not request.confirm:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Confirmation required to cancel order"
        )

    try:
        success = client.cancel_order(request.order_id)
        if not success:
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND,
                detail=f"Order not found: {request.order_id}"
            )
        logger.info(f"Order {request.order_id} cancelled via REST API")
    except RuntimeError as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=str(e)
        )


@app.post("/api/v1/combos/buy")
async def buy_combo(
    request: ComboOrderRequest,
    client: LeanClient = Depends(get_lean_client)
):
    """
    Place buy combo order.

    Requires confirmation flag in request body.
    """
    if not request.confirm:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Confirmation required to place combo order"
        )

    # Implementation: Place combo order via LEAN
    # This would use client.algorithm.ComboMarketOrder() or ComboLimitOrder()
    raise HTTPException(
        status_code=status.HTTP_501_NOT_IMPLEMENTED,
        detail="Combo order placement not yet implemented"
    )


@app.post("/api/v1/combos/sell")
async def sell_combo(
    request: ComboOrderRequest,
    client: LeanClient = Depends(get_lean_client)
):
    """
    Place sell combo order.

    Requires confirmation flag in request body.
    """
    if not request.confirm:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Confirmation required to place combo order"
        )

    # Implementation: Place combo order via LEAN
    # This would use client.algorithm.ComboMarketOrder() or ComboLimitOrder()
    raise HTTPException(
        status_code=status.HTTP_501_NOT_IMPLEMENTED,
        detail="Combo order placement not yet implemented"
    )


@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    """
    WebSocket endpoint for real-time LEAN events.

    Clients connect to receive real-time updates including:
    - Order events (filled, cancelled)
    - Position updates
    - Symbol market data updates
    - Alerts
    - Periodic snapshot updates
    """
    await websocket_manager.connect(websocket)
    try:
        # Send initial connection confirmation
        await websocket.send_json({
            "type": "connected",
            "data": {
                "message": "WebSocket connected",
                "timestamp": datetime.now(timezone.utc).isoformat()
            }
        })

        # Keep connection alive and handle client messages
        while True:
            try:
                # Wait for client messages (ping/pong, subscriptions, etc.)
                data = await websocket.receive_text()

                # Handle client messages (future: subscription management)
                # For now, just echo back or ignore
                logger.debug(f"Received WebSocket message: {data}")

            except WebSocketDisconnect:
                break

    except WebSocketDisconnect:
        pass
    except Exception as e:
        logger.error(f"WebSocket error: {e}", exc_info=True)
    finally:
        await websocket_manager.disconnect(websocket)


@app.exception_handler(Exception)
async def global_exception_handler(request, exc):
    """Global exception handler."""
    logger.error(f"Unhandled exception: {exc}", exc_info=True)
    return JSONResponse(
        status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        content=ErrorResponse(
            error="Internal server error",
            detail=str(exc),
            timestamp=datetime.now(timezone.utc)
        ).model_dump()
    )


# Function to set algorithm instance (called by LEAN launcher)
def set_algorithm(algorithm):
    """
    Set the LEAN algorithm instance and start event bridge.

    This function should be called by the LEAN launcher after the algorithm
    is initialized and running.

    Args:
        algorithm: Running BoxSpreadAlgorithm instance
    """
    global event_bridge

    lean_client.set_algorithm(algorithm)
    logger.info("LEAN algorithm instance set in API wrapper")

    # Initialize and start event bridge
    if event_bridge is None:
        event_bridge = get_event_bridge(lean_client)
        event_bridge.start()
        logger.info("Event bridge started")

    # Hook event bridge into algorithm callbacks
    try:
        hook_algorithm_events(algorithm)
        logger.info("Algorithm event hooks installed")
    except Exception as e:
        logger.warning(f"Failed to hook algorithm events: {e}")


if __name__ == "__main__":
    import uvicorn

    host = os.getenv("LEAN_API_HOST", "0.0.0.0")
    port = int(os.getenv("LEAN_API_PORT", "8000"))

    logger.info(f"Starting LEAN REST API wrapper on {host}:{port}")
    uvicorn.run(app, host=host, port=port)
