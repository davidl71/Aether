import { useEffect, useState, useRef, useCallback } from 'react';

export interface FetchState<T> {
  data: T | null;
  isLoading: boolean;
  error: string | null;
  refetch: () => void;
}

export interface UseFetchJSONOptions {
  /** Polling interval in milliseconds. No polling if omitted. */
  pollIntervalMs?: number;
  /** Skip fetching when true (e.g. missing symbol). */
  skip?: boolean;
}

/**
 * Generic hook for fetching JSON data with loading/error state.
 *
 * @param url   - The URL to fetch.
 * @param transform - Optional function to transform the raw JSON before storing.
 * @param options - Polling interval, skip flag, etc.
 */
export function useFetchJSON<T>(
  url: string,
  transform?: (json: unknown) => T,
  options: UseFetchJSONOptions = {},
): FetchState<T> {
  const { pollIntervalMs, skip = false } = options;
  const [data, setData] = useState<T | null>(null);
  const [isLoading, setIsLoading] = useState(!skip);
  const [error, setError] = useState<string | null>(null);
  const generationRef = useRef(0);

  const doFetch = useCallback(async (gen: number) => {
    try {
      setIsLoading(true);
      setError(null);

      const resp = await fetch(url);
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);

      const json = await resp.json();
      if (gen !== generationRef.current) return;

      const result = transform ? transform(json) : (json as T);
      setData(result);
    } catch (err) {
      if (gen !== generationRef.current) return;
      setError(err instanceof Error ? err.message : 'Fetch failed');
    } finally {
      if (gen === generationRef.current) setIsLoading(false);
    }
  }, [url, transform]);

  const refetch = useCallback(() => {
    const gen = ++generationRef.current;
    doFetch(gen);
  }, [doFetch]);

  useEffect(() => {
    if (skip) {
      setData(null);
      setIsLoading(false);
      return;
    }

    const gen = ++generationRef.current;
    doFetch(gen);

    if (pollIntervalMs && pollIntervalMs > 0) {
      const id = setInterval(() => doFetch(generationRef.current), pollIntervalMs);
      return () => clearInterval(id);
    }
  }, [url, skip, pollIntervalMs, doFetch]);

  return { data, isLoading, error, refetch };
}
