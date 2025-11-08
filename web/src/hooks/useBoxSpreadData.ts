import { useEffect, useState } from 'react';
import type { BoxSpreadPayload } from '../types';

interface HookState {
  data: BoxSpreadPayload | null;
  isLoading: boolean;
  error: string | null;
}

const DATA_URL = '/data/box_spread_sample.json';

export function useBoxSpreadData(): HookState {
  const [state, setState] = useState<HookState>({
    data: null,
    isLoading: true,
    error: null
  });

  useEffect(() => {
    let isCancelled = false;

    async function fetchData() {
      try {
        const response = await fetch(DATA_URL);
        if (!response.ok) {
          throw new Error(`Request failed with status ${response.status}`);
        }

        const payload = (await response.json()) as BoxSpreadPayload;

        if (!isCancelled) {
          setState({ data: payload, isLoading: false, error: null });
        }
      } catch (err) {
        if (!isCancelled) {
          setState({ data: null, isLoading: false, error: err instanceof Error ? err.message : 'Unknown error' });
        }
      }
    }

    void fetchData();

    return () => {
      isCancelled = true;
    };
  }, []);

  return state;
}
