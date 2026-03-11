# Alpaca Integration for PWA

This runtime path is retired.

Alpaca is no longer an active web or Python service surface in this repo. The historical runtime retirement note is:

- [ALPACA_TASTYTRADE_RUNTIME_RETIREMENT.md](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/archive/ALPACA_TASTYTRADE_RUNTIME_RETIREMENT.md)

If Alpaca returns later, it should come back behind the Rust shared origin rather than as a standalone Python FastAPI service.

### Health Check
```bash
curl http://127.0.0.1:8000/api/health
```

Response:
```json
{
  "status": "ok",
  "ts": "2025-01-27T12:00:00+00:00"
}
```

### Snapshot
```bash
curl http://127.0.0.1:8000/api/snapshot
```

Response matches the PWA's `SnapshotPayload` type with:
- Real-time quotes (bid/ask/last)
- Symbol snapshots
- Account metrics
- Compatible with existing PWA components

## Next Steps

- Add options chain data via Alpaca's options API
- Implement order placement (requires Alpaca trading API)
- Add WebSocket support for real-time updates
- Integrate with box spread calculation engine
