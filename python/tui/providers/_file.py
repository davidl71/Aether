"""FileProvider: reads snapshots from a JSON file on disk."""
from __future__ import annotations

import json
import logging
import threading
import time
from pathlib import Path
from typing import Optional

from ..models import SnapshotPayload
from ._base import Provider

logger = logging.getLogger(__name__)


class FileProvider(Provider):
    """
    File provider that reads snapshots from a JSON file on disk.

    MIGRATION NOTE: File I/O can be done in C++ using std::filesystem and nlohmann/json.
    """

    def __init__(self, file_path: str, update_interval_ms: int = 1000):
        super().__init__()
        self.file_path = Path(file_path)
        self.update_interval_ms = update_interval_ms
        self._worker_thread: Optional[threading.Thread] = None
        self._last_mtime: float = 0.0

    def start(self) -> None:
        if self._running:
            return
        self._running = True
        self._worker_thread = threading.Thread(target=self._poll_loop, daemon=True)
        self._worker_thread.start()
        logger.info(f"FileProvider started: {self.file_path}")

    def stop(self) -> None:
        self._running = False
        if self._worker_thread:
            self._worker_thread.join(timeout=2.0)
        logger.info("FileProvider stopped")

    def get_snapshot(self) -> SnapshotPayload:
        with self._lock:
            if self._latest_snapshot is None:
                return self._load_from_file()
            return self._latest_snapshot

    def is_running(self) -> bool:
        return self._running

    def _poll_loop(self) -> None:
        while self._running:
            try:
                if self.file_path.exists():
                    current_mtime = self.file_path.stat().st_mtime
                    if current_mtime > self._last_mtime:
                        snapshot = self._load_from_file()
                        with self._lock:
                            self._latest_snapshot = snapshot
                        self._last_mtime = current_mtime
            except Exception as e:
                logger.error(f"File poll error: {e}")
            time.sleep(self.update_interval_ms / 1000.0)

    def _load_from_file(self) -> SnapshotPayload:
        try:
            if not self.file_path.exists():
                logger.warning(f"Snapshot file not found: {self.file_path}")
                return SnapshotPayload()

            with open(self.file_path, 'r') as f:
                data = json.load(f)
            return SnapshotPayload.from_dict(data)
        except Exception as e:
            logger.error(f"File load error: {e}")
            return SnapshotPayload()
