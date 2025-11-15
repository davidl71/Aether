import { useState, useMemo } from 'react';
import type { BoxSpreadScenario } from '../types';
import { formatCurrency, formatPercent } from '../utils/formatters';

interface BoxSpreadTableProps {
  scenarios: BoxSpreadScenario[];
  asOf: string;
  underlying: string;
}

function BoxSpreadTable({ scenarios, asOf, underlying }: BoxSpreadTableProps) {
  // Default to hiding American-style options (show only European)
  const [showAmerican, setShowAmerican] = useState(false);

  // Filter scenarios by option style
  const filteredScenarios = useMemo(() => {
    if (showAmerican) {
      return scenarios; // Show all when toggled on
    }
    // Default: hide American-style options
    return scenarios.filter((scenario) => scenario.option_style === 'European');
  }, [scenarios, showAmerican]);

  const europeanCount = scenarios.filter((s) => s.option_style === 'European').length;
  const americanCount = scenarios.filter((s) => s.option_style === 'American').length;

  return (
    <section className="panel">
      <header className="panel__header">
        <div>
          <h2>Scenario Detail</h2>
          <p>{`Underlying ${underlying} · As of ${new Date(asOf).toLocaleString()}`}</p>
        </div>
        {americanCount > 0 && (
          <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={showAmerican}
              onChange={(e) => setShowAmerican(e.target.checked)}
            />
            <span>Show American-style options ({americanCount} hidden)</span>
          </label>
        )}
      </header>

      <div className="table-wrapper">
        <table className="data-table" aria-label="Box Spread Scenarios">
          <thead>
            <tr>
              <th>Width (pts)</th>
              <th>Style</th>
              <th>Buy Profit</th>
              <th>Sell Profit</th>
              <th>Disparity</th>
              <th>P-C Viol (bps)</th>
              <th>Mid</th>
              <th>APR</th>
              <th>Fill Prob</th>
            </tr>
          </thead>
          <tbody>
            {filteredScenarios.length === 0 ? (
              <tr>
                <td colSpan={9} style={{ textAlign: 'center', padding: '20px', color: '#666' }}>
                  {scenarios.length === 0
                    ? 'No scenarios available'
                    : `No ${showAmerican ? '' : 'European-style '}scenarios match the filters`}
                </td>
              </tr>
            ) : (
              filteredScenarios.map((scenario) => {
                const buyProfit = scenario.buy_profit ?? 0;
                const sellProfit = scenario.sell_profit ?? 0;
                const disparity = scenario.buy_sell_disparity ?? (buyProfit - sellProfit);
                const pcViolation = scenario.put_call_parity_violation ?? 0;
                const buyBetter = disparity > 0;

                return (
                  <tr key={scenario.width}>
                    <td>{scenario.width.toFixed(2)}</td>
                    <td>{scenario.option_style}</td>
                    <td style={{
                      color: buyProfit > 0 ? '#4caf50' : '#f44336',
                      fontWeight: buyBetter ? 'bold' : 'normal'
                    }}>
                      {buyProfit !== undefined ? formatCurrency(buyProfit) : '—'}
                    </td>
                    <td style={{
                      color: sellProfit > 0 ? '#4caf50' : '#f44336',
                      fontWeight: !buyBetter ? 'bold' : 'normal'
                    }}>
                      {sellProfit !== undefined ? formatCurrency(sellProfit) : '—'}
                    </td>
                    <td style={{
                      color: Math.abs(disparity) > 0.5 ? (buyBetter ? '#4caf50' : '#ff9800') : '#666',
                      fontWeight: Math.abs(disparity) > 1 ? 'bold' : 'normal'
                    }}>
                      {disparity !== undefined ? `${disparity > 0 ? '+' : ''}${disparity.toFixed(2)}` : '—'}
                    </td>
                    <td style={{
                      color: Math.abs(pcViolation) > 50 ? '#f44336' : '#666',
                      fontSize: '11px'
                    }}>
                      {pcViolation !== undefined ? `${pcViolation > 0 ? '+' : ''}${pcViolation.toFixed(1)}` : '—'}
                    </td>
                    <td>{formatCurrency(scenario.mid_price)}</td>
                    <td>{formatPercent(scenario.annualized_return)}</td>
                    <td>{formatPercent(scenario.fill_probability)}</td>
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>
      {europeanCount > 0 && (
        <div style={{ padding: '8px', fontSize: '12px', color: '#666' }}>
          Showing {filteredScenarios.length} of {scenarios.length} scenarios
          {!showAmerican && americanCount > 0 && ` (${americanCount} American-style hidden)`}
        </div>
      )}
    </section>
  );
}

export default BoxSpreadTable;
