import { useEffect, useState } from 'react';
import type { TreasuryYieldData, TreasuryBenchmark } from '../types';
import { getServiceUrl } from '../config/ports';

interface HookState {
  data: TreasuryYieldData | null;
  isLoading: boolean;
  error: string | null;
}

// FRED API series IDs for Treasury yields
// No API key required for public data
const FRED_SERIES = {
  '1MO': 'DGS1MO',   // 1-Month Treasury Bill
  '3MO': 'DGS3MO',   // 3-Month Treasury Bill
  '6MO': 'DGS6MO',   // 6-Month Treasury Bill
  '1Y': 'DGS1',      // 1-Year Treasury Note
  '2Y': 'DGS2',      // 2-Year Treasury Note
  '3Y': 'DGS3',      // 3-Year Treasury Note
  '5Y': 'DGS5',      // 5-Year Treasury Note
  '7Y': 'DGS7',      // 7-Year Treasury Note
  '10Y': 'DGS10',    // 10-Year Treasury Note
  '20Y': 'DGS20',    // 20-Year Treasury Bond
  '30Y': 'DGS30'     // 30-Year Treasury Bond
};

// Approximate days to maturity for each benchmark
const MATURITY_DAYS: Record<string, number> = {
  '1MO': 30,
  '3MO': 90,
  '6MO': 180,
  '1Y': 365,
  '2Y': 730,
  '3Y': 1095,
  '5Y': 1825,
  '7Y': 2555,
  '10Y': 3650,
  '20Y': 7300,
  '30Y': 10950
};

const MATURITY_TYPES: Record<string, 'T-Bill' | 'T-Note' | 'T-Bond'> = {
  '1MO': 'T-Bill',
  '3MO': 'T-Bill',
  '6MO': 'T-Bill',
  '1Y': 'T-Note',
  '2Y': 'T-Note',
  '3Y': 'T-Note',
  '5Y': 'T-Note',
  '7Y': 'T-Note',
  '10Y': 'T-Note',
  '20Y': 'T-Bond',
  '30Y': 'T-Bond'
};

// FRED API base URL (no API key required for public data)
const FRED_API_BASE = 'https://api.stlouisfed.org/fred/series/observations';

// Cache key for localStorage
const CACHE_KEY = 'treasury_yields_cache';
const CACHE_DURATION_MS = 24 * 60 * 60 * 1000; // 24 hours

const RISK_FREE_RATE_TREASURY_URL = getServiceUrl('riskFreeRate', '/api/benchmarks/treasury');

function maturityLabel(tenor: string): string {
  const m: Record<string, string> = {
    '1M': '1MO', '3M': '3MO', '6M': '6MO', '1Y': '1Y', '2Y': '2Y', '3Y': '3Y',
    '5Y': '5Y', '7Y': '7Y', '10Y': '10Y', '20Y': '20Y', '30Y': '30Y',
  };
  return m[tenor] ?? tenor;
}

interface CachedData {
  data: TreasuryYieldData;
  timestamp: number;
}

function getCachedData(): TreasuryYieldData | null {
  try {
    const cached = localStorage.getItem(CACHE_KEY);
    if (!cached) return null;

    const parsed: CachedData = JSON.parse(cached);
    const now = Date.now();

    // Check if cache is still valid (within 24 hours)
    if (now - parsed.timestamp < CACHE_DURATION_MS) {
      return parsed.data;
    }

    // Cache expired, remove it
    localStorage.removeItem(CACHE_KEY);
    return null;
  } catch {
    return null;
  }
}

function setCachedData(data: TreasuryYieldData): void {
  try {
    const cached: CachedData = {
      data,
      timestamp: Date.now()
    };
    localStorage.setItem(CACHE_KEY, JSON.stringify(cached));
  } catch {
    // Ignore localStorage errors
  }
}

