// wasm/types.ts - TypeScript types matching WASM C++ types

export interface BoxSpreadInput {
  longCallStrike: number;
  longCallBid: number;
  longCallAsk: number;
  shortCallStrike: number;
  shortCallBid: number;
  shortCallAsk: number;
  longPutStrike: number;
  longPutBid: number;
  longPutAsk: number;
  shortPutStrike: number;
  shortPutBid: number;
  shortPutAsk: number;
  underlyingPrice: number;
  riskFreeRate: number;
  daysToExpiry: number;
  volatility?: number;  // Optional, for Greeks calculations
}

export interface BoxSpreadResult {
  netDebit: number;
  arbitrageProfit: number;
  roi: number;
  apr: number;
  confidenceScore: number;
  isProfitable: boolean;
  riskScore: number;
  delta: number;
  gamma: number;
  theta: number;
  vega: number;
}

export interface RiskInput {
  positionSize: number;
  volatility: number;
  timeHorizonDays: number;
  confidenceLevel: number;
}

export interface RiskResult {
  var: number;
  expectedLoss: number;
  maxLoss: number;
  positionSizeLimit: number;
}
