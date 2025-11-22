import { useEffect, useState } from 'react';
import { snapshotClient } from '../api/snapshot';
import type { SnapshotPayload } from '../types/snapshot';

interface SnapshotState {
  snapshot: SnapshotPayload | null;
  isLoading: boolean;
  error: string | null;
}

export function useSnapshot() {
  const [state, setState] = useState<SnapshotState>({
    snapshot: null,
    isLoading: true,
    error: null
  });

  useEffect(() => {
    console.log('useSnapshot: Subscribing to snapshot client...');
    const unsubscribe = snapshotClient.subscribe(
      (payload) => {
        console.log('useSnapshot: Received snapshot payload', payload);
        setState({ snapshot: payload, isLoading: false, error: null });
      },
      (error) => {
        console.error('useSnapshot: Error from snapshot client', error);
        setState((prev) => ({
          snapshot: prev.snapshot,
          isLoading: false,
          error: error.message
        }));
      }
    );

    // Timeout fallback - if no data after 10 seconds, show error
    const timeout = setTimeout(() => {
      setState((prev) => {
        if (prev.isLoading && !prev.snapshot) {
          console.warn('useSnapshot: Timeout - no snapshot received after 10 seconds');
          return {
            ...prev,
            isLoading: false,
            error: 'Timeout: No snapshot data received. Check if backend is running or if /data/snapshot.json exists.'
          };
        }
        return prev;
      });
    }, 10000);

    return () => {
      clearTimeout(timeout);
      unsubscribe();
    };
  }, []);

  return state;
}
