import { useEffect, useState } from 'react';
import type { CandlestickData, Timeframe } from '../types/chart';

interface UseChartDataOptions {
  symbol: string;
  timeframe: Timeframe;
  apiBaseUrl?: string;
}

/**
 * Hook to fetch candlestick chart data
 * Currently returns mock data - should be replaced with actual API call
 */
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
        // TODO: Replace with actual API endpoint
        // For now, generate mock data
        const mockData = generateMockCandlestickData(timeframe);

        if (!cancelled) {
          setData(mockData);
          setIsLoading(false);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : 'Failed to fetch chart data');
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

/**
 * Generate mock candlestick data for development
 * TODO: Replace with actual API call
 */
function generateMockCandlestickData(timeframe: Timeframe): CandlestickData[] {
  const now = Math.floor(Date.now() / 1000);
  const intervals = {
    '1D': 300, // 5 minutes
    '1W': 3600, // 1 hour
    '1M': 14400, // 4 hours
    '3M': 86400, // 1 day
    '1Y': 86400 * 7 // 1 week
  };

  const interval = intervals[timeframe];
  const count = {
    '1D': 288, // 24 hours * 12 (5-min intervals)
    '1W': 168, // 7 days * 24 hours
    '1M': 180, // 30 days * 6 (4-hour intervals)
    '3M': 90, // 90 days
    '1Y': 52 // 52 weeks
  };

  const dataCount = count[timeframe];
  const basePrice = 4500; // Example: SPX around 4500
  const data: CandlestickData[] = [];

  let currentPrice = basePrice;

  for (let i = dataCount - 1; i >= 0; i--) {
    const time = now - i * interval;
    const change = (Math.random() - 0.5) * 20; // Random price movement
    const open = currentPrice;
    const close = open + change;
    const high = Math.max(open, close) + Math.random() * 10;
    const low = Math.min(open, close) - Math.random() * 10;
    const volume = Math.floor(Math.random() * 1000000) + 100000;

    data.push({
      time,
      open,
      high,
      low,
      close,
      volume
    });

    currentPrice = close;
  }

  return data;
}
