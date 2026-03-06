import { useEffect, useState } from 'react';
import { getServiceUrl } from '../config/ports';

export interface SofrOvernight {
  rate: number | null;
  timestamp: string | null;
}

export interface BenchmarkRate {
  tenor: string;
  rate: number;
  days_to_expiry?: number;
  timestamp: string;
}

export interface SofrResponse {
  overnight: SofrOvernight;
  term_rates: BenchmarkRate[];
  timestamp: string;
}

export interface TreasuryResponse {
  rates: BenchmarkRate[];
  timestamp: string;
}

export interface RiskFreeRateBenchmarksState {
  sofr: SofrResponse | null;
  treasury: TreasuryResponse | null;
  isLoading: boolean;
  error: string | null;
}

const SOFR_URL = getServiceUrl('riskFreeRate', '/api/benchmarks/sofr');
const TREASURY_URL = getServiceUrl('riskFreeRate', '/api/benchmarks/treasury');

export function useRiskFreeRateBenchmarks(refreshIntervalMs = 60_000): RiskFreeRateBenchmarksState {
  const [state, setState] = useState<RiskFreeRateBenchmarksState>({
    sofr: null,
    treasury: null,
    isLoading: true,
    error: null,
  });

  useEffect(() => {
    let cancelled = false;

    async function fetchAll() {
      setState((s) => ({ ...s, isLoading: true, error: null }));
      try {
        const [sofrRes, treasuryRes] = await Promise.all([
          fetch(SOFR_URL, { credentials: 'omit' }),
          fetch(TREASURY_URL, { credentials: 'omit' }),
        ]);

        if (cancelled) return;

        const sofrOk = sofrRes.ok;
        const treasuryOk = treasuryRes.ok;

        const sofr: SofrResponse | null = sofrOk ? await sofrRes.json() : null;
        const treasury: TreasuryResponse | null = treasuryOk ? await treasuryRes.json() : null;

        if (cancelled) return;

        const error =
          !sofrOk && !treasuryOk
            ? 'Risk-Free Rate service unavailable. Start the service (port 8004) and ensure FRED API key is set.'
            : null;

        setState({ sofr, treasury, isLoading: false, error });
      } catch (err) {
        if (!cancelled) {
          setState({
            sofr: null,
            treasury: null,
            isLoading: false,
            error: err instanceof Error ? err.message : 'Failed to fetch benchmarks',
          });
        }
      }
    }

    void fetchAll();
    const id = refreshIntervalMs > 0 ? setInterval(fetchAll, refreshIntervalMs) : undefined;
    return () => {
      cancelled = true;
      if (id) clearInterval(id);
    };
  }, [refreshIntervalMs]);

  return state;
}
