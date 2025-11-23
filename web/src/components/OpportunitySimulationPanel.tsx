import { useMemo, useState } from 'react';
import type { PositionSnapshot } from '../types/snapshot';
import { formatCurrency, formatPercent } from '../utils/formatters';
import '../styles/opportunity-simulation.css';

interface OpportunitySimulationPanelProps {
  positions: PositionSnapshot[];
  bankAccounts?: {
    account_path: string;
    account_name: string;
    bank_name: string | null;
    balance: number;
    currency: string;
    credit_rate: number | null;
    debit_rate: number | null;
  }[];
}

interface SimulationScenario {
  id: string;
  name: string;
  description: string;
  type: 'loan_consolidation' | 'margin_for_box_spread' | 'investment_fund' | 'multi_instrument';
  parameters: {
    loanAmount?: number;
    loanRate?: number;
    targetLoanRate?: number;
    boxSpreadRate?: number;
    fundReturn?: number;
    collateralValue?: number;
  };
  results?: {
    netBenefit: number;
    cashFlowImpact: number;
    riskReduction: number;
    capitalEfficiency: number;
  };
}

export function OpportunitySimulationPanel({
  positions,
  bankAccounts = [],
}: OpportunitySimulationPanelProps) {
  const [selectedScenario, setSelectedScenario] = useState<string | null>(null);
  const [scenarioParams, setScenarioParams] = useState<Record<string, any>>({});

  // Available scenarios based on current positions
  const availableScenarios = useMemo(() => {
    const scenarios: SimulationScenario[] = [];

    // Find loans
    const loans = positions.filter(
      p => p.instrument_type === 'bank_loan' || p.instrument_type === 'pension_loan'
    );
    const bankLoans = bankAccounts.filter(a => a.debit_rate && a.debit_rate > 0);

    // Scenario 1: Loan Consolidation
    if (loans.length > 1 || bankLoans.length > 0) {
      interface LoanCandidate { rate: number; name: string; balance: number }
      const loanCandidates: LoanCandidate[] = [
        ...loans.map(l => ({
          rate: l.rate || 0,
          name: l.name,
          balance: l.cash_flow || l.candle.close || 0
        })),
        ...bankLoans.map(a => ({
          rate: a.debit_rate || 0,
          name: a.account_name,
          balance: a.balance
        }))
      ].filter((l): l is LoanCandidate => l.rate > 0);

      if (loanCandidates.length > 0) {
        const highestRateLoan = loanCandidates.sort((a, b) => b.rate - a.rate)[0];

        if (highestRateLoan.rate > 0.03) { // Only if rate > 3%
          scenarios.push({
            id: 'loan_consolidation',
            name: 'Loan Consolidation',
            description: `Consolidate high-rate loans using lower-rate financing`,
            type: 'loan_consolidation',
            parameters: {
              loanAmount: highestRateLoan.balance,
              loanRate: highestRateLoan.rate,
              targetLoanRate: 0.04, // Assume 4% consolidation rate
            }
          });
        }
      }
    }

    // Scenario 2: Margin for Box Spreads
    const boxSpreads = positions.filter(p => p.instrument_type === 'box_spread');
    if (loans.length > 0 && boxSpreads.length > 0) {
      const loan = loans[0];
      const loanAmount = loan.cash_flow || loan.candle.close || 0;
      if (loanAmount > 0) {
        scenarios.push({
          id: 'margin_for_box_spread',
          name: 'Use Loan as Margin for Box Spreads',
          description: `Use loan proceeds as margin collateral for box spread positions`,
          type: 'margin_for_box_spread',
          parameters: {
            loanAmount,
            loanRate: loan.rate || 0,
            boxSpreadRate: boxSpreads[0].rate || 0.05,
          }
        });
      }
    }

    // Scenario 3: Investment Fund Strategy
    if (loans.length > 0) {
      const loan = loans[0];
      const loanAmount = loan.cash_flow || loan.candle.close || 0;
      if (loanAmount > 0) {
        scenarios.push({
          id: 'investment_fund',
          name: 'Investment Fund Strategy',
          description: `Use loan to invest in fund, use fund as collateral for cheaper loan`,
          type: 'investment_fund',
          parameters: {
            loanAmount,
            loanRate: loan.rate || 0,
            fundReturn: 0.06, // Assume 6% fund return
          }
        });
      }
    }

    return scenarios;
  }, [positions, bankAccounts]);

  // Calculate scenario results
  const scenarioResults = useMemo(() => {
    if (!selectedScenario) return null;

    const scenario = availableScenarios.find(s => s.id === selectedScenario);
    if (!scenario) return null;

    const params = scenarioParams[selectedScenario] || scenario.parameters;

    switch (scenario.type) {
      case 'loan_consolidation': {
        const currentCost = (params.loanAmount || 0) * (params.loanRate || 0);
        const newCost = (params.loanAmount || 0) * (params.targetLoanRate || 0);
        const netBenefit = currentCost - newCost;
        return {
          netBenefit,
          cashFlowImpact: netBenefit / 12, // Monthly benefit
          riskReduction: 0.15, // Estimated risk reduction
          capitalEfficiency: 1.0,
        };
      }

      case 'margin_for_box_spread': {
        const loanCost = (params.loanAmount || 0) * (params.loanRate || 0);
        const boxSpreadReturn = (params.loanAmount || 0) * (params.boxSpreadRate || 0);
        const netBenefit = boxSpreadReturn - loanCost;
        return {
          netBenefit,
          cashFlowImpact: netBenefit / 12,
          riskReduction: 0.05,
          capitalEfficiency: 1.2,
        };
      }

      case 'investment_fund': {
        const loanCost = (params.loanAmount || 0) * (params.loanRate || 0);
        const fundReturn = (params.loanAmount || 0) * (params.fundReturn || 0);
        const netBenefit = fundReturn - loanCost;
        return {
          netBenefit,
          cashFlowImpact: netBenefit / 12,
          riskReduction: 0.10,
          capitalEfficiency: 1.5,
        };
      }

      default:
        return null;
    }
  }, [selectedScenario, scenarioParams, availableScenarios]);

  return (
    <div className="opportunity-simulation-panel">
      <div className="opportunity-simulation-panel__header">
        <h2>Opportunity Simulation</h2>
        <p className="opportunity-simulation-panel__subtitle">
          Simulate what-if scenarios for loan usage and optimization
        </p>
      </div>

      <div className="opportunity-simulation-panel__content">
        <div className="opportunity-simulation-panel__scenarios">
          <h3>Available Scenarios</h3>
          {availableScenarios.length === 0 ? (
            <div className="opportunity-simulation-panel__empty">
              No scenarios available based on current positions
            </div>
          ) : (
            <div className="scenario-list">
              {availableScenarios.map((scenario) => (
                <div
                  key={scenario.id}
                  className={`scenario-card ${
                    selectedScenario === scenario.id ? 'scenario-card--selected' : ''
                  }`}
                  onClick={() => setSelectedScenario(scenario.id)}
                >
                  <div className="scenario-card__header">
                    <h4>{scenario.name}</h4>
                    <span className="scenario-card__type">{scenario.type.replace(/_/g, ' ')}</span>
                  </div>
                  <p className="scenario-card__description">{scenario.description}</p>
                </div>
              ))}
            </div>
          )}
        </div>

        {selectedScenario && scenarioResults && (
          <div className="opportunity-simulation-panel__results">
            <h3>Simulation Results</h3>
            <div className="simulation-results">
              <div className="simulation-result-card">
                <div className="simulation-result-card__label">Net Benefit (Annual)</div>
                <div
                  className={`simulation-result-card__value ${
                    scenarioResults.netBenefit >= 0
                      ? 'simulation-result-card__value--positive'
                      : 'simulation-result-card__value--negative'
                  }`}
                >
                  {formatCurrency(scenarioResults.netBenefit)}
                </div>
              </div>

              <div className="simulation-result-card">
                <div className="simulation-result-card__label">Cash Flow Impact (Monthly)</div>
                <div
                  className={`simulation-result-card__value ${
                    scenarioResults.cashFlowImpact >= 0
                      ? 'simulation-result-card__value--positive'
                      : 'simulation-result-card__value--negative'
                  }`}
                >
                  {formatCurrency(scenarioResults.cashFlowImpact)}
                </div>
              </div>

              <div className="simulation-result-card">
                <div className="simulation-result-card__label">Risk Reduction</div>
                <div className="simulation-result-card__value">
                  {formatPercent(scenarioResults.riskReduction)}
                </div>
              </div>

              <div className="simulation-result-card">
                <div className="simulation-result-card__label">Capital Efficiency</div>
                <div className="simulation-result-card__value">
                  {scenarioResults.capitalEfficiency.toFixed(2)}x
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
