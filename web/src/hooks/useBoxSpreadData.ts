import { useEffect, useState } from 'react';
import type { BoxSpreadPayload, BoxSpreadScenario } from '../types';

interface HookState {
  data: BoxSpreadPayload | null;
  isLoading: boolean;
  error: string | null;
}

const API_URL = 'http://127.0.0.1:8080/api/v1/scenarios';
const STATIC_FALLBACK_URL = '/data/box_spread_sample.json';

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
        const response = await fetch(API_URL);
        if (!response.ok) {
          throw new Error(`API returned ${response.status}`);
        }
        const json = await response.json();
        const payload = mapApiResponse(json);
        if (!isCancelled) {
          setState({ data: payload, isLoading: false, error: null });
        }
        return;
      } catch {
        // API unavailable -- fall back to static JSON
      }

      try {
        const response = await fetch(STATIC_FALLBACK_URL);
        if (!response.ok) {
          throw new Error(`Static fallback failed with ${response.status}`);
        }
        const payload = (await response.json()) as BoxSpreadPayload;
        if (!isCancelled) {
          setState({ data: payload, isLoading: false, error: null });
        }
      } catch (err) {
        if (!isCancelled) {
          setState({
            data: null,
            isLoading: false,
            error: err instanceof Error ? err.message : 'Unknown error',
          });
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

function mapApiResponse(json: Record<string, unknown>): BoxSpreadPayload {
  const rawScenarios = (json.scenarios ?? []) as Record<string, unknown>[];
  const scenarios: BoxSpreadScenario[] = rawScenarios.map((s) => ({
    width: (s.strike_width as number) ?? 0,
    put_bid: 0,
    call_ask: 0,
    synthetic_bid: 0,
    synthetic_ask: 0,
    mid_price: (s.estimated_net_debit as number) ?? (s.current_mark as number) ?? 0,
    annualized_return: (s.implied_apr as number) ?? (s.annualized_apr as number) ?? 0,
    fill_probability: s.type === 'indicative' ? 0.5 : 0.8,
    option_style: 'European' as const,
  }));

  return {
    as_of: (json.as_of as string) ?? new Date().toISOString(),
    underlying: (json.underlying as string) ?? 'SPX',
    scenarios,
  };
}
