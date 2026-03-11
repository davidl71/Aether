/**
 * Frontend calculations client
 *
 * Client for shared frontend read-model endpoints owned by the Rust API.
 */

import type { PositionSnapshot } from '../types/snapshot';
import type { BankAccount } from '../types/banking';

const DEFAULT_FRONTEND_API_URL = 'http://localhost:8080';

function getFrontendApiUrl(): string {
  const env = (import.meta as unknown as { env?: Record<string, unknown> }).env;
  if (typeof env?.VITE_API_URL === 'string' && env.VITE_API_URL.length > 0) {
    return env.VITE_API_URL;
  }
  return DEFAULT_FRONTEND_API_URL;
}

export interface PositionInput {
  name: string;
  quantity?: number | null;
  roi?: number | null;
  maker_count?: number | null;
  taker_count?: number | null;
  rebate_estimate?: number | null;
  vega?: number | null;
  theta?: number | null;
  fair_diff?: number | null;
  maturity_date?: string | null;
  cash_flow?: number | null;
  candle?: { close?: number | null } | null;
  instrument_type?: string | null;
  rate?: number | null;
  collateral_value?: number | null;
  currency?: string | null;
  market_value?: number | null;
  bid?: number | null;
  ask?: number | null;
  last?: number | null;
  spread?: number | null;
  price?: number | null;
  side?: string | null;
  expected_cash_at_expiry?: number | null;
  dividend?: number | null;
  conid?: number | null;
}

export interface BankAccountInput {
  account_name: string;
  balance: number;
  account_path?: string | null;
  bank_name?: string | null;
  account_number?: string | null;
  debit_rate?: number | null;
  credit_rate?: number | null;
  currency?: string | null;
  balances_by_currency?: Record<string, number> | null;
  is_mixed_currency?: boolean;
}

export interface CashFlowEvent {
  date: string; // ISO date string (YYYY-MM-DD)
  amount: number; // Positive for inflows, negative for outflows
  description: string;
  position_name: string;
  type: string; // 'maturity', 'loan_payment', 'other'
}

export interface MonthlyCashFlow {
  month: string; // YYYY-MM
  inflows: number;
  outflows: number;
  net: number;
  events: CashFlowEvent[];
}

export interface CashFlowTimelineRequest {
  positions: PositionInput[];
  bank_accounts: BankAccountInput[];
  projection_months: number;
}

export interface CashFlowTimelineResponse {
  events: CashFlowEvent[];
  monthly_flows: Record<string, MonthlyCashFlow>; // Key: YYYY-MM
  total_inflows: number;
  total_outflows: number;
  net_cash_flow: number;
}

export interface SimulationScenario {
  id: string;
  name: string;
  type: string;
  description: string;
  parameters: Record<string, number>;
  net_benefit: number;
}

export interface OpportunitySimulationRequest {
  positions: PositionInput[];
  bank_accounts: BankAccountInput[];
}

export interface ScenarioCalculationRequest {
  scenario: {
    id: string;
    name: string;
    type: string;
    description: string;
    parameters: Record<string, number>;
  };
}

export interface ScenarioCalculationResponse {
  net_benefit: number;
  cash_flow_impact: number;
  risk_reduction: number;
  capital_efficiency?: number | null;
}

export interface FrontendRelationship {
  from: string;
  to: string;
  type: 'collateral' | 'margin' | 'financing' | 'investment';
  description: string;
  value: number;
}

export interface UnifiedPositionsResponse {
  positions: PositionSnapshot[];
}

export interface RelationshipResponse {
  relationships: FrontendRelationship[];
  nodes: string[];
}

/**
 * Convert PositionSnapshot to PositionInput format
 */
