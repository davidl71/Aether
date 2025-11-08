export interface BoxSpreadScenario {
  width: number;
  put_bid: number;
  call_ask: number;
  synthetic_bid: number;
  synthetic_ask: number;
  mid_price: number;
  annualized_return: number;
  fill_probability: number;
}

export interface BoxSpreadPayload {
  as_of: string;
  underlying: string;
  scenarios: BoxSpreadScenario[];
}

export interface BoxSpreadSummary {
  totalScenarios: number;
  avgApr: number;
  probableCount: number;
  maxAprScenario: BoxSpreadScenario | null;
}
