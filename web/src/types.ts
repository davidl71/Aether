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

  // Expiration data for yield curve
  expiration_date?: string;          // Expiration date in YYYYMMDD or ISO format
  days_to_expiry?: number;           // Days until expiration
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

// Treasury Benchmark Types
export interface TreasuryBenchmark {
  maturity: string;              // e.g., "1MO", "3MO", "6MO", "1Y", "2Y", "3Y", "5Y", "7Y", "10Y", "20Y", "30Y"
  maturityDays: number;          // Approximate days to maturity
  yield: number;                 // Yield percentage (e.g., 5.25 for 5.25%)
  date: string;                  // Date of the yield data
  type: 'T-Bill' | 'T-Note' | 'T-Bond';
}

export interface TreasuryYieldData {
  benchmarks: TreasuryBenchmark[];
  lastUpdated: string;
}