function positionToInput(position: PositionSnapshot): PositionInput {
  return {
    name: position.name,
    quantity: position.quantity,
    roi: position.roi,
    maker_count: position.maker_count,
    taker_count: position.taker_count,
    rebate_estimate: position.rebate_estimate,
    vega: position.vega,
    theta: position.theta,
    fair_diff: position.fair_diff,
    maturity_date: position.maturity_date || null,
    cash_flow: position.cash_flow ?? null,
    candle: position.candle ? { close: position.candle.close ?? null } : null,
    instrument_type: position.instrument_type || null,
    rate: position.rate ?? null,
    collateral_value: position.collateral_value ?? null,
    currency: position.currency ?? null,
    market_value: position.market_value ?? null,
    bid: position.bid ?? null,
    ask: position.ask ?? null,
    last: position.last ?? null,
    spread: position.spread ?? null,
    price: position.price ?? null,
    side: position.side ?? null,
    expected_cash_at_expiry: position.expected_cash_at_expiry ?? null,
    dividend: position.dividend ?? null,
    conid: position.conid ?? null,
  };
}

/**
 * Convert bank account to BankAccountInput format
 */
function bankAccountToInput(account: BankAccount): BankAccountInput {
  return {
    account_name: account.account_name,
    balance: account.balance,
    account_path: account.account_path,
    bank_name: account.bank_name,
    account_number: account.account_number,
    debit_rate: account.debit_rate ?? null,
    credit_rate: account.credit_rate ?? null,
    currency: account.currency || null,
    balances_by_currency: account.balances_by_currency ?? null,
    is_mixed_currency: account.is_mixed_currency ?? false,
  };
}

/**
 * Calculate cash flow timeline
 */
export async function calculateCashFlowTimeline(
  positions: PositionSnapshot[],
  bankAccounts: BankAccount[],
  projectionMonths: number = 12
): Promise<CashFlowTimelineResponse> {
  const apiUrl = getFrontendApiUrl();
  const request: CashFlowTimelineRequest = {
    positions: positions.map(positionToInput),
    bank_accounts: bankAccounts.map(bankAccountToInput),
    projection_months: projectionMonths,
  };

  const response = await fetch(`${apiUrl}/api/v1/cash-flow/timeline`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Cash flow calculation failed: ${response.status} ${errorText}`);
  }

  return response.json();
}

/**
 * Find available opportunity simulation scenarios
 */
export async function findSimulationScenarios(
  positions: PositionSnapshot[],
  bankAccounts: BankAccount[]
): Promise<SimulationScenario[]> {
  const apiUrl = getFrontendApiUrl();
  const request: OpportunitySimulationRequest = {
    positions: positions.map(positionToInput),
    bank_accounts: bankAccounts.map(bankAccountToInput),
  };

  const response = await fetch(`${apiUrl}/api/v1/opportunity-simulation/scenarios`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Scenario discovery failed: ${response.status} ${errorText}`);
  }

  return response.json();
}

export async function fetchUnifiedPositions(
  positions: PositionSnapshot[],
  bankAccounts: BankAccount[]
): Promise<UnifiedPositionsResponse> {
  const apiUrl = getFrontendApiUrl();
  const response = await fetch(`${apiUrl}/api/v1/frontend/unified-positions`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      positions: positions.map(positionToInput),
      bank_accounts: bankAccounts.map(bankAccountToInput),
    }),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Unified positions fetch failed: ${response.status} ${errorText}`);
  }

  return response.json();
}

export async function fetchRelationships(
  positions: PositionSnapshot[],
  bankAccounts: BankAccount[]
): Promise<RelationshipResponse> {
  const apiUrl = getFrontendApiUrl();
  const response = await fetch(`${apiUrl}/api/v1/frontend/relationships`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      positions: positions.map(positionToInput),
      bank_accounts: bankAccounts.map(bankAccountToInput),
    }),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Relationship fetch failed: ${response.status} ${errorText}`);
  }

  return response.json();
}

/**
 * Calculate detailed results for a scenario
 */
export async function calculateScenarioResults(
  scenario: {
    id: string;
    name: string;
    type: string;
    description: string;
    parameters: Record<string, number>;
  }
): Promise<ScenarioCalculationResponse> {
  const apiUrl = getFrontendApiUrl();
  const request: ScenarioCalculationRequest = { scenario };

  const response = await fetch(`${apiUrl}/api/v1/opportunity-simulation/calculate`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Scenario calculation failed: ${response.status} ${errorText}`);
  }

  return response.json();
}
