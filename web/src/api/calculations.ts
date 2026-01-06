/**
 * Calculations API Client
 *
 * Client for shared calculation endpoints (cash flow timeline, opportunity simulation).
 * Uses the calculations API service (python/services/calculations_api.py).
 */

import type { PositionSnapshot } from '../types/snapshot';

const DEFAULT_CALCULATIONS_API_URL = 'http://localhost:8004';

function getCalculationsApiUrl(): string {
  const env = (import.meta as unknown as { env?: Record<string, unknown> }).env;
  if (typeof env?.VITE_CALCULATIONS_API_URL === 'string' && env.VITE_CALCULATIONS_API_URL.length > 0) {
    return env.VITE_CALCULATIONS_API_URL;
  }
  return DEFAULT_CALCULATIONS_API_URL;
}

export interface PositionInput {
  name: string;
  maturity_date?: string | null;
  cash_flow?: number | null;
  candle?: { close?: number | null } | null;
  instrument_type?: string | null;
  rate?: number | null;
}

export interface BankAccountInput {
  account_name: string;
  balance: number;
  debit_rate?: number | null;
  credit_rate?: number | null;
  currency?: string | null;
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

/**
 * Convert PositionSnapshot to PositionInput format
 */
function positionToInput(position: PositionSnapshot): PositionInput {
  return {
    name: position.name,
    maturity_date: position.maturity_date || null,
    cash_flow: position.cash_flow ?? null,
    candle: position.candle ? { close: position.candle.close ?? null } : null,
    instrument_type: position.instrument_type || null,
    rate: position.rate ?? null,
  };
}

/**
 * Convert bank account to BankAccountInput format
 */
function bankAccountToInput(account: {
  account_name: string;
  balance: number;
  debit_rate?: number | null;
  credit_rate?: number | null;
  currency?: string | null;
}): BankAccountInput {
  return {
    account_name: account.account_name,
    balance: account.balance,
    debit_rate: account.debit_rate ?? null,
    credit_rate: account.credit_rate ?? null,
    currency: account.currency || null,
  };
}

/**
 * Calculate cash flow timeline
 */
export async function calculateCashFlowTimeline(
  positions: PositionSnapshot[],
  bankAccounts: Array<{
    account_name: string;
    balance: number;
    debit_rate?: number | null;
    credit_rate?: number | null;
    currency?: string | null;
  }>,
  projectionMonths: number = 12
): Promise<CashFlowTimelineResponse> {
  const apiUrl = getCalculationsApiUrl();
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
  bankAccounts: Array<{
    account_name: string;
    balance: number;
    debit_rate?: number | null;
    credit_rate?: number | null;
  }>
): Promise<SimulationScenario[]> {
  const apiUrl = getCalculationsApiUrl();
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
  const apiUrl = getCalculationsApiUrl();
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
