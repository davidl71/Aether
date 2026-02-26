import { useEffect, useState, useCallback } from 'react';
import { SERVICE_PORTS } from '../config/ports';

export interface TastytradeAccount {
  accountId: string;
  accountType: string;
  nickname?: string;
}

export interface TastytradePosition {
  symbol: string;
  underlyingSymbol: string;
  quantity: number;
  averageOpenPrice: number;
  currentPrice: number;
  unrealizedPnl: number;
  instrumentType: string;
}

export interface TastytradeBalance {
  netLiquidatingValue: number;
  cashBalance: number;
  buyingPower: number;
  maintenanceRequirement: number;
  pendingCash: number;
}

export interface TastytradeSnapshot {
  account: TastytradeAccount;
  balance: TastytradeBalance;
  positions: TastytradePosition[];
  timestamp: string;
}

interface TastytradeState {
  snapshot: TastytradeSnapshot | null;
  isLoading: boolean;
  error: string | null;
  isAvailable: boolean;
}

const BASE_URL = `http://localhost:${SERVICE_PORTS.tastytrade}`;

export function useTastytrade(enabled = true) {
  const [state, setState] = useState<TastytradeState>({
    snapshot: null,
    isLoading: false,
    error: null,
    isAvailable: false,
  });

  const checkHealth = useCallback(async () => {
    try {
      const res = await fetch(`${BASE_URL}/api/health`, {
        signal: AbortSignal.timeout(3000),
      });
      return res.ok;
    } catch {
      return false;
    }
  }, []);

  const fetchSnapshot = useCallback(async () => {
    setState((prev) => ({ ...prev, isLoading: true }));
    try {
      const res = await fetch(`${BASE_URL}/api/snapshot`, {
        signal: AbortSignal.timeout(5000),
      });
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}: ${res.statusText}`);
      }
      const data = await res.json();

      const snapshot: TastytradeSnapshot = {
        account: {
          accountId: data.account_id ?? data.accountId ?? '',
          accountType: data.account_type ?? 'margin',
          nickname: data.nickname,
        },
        balance: {
          netLiquidatingValue: data.net_liquidating_value ?? data.account_value ?? 0,
          cashBalance: data.cash_balance ?? data.cash ?? 0,
          buyingPower: data.buying_power ?? 0,
          maintenanceRequirement: data.maintenance_requirement ?? 0,
          pendingCash: data.pending_cash ?? 0,
        },
        positions: (data.positions ?? []).map((p: Record<string, unknown>) => ({
          symbol: (p.symbol ?? p.ticker ?? '') as string,
          underlyingSymbol: (p.underlying_symbol ?? p.symbol ?? '') as string,
          quantity: (p.quantity ?? p.size ?? 0) as number,
          averageOpenPrice: (p.average_open_price ?? p.avg_price ?? 0) as number,
          currentPrice: (p.current_price ?? p.mark ?? 0) as number,
          unrealizedPnl: (p.unrealized_pnl ?? p.pnl ?? 0) as number,
          instrumentType: (p.instrument_type ?? p.type ?? 'Equity') as string,
        })),
        timestamp: (data.timestamp ?? new Date().toISOString()) as string,
      };

      setState({
        snapshot,
        isLoading: false,
        error: null,
        isAvailable: true,
      });
    } catch (err) {
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: err instanceof Error ? err.message : 'Unknown error',
      }));
    }
  }, []);

  useEffect(() => {
    if (!enabled) return;

    let cancelled = false;

    const init = async () => {
      const healthy = await checkHealth();
      if (cancelled) return;

      if (healthy) {
        setState((prev) => ({ ...prev, isAvailable: true }));
        await fetchSnapshot();
      } else {
        setState((prev) => ({
          ...prev,
          isAvailable: false,
          error: 'Tastytrade service not available',
        }));
      }
    };

    init();

    const interval = setInterval(async () => {
      if (cancelled) return;
      const healthy = await checkHealth();
      if (healthy && !cancelled) {
        fetchSnapshot();
      }
    }, 30_000);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [enabled, checkHealth, fetchSnapshot]);

  return { ...state, refresh: fetchSnapshot };
}
