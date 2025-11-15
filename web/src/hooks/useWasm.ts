// hooks/useWasm.ts - React hook for WASM module

import { useEffect, useState, useCallback } from 'react';
import { initWasm, isWasmReady, getWasmModule } from '../wasm/loader';
import type { BoxSpreadInput, BoxSpreadResult, RiskInput, RiskResult } from '../wasm/types';

export interface UseWasmReturn {
  ready: boolean;
  error: string | null;
  calculateBoxSpread: (input: BoxSpreadInput) => BoxSpreadResult;
  calculateRisk: (input: RiskInput) => RiskResult;
}

/**
 * React hook for WASM calculations
 *
 * @example
 * ```tsx
 * const { ready, calculateBoxSpread } = useWasm();
 *
 * if (!ready) return <div>Loading...</div>;
 *
 * const result = calculateBoxSpread({
 *   longCallStrike: 100,
 *   // ... other fields
 * });
 * ```
 */
export function useWasm(): UseWasmReturn {
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    initWasm()
      .then(() => {
        setReady(true);
        setError(null);
      })
      .catch((err) => {
        setError(err.message || 'Failed to initialize WASM');
        setReady(false);
      });
  }, []);

  const calculateBoxSpread = useCallback((input: BoxSpreadInput): BoxSpreadResult => {
    if (!isWasmReady()) {
      throw new Error('WASM not ready. Wait for initialization.');
    }

    try {
      const module = getWasmModule();
      return module.calculateBoxSpread(input);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Unknown error';
      throw new Error(`WASM calculation failed: ${errorMsg}`);
    }
  }, []);

  const calculateRisk = useCallback((input: RiskInput): RiskResult => {
    if (!isWasmReady()) {
      throw new Error('WASM not ready. Wait for initialization.');
    }

    try {
      const module = getWasmModule();
      return module.calculateRisk(input);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Unknown error';
      throw new Error(`WASM risk calculation failed: ${errorMsg}`);
    }
  }, []);

  return {
    ready,
    error,
    calculateBoxSpread,
    calculateRisk,
  };
}
