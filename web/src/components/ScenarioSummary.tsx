import type { BoxSpreadSummary } from '../types';
import { formatPercent } from '../utils/formatters';

interface ScenarioSummaryProps {
  summary: BoxSpreadSummary;
}

function ScenarioSummary({ summary }: ScenarioSummaryProps) {
  return (
    <section className="panel">
      <header className="panel__header">
        <div>
          <h2>Overview</h2>
          <p>Snapshot of the current synthetic box spread grid.</p>
        </div>
      </header>

      <div className="summary-grid">
        <div className="summary-card">
          <span className="summary-card__label">Scenarios</span>
          <strong className="summary-card__value">{summary.totalScenarios}</strong>
        </div>
        <div className="summary-card">
          <span className="summary-card__label">Average APR</span>
          <strong className="summary-card__value">{formatPercent(summary.avgApr)}</strong>
        </div>
        <div className="summary-card">
          <span className="summary-card__label">Probable Fills</span>
          <strong className="summary-card__value">{summary.probableCount}</strong>
        </div>
        <div className="summary-card">
          <span className="summary-card__label">Top APR Scenario</span>
          <strong className="summary-card__value">
            {summary.maxAprScenario
              ? `${summary.maxAprScenario.width.toFixed(2)} pts · ${formatPercent(summary.maxAprScenario.annualized_return)}`
              : 'Pending'}
          </strong>
        </div>
      </div>
    </section>
  );
}

export default ScenarioSummary;
