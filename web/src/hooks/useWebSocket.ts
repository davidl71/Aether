import { useEffect, useState } from 'react';
import { snapshotClient, type ConnectionStatus } from '../api/snapshot';

/**
 * Hook to monitor WebSocket connection status
 * Exposes connection state for UI indicators
 */
export function useWebSocketStatus() {
  const [status, setStatus] = useState<ConnectionStatus>(snapshotClient.getConnectionStatus());

  useEffect(() => {
    const unsubscribe = snapshotClient.onStatusChange((newStatus) => {
      setStatus(newStatus);
    });

    return unsubscribe;
  }, []);

  return { status };
}
