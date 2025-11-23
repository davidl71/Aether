export interface Candle {
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  entry: number;
  updated: string;
}

export interface OptionStrike {
  strike: number;
  call_bid: number;
  call_ask: number;
  put_bid: number;
  put_ask: number;
}

export interface OptionSeries {
  expiration: string;
  strikes: OptionStrike[];
}

export interface SymbolSnapshot {
  symbol: string;
  last: number;
  bid: number;
  ask: number;
  spread: number;
  roi: number;
  maker_count: number;
  taker_count: number;
  volume: number;
  candle: Candle;
  option_chains: OptionSeries[];
}

export type InstrumentType =
  | 'box_spread'
  | 'bank_loan'
  | 'pension_loan'
  | 'bond'
  | 't_bill'
  | 'futures'
  | 'other';

export interface PositionSnapshot {
  name: string;
  quantity: number;
  roi: number;
  maker_count: number;
  taker_count: number;
  rebate_estimate: number;
  vega: number;
  theta: number;
  fair_diff: number;
  candle: Candle;
  // Extended fields for unified positions
  instrument_type?: InstrumentType;
  rate?: number; // Annual rate (APR) for loans/financing
  maturity_date?: string; // ISO 8601 date string
  cash_flow?: number; // Expected cash flow amount
  collateral_value?: number; // Collateral value if applicable
  currency?: string; // Currency code (defaults to USD)
}

export interface TimelineEvent {
  timestamp: string;
  text: string;
  severity: 'info' | 'success' | 'warn' | 'warning' | 'error' | 'critical';
}

export interface AccountMetrics {
  net_liq: number;
  buying_power: number;
  excess_liquidity: number;
  margin_requirement: number;
  commissions: number;
  portal_ok: boolean;
  tws_ok: boolean;
  orats_ok: boolean;
  questdb_ok: boolean;
}

export interface SnapshotPayload {
  generated_at: string;
  mode: string;
  strategy: string;
  account_id: string;
  metrics: AccountMetrics;
  symbols: SymbolSnapshot[];
  positions: PositionSnapshot[];
  historic: PositionSnapshot[];
  orders: TimelineEvent[];
  alerts: TimelineEvent[];
}

export type Severity = TimelineEvent['severity'];
