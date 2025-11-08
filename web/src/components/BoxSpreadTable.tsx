import type { BoxSpreadScenario } from '../types';
import { formatCurrency, formatPercent } from '../utils/formatters';

interface BoxSpreadTableProps {
  scenarios: BoxSpreadScenario[];
  asOf: string;
  underlying: string;
}

function BoxSpreadTable({ scenarios, asOf, underlying }: BoxSpreadTableProps) {
  return (
    <section className="panel">
      <header className="panel__header">
        <div>
          <h2>Scenario Detail</h2>
          <p>{`Underlying ${underlying} · As of ${new Date(asOf).toLocaleString()}`}</p>
        </div>
      </header>

      <div className="table-wrapper">
        <table className="data-table" aria-label="Box Spread Scenarios">
          <thead>
            <tr>
              <th>Width (pts)</th>
              <th>Put Bid</th>
              <th>Call Ask</th>
              <th>Synthetic Bid</th>
              <th>Synthetic Ask</th>
              <th>Mid</th>
              <th>APR</th>
              <th>Fill Probability</th>
            </tr>
          </thead>
          <tbody>
            {scenarios.map((scenario) => (
              <tr key={scenario.width}>
                <td>{scenario.width.toFixed(2)}</td>
                <td>{formatCurrency(scenario.put_bid)}</td>
                <td>{formatCurrency(scenario.call_ask)}</td>
                <td>{formatCurrency(scenario.synthetic_bid)}</td>
                <td>{formatCurrency(scenario.synthetic_ask)}</td>
                <td>{formatCurrency(scenario.mid_price)}</td>
                <td>{formatPercent(scenario.annualized_return)}</td>
                <td>{formatPercent(scenario.fill_probability)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

export default BoxSpreadTable;