async function fetchTreasuryYield(seriesId: string): Promise<number | null> {
  try {
    // FRED API - no API key required, but we use a simple request
    // Note: For production, you may want to use a backend proxy to avoid CORS issues
    const url = `${FRED_API_BASE}?series_id=${seriesId}&api_key=free&file_type=json&limit=1&sort_order=desc`;

    // For now, we'll use a mock/fallback approach since FRED API may have CORS restrictions
    // In production, this should be proxied through the backend
    const response = await fetch(url, {
      mode: 'cors',
      credentials: 'omit'
    });

    if (!response.ok) {
      return null;
    }

    const json = await response.json();
    if (json.observations && json.observations.length > 0) {
      const latest = json.observations[0];
      const value = parseFloat(latest.value);
      return isNaN(value) ? null : value;
    }

    return null;
  } catch (error) {
    console.warn(`Failed to fetch Treasury yield for ${seriesId}:`, error);
    return null;
  }
}

// Alternative: Use Treasury.gov API or a backend proxy
// Prefer risk-free-rate service (backend with FRED key) to avoid CORS and use single source
async function fetchAllTreasuryYields(): Promise<TreasuryBenchmark[]> {
  try {
    const response = await fetch(RISK_FREE_RATE_TREASURY_URL, { credentials: 'omit' });
    if (response.ok) {
      const json = await response.json();
      const rates = json.rates ?? [];
      const benchmarks: TreasuryBenchmark[] = rates.map((r: { tenor: string; rate: number; days_to_expiry?: number; timestamp: string }) => ({
        maturity: maturityLabel(r.tenor),
        maturityDays: r.days_to_expiry ?? MATURITY_DAYS[maturityLabel(r.tenor)] ?? 365,
        yield: r.rate,
        date: (r.timestamp || json.timestamp || '').split('T')[0],
        type: MATURITY_TYPES[maturityLabel(r.tenor)] ?? 'T-Note',
      }));
      if (benchmarks.length > 0) return benchmarks;
    }
  } catch (e) {
    console.debug('Risk-Free Rate service Treasury fetch failed, falling back to FRED:', e);
  }

  // Fallback: FRED API (may fail due to CORS)
  const benchmarks: TreasuryBenchmark[] = [];
  const fetchPromises = Object.entries(FRED_SERIES).map(async ([maturity, seriesId]) => {
    const yieldValue = await fetchTreasuryYield(seriesId);

    if (yieldValue !== null) {
      benchmarks.push({
        maturity,
        maturityDays: MATURITY_DAYS[maturity],
        yield: yieldValue,
        date: new Date().toISOString().split('T')[0],
        type: MATURITY_TYPES[maturity]
      });
    }
  });

  await Promise.all(fetchPromises);

  // If we got no data (likely CORS issue), return empty array
  // The component will handle this gracefully
  return benchmarks;
}

export function useTreasuryYields(): HookState {
  const [state, setState] = useState<HookState>({
    data: null,
    isLoading: true,
    error: null
  });

  useEffect(() => {
    let isCancelled = false;

    async function fetchData() {
      // Check cache first
      const cached = getCachedData();
      if (cached) {
        if (!isCancelled) {
          setState({ data: cached, isLoading: false, error: null });
        }
        return;
      }

      try {
        setState((prev) => ({ ...prev, isLoading: true, error: null }));

        const benchmarks = await fetchAllTreasuryYields();

        if (isCancelled) return;

        const yieldData: TreasuryYieldData = {
          benchmarks,
          lastUpdated: new Date().toISOString()
        };

        // Cache the data
        setCachedData(yieldData);

        setState({ data: yieldData, isLoading: false, error: null });
      } catch (err) {
        if (!isCancelled) {
          setState({
            data: null,
            isLoading: false,
            error: err instanceof Error ? err.message : 'Failed to fetch Treasury yields'
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

/**
 * Find the closest Treasury benchmark for a given days to expiry
 */
export function findClosestBenchmark(
  daysToExpiry: number,
  benchmarks: TreasuryBenchmark[]
): TreasuryBenchmark | null {
  if (benchmarks.length === 0) return null;

  // Find benchmark with closest maturity days
  let closest = benchmarks[0];
  let minDiff = Math.abs(benchmarks[0].maturityDays - daysToExpiry);

  for (const benchmark of benchmarks) {
    const diff = Math.abs(benchmark.maturityDays - daysToExpiry);
    if (diff < minDiff) {
      minDiff = diff;
      closest = benchmark;
    }
  }

  return closest;
}
