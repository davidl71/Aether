import { useMemo, useState, useEffect } from 'react';
import type { PositionSnapshot } from '../types/snapshot';
import { formatCurrency, formatPercent } from '../utils/formatters';
import {
  findSimulationScenarios,
  calculateScenarioResults,
  type SimulationScenario,
  type ScenarioCalculationResponse,
} from '../api/calculations';
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
  const [availableScenarios, setAvailableScenarios] = useState<SimulationScenario[]>([]);
  const [scenarioResults, setScenarioResults] = useState<ScenarioCalculationResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [loadingResults, setLoadingResults] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Fetch available scenarios from API
  useEffect(() => {
    let cancelled = false;

    async function fetchScenarios() {
      setLoading(true);
      setError(null);

      try {
        const scenarios = await findSimulationScenarios(positions, bankAccounts);
        if (!cancelled) {
          setAvailableScenarios(scenarios);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : 'Failed to find scenarios');
          console.error('Scenario discovery error:', err);
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    fetchScenarios();

    return () => {
      cancelled = true;
    };
  }, [positions, bankAccounts]);

  // Calculate results when scenario is selected
  useEffect(() => {
    if (!selectedScenario) {
      setScenarioResults(null);
      return;
    }

    let cancelled = false;

    async function fetchResults() {
      const scenario = availableScenarios.find(s => s.id === selectedScenario);
      if (!scenario) return;

      setLoadingResults(true);

      try {
        const results = await calculateScenarioResults({
          id: scenario.id,
          name: scenario.name,
          type: scenario.type,
          description: scenario.description,
          parameters: scenario.parameters,
        });
        if (!cancelled) {
          setScenarioResults(results);
        }
      } catch (err) {
        if (!cancelled) {
          console.error('Scenario calculation error:', err);
          setScenarioResults(null);
        }
      } finally {
        if (!cancelled) {
          setLoadingResults(false);
        }
      }
    }

    fetchResults();

    return () => {
      cancelled = true;
    };
  }, [selectedScenario, availableScenarios]);

  return (
    <div className="opportunity-simulation-panel">
      <div className="opportunity-simulation-panel__header">
        <h2>Opportunity Simulation</h2>
        <p className="opportunity-simulation-panel__subtitle">
          Simulate what-if scenarios for loan usage and optimization
        </p>
      </div>

      {error && (
        <div className="opportunity-simulation-panel__error" style={{
          padding: '12px 16px',
          marginBottom: '16px',
          background: 'rgba(239, 68, 68, 0.1)',
          border: '1px solid rgba(239, 68, 68, 0.3)',
          borderRadius: '8px',
          color: '#ef4444'
        }}>
          <strong>Error:</strong> {error}
        </div>
      )}

      <div className="opportunity-simulation-panel__content">
        <div className="opportunity-simulation-panel__scenarios">
          <h3>Available Scenarios</h3>
          {loading ? (
            <div className="opportunity-simulation-panel__empty">
              Loading scenarios...
            </div>
          ) : availableScenarios.length === 0 ? (
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
                  {scenario.net_benefit !== undefined && (
                    <div className="scenario-card__net-benefit">
                      Net Benefit: {formatCurrency(scenario.net_benefit)}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        {selectedScenario && (
          loadingResults ? (
            <div className="opportunity-simulation-panel__results">
              <h3>Simulation Results</h3>
              <div style={{ textAlign: 'center', padding: '20px', color: '#9ca3af' }}>
                Calculating results...
              </div>
            </div>
          ) : scenarioResults && (
          <div className="opportunity-simulation-panel__results">
            <h3>Simulation Results</h3>
            <div className="simulation-results">
              <div className="simulation-result-card">
                <div className="simulation-result-card__label">Net Benefit (Annual)</div>
                <div
                  className={`simulation-result-card__value ${
                    scenarioResults.net_benefit >= 0
                      ? 'simulation-result-card__value--positive'
                      : 'simulation-result-card__value--negative'
                  }`}
                >
                  {formatCurrency(scenarioResults.net_benefit)}
                </div>
              </div>

              <div className="simulation-result-card">
                <div className="simulation-result-card__label">Cash Flow Impact (Monthly)</div>
                <div
                  className={`simulation-result-card__value ${
                    scenarioResults.cash_flow_impact >= 0
                      ? 'simulation-result-card__value--positive'
                      : 'simulation-result-card__value--negative'
                  }`}
                >
                  {formatCurrency(scenarioResults.cash_flow_impact)}
                </div>
              </div>

              <div className="simulation-result-card">
                <div className="simulation-result-card__label">Risk Reduction</div>
                <div className="simulation-result-card__value">
                  {formatPercent(scenarioResults.risk_reduction)}
                </div>
              </div>

              {scenarioResults.capital_efficiency !== null && scenarioResults.capital_efficiency !== undefined && (
                <div className="simulation-result-card">
                  <div className="simulation-result-card__label">Capital Efficiency</div>
                  <div className="simulation-result-card__value">
                    {scenarioResults.capital_efficiency.toFixed(2)}x
                  </div>
                </div>
              )}
            </div>
          </div>
          )
        )}
      </div>
    </div>
  );
}
