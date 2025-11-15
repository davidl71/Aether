export interface BoxSpreadScenario {
  width: number;
  put_bid: number;
  call_ask: number;
  synthetic_bid: number;
  synthetic_ask: number;
  mid_price: number;
  annualized_return: number;
  fill_probability: number;
  option_style: 'European' | 'American';

  // Buy vs Sell disparity (intraday differences)
  buy_profit?: number;              // Profit from buying box spread
  buy_implied_rate?: number;        // Implied rate when buying
  sell_profit?: number;             // Profit from selling box spread
  sell_implied_rate?: number;       // Implied rate when selling
  buy_sell_disparity?: number;      // Difference between buy and sell profitability
  put_call_parity_violation?: number; // Put-call parity violation (bps)
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
