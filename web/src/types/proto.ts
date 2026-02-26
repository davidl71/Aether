/**
 * Canonical TypeScript types mirroring proto/messages.proto
 * (ib.platform.v1).
 *
 * These are the cross-language contract types.  When ts-proto codegen
 * is wired in, replace imports of this module with generated code.
 */

// ---------------------------------------------------------------------------
// Market Data
// ---------------------------------------------------------------------------

export interface MarketDataEvent {
  symbol: string;
  bid: number;
  ask: number;
  last: number;
  volume: number;
  timestamp?: string;
}

export interface CandleSnapshotProto {
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  entry: number;
  updated?: string;
}

export interface SymbolSnapshotProto {
  symbol: string;
  last: number;
  bid: number;
  ask: number;
  spread: number;
  roi: number;
  maker_count: number;
  taker_count: number;
  volume: number;
  candle?: CandleSnapshotProto;
}

// ---------------------------------------------------------------------------
// Positions & Orders
// ---------------------------------------------------------------------------

export interface Position {
  id: string;
  symbol: string;
  quantity: number;
  cost_basis: number;
  mark: number;
  unrealized_pnl: number;
}

export interface HistoricPosition {
  id: string;
  symbol: string;
  quantity: number;
  realized_pnl: number;
  closed_at?: string;
}

export interface Order {
  id: string;
  symbol: string;
  side: string;
  quantity: number;
  status: string;
  submitted_at?: string;
}

// ---------------------------------------------------------------------------
// Strategy
// ---------------------------------------------------------------------------

export interface StrategyDecision {
  symbol: string;
  quantity: number;
  side: string;
  mark: number;
  created_at?: string;
}

export interface StrategySignal {
  symbol: string;
  price: number;
  timestamp?: string;
}

// ---------------------------------------------------------------------------
// Risk
// ---------------------------------------------------------------------------

export interface RiskStatus {
  allowed: boolean;
  reason: string;
  updated_at?: string;
}

export interface RiskLimit {
  symbol: string;
  max_position: number;
  max_notional: number;
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

export type AlertLevel = 'INFO' | 'WARNING' | 'ERROR';

export interface Alert {
  level: AlertLevel;
  message: string;
  timestamp?: string;
}

export interface Metrics {
  net_liq: number;
  buying_power: number;
  excess_liquidity: number;
  margin_requirement: number;
  commissions: number;
  portal_ok: boolean;
  tws_ok: boolean;
  orats_ok: boolean;
  questdb_ok: boolean;
  nats_ok: boolean;
}

export interface SystemSnapshot {
  generated_at?: string;
  started_at?: string;
  mode: string;
  strategy: string;
  account_id: string;
  metrics?: Metrics;
  symbols: SymbolSnapshotProto[];
  positions: Position[];
  historic: HistoricPosition[];
  orders: Order[];
  decisions: StrategyDecision[];
  alerts: Alert[];
  risk?: RiskStatus;
}

// ---------------------------------------------------------------------------
// Box Spread
// ---------------------------------------------------------------------------

export interface BoxSpreadScenario {
  symbol: string;
  strike_width: number;
  theoretical_value: number;
  estimated_net_debit: number;
  implied_apr: number;
  scenario_type: string;
}

export interface BoxSpreadExecution {
  symbol: string;
  lower_strike: number;
  upper_strike: number;
  expiry: string;
  net_debit: number;
  trade_id: string;
  executed_at?: string;
}
