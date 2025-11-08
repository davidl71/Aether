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
    const unsubscribe = snapshotClient.subscribe(
      (payload) => {
        setState({ snapshot: payload, isLoading: false, error: null });
      },
      (error) => {
        setState((prev) => ({
          snapshot: prev.snapshot,
          isLoading: false,
          error: error.message
        }));
      }
    );

    return () => {
      unsubscribe();
    };
  }, []);

  return state;
}
