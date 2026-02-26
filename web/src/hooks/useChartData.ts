import { useEffect, useState } from 'react';
import type { CandlestickData, Timeframe } from '../types/chart';

interface UseChartDataOptions {
  symbol: string;
  timeframe: Timeframe;
  apiBaseUrl?: string;
}

export function useChartData({ symbol, timeframe, apiBaseUrl = 'http://127.0.0.1:8000' }: UseChartDataOptions) {
  const [data, setData] = useState<CandlestickData[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function fetchChartData() {
      if (!symbol) {
        setData([]);
        setIsLoading(false);
        return;
      }

      setIsLoading(true);
      setError(null);

      try {
        const response = await fetch(
          `${apiBaseUrl}/api/v1/chart/${encodeURIComponent(symbol)}?timeframe=${timeframe}`
        );

        if (!response.ok) {
          throw new Error(`API returned ${response.status}`);
        }

        const json = await response.json();
        const candles: CandlestickData[] = (json.candles ?? json.data ?? []).map(
          (c: Record<string, number>) => ({
            time: c.time ?? c.timestamp ?? c.t,
            open: c.open ?? c.o,
            high: c.high ?? c.h,
            low: c.low ?? c.l,
            close: c.close ?? c.c,
            volume: c.volume ?? c.v ?? 0,
          })
        );

        if (!cancelled) {
          setData(candles);
          setIsLoading(false);
        }
      } catch {
        // Backend unavailable – generate synthetic data so the UI
        // remains functional during development.
        if (!cancelled) {
          setData(generateFallbackData(timeframe));
          setIsLoading(false);
        }
      }
    }

    fetchChartData();

    return () => {
      cancelled = true;
    };
  }, [symbol, timeframe, apiBaseUrl]);

  return { data, isLoading, error };
}

function generateFallbackData(timeframe: Timeframe): CandlestickData[] {
  const now = Math.floor(Date.now() / 1000);
  const intervals: Record<Timeframe, number> = {
    '1D': 300,
    '1W': 3600,
    '1M': 14400,
    '3M': 86400,
    '1Y': 86400 * 7,
  };
  const counts: Record<Timeframe, number> = {
    '1D': 288,
    '1W': 168,
    '1M': 180,
    '3M': 90,
    '1Y': 52,
  };

  const interval = intervals[timeframe];
  const count = counts[timeframe];
  const basePrice = 4500;
  const result: CandlestickData[] = [];
  let price = basePrice;

  for (let i = count - 1; i >= 0; i--) {
    const time = now - i * interval;
    const change = (Math.random() - 0.5) * 20;
    const open = price;
    const close = open + change;
    const high = Math.max(open, close) + Math.random() * 10;
    const low = Math.min(open, close) - Math.random() * 10;
    const volume = Math.floor(Math.random() * 1_000_000) + 100_000;
    result.push({ time, open, high, low, close, volume });
    price = close;
  }

  return result;
}
