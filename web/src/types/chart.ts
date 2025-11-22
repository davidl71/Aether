/**
 * Chart data types for candlestick charts
 */

export interface CandlestickData {
  time: number; // Unix timestamp in seconds
  open: number;
  high: number;
  low: number;
  close: number;
  volume?: number;
}

export interface ChartData {
  symbol: string;
  timeframe: '1D' | '1W' | '1M' | '3M' | '1Y';
  candles: CandlestickData[];
}

export type Timeframe = '1D' | '1W' | '1M' | '3M' | '1Y';
