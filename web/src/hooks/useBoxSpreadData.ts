import { useEffect, useState, useRef, useCallback } from 'react';
import type { BoxSpreadPayload, BoxSpreadScenario } from '../types';
import { getRustBackendUrl } from '../config/ports';

interface HookState {
  data: BoxSpreadPayload | null;
  isLoading: boolean;
  error: string | null;
}

const STATIC_FALLBACK_URL = '/data/box_spread_sample.json';
const POLL_INTERVAL_MS = 10_000;

export function useBoxSpreadData(): HookState {
  const [state, setState] = useState<HookState>({
    data: null,
    isLoading: true,
    error: null
  });
  const generationRef = useRef(0);
  const usingApiRef = useRef(false);

  const fetchData = useCallback(async (gen: number) => {
    const apiUrl = getRustBackendUrl('/api/v1/scenarios');

    try {
      const response = await fetch(apiUrl, {
        headers: { 'cache-control': 'no-cache' },
      });
      if (!response.ok) {
        throw new Error(`API returned ${response.status}`);
      }
      const json = await response.json();
      const payload = mapApiResponse(json);
      if (gen === generationRef.current) {
        usingApiRef.current = true;
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
      if (gen === generationRef.current) {
        usingApiRef.current = false;
        setState({ data: payload, isLoading: false, error: null });
      }
    } catch (err) {
      if (gen === generationRef.current) {
        setState({
          data: null,
          isLoading: false,
          error: err instanceof Error ? err.message : 'Unknown error',
        });
      }
    }
  }, []);

  useEffect(() => {
    const gen = ++generationRef.current;

    void fetchData(gen);

    const intervalId = setInterval(() => {
      void fetchData(gen);
    }, POLL_INTERVAL_MS);

    return () => {
      generationRef.current++;
      clearInterval(intervalId);
    };
  }, [fetchData]);

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
