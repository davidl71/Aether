import { useMemo } from 'react';
import type { BoxSpreadScenario } from '../types';
import { formatCurrency, formatPercent } from '../utils/formatters';
import { calculate3StdWidth, calculateBoxSpreadStrikes } from '../utils/volatility';

interface BoxSpreadCombinationsProps {
  scenarios: BoxSpreadScenario[];
  selectedStrike: number;
  selectedExpiration: string;
  underlyingPrice: number;
  daysToExpiry: number;
}

export function BoxSpreadCombinations({
  scenarios,
  selectedStrike,
  selectedExpiration,
  underlyingPrice,
  daysToExpiry
}: BoxSpreadCombinationsProps) {
  // Calculate 3std width and box spread strikes
  const boxSpreadStrikes = useMemo(() => {
    const width3Std = calculate3StdWidth(underlyingPrice, daysToExpiry);
    return calculateBoxSpreadStrikes(selectedStrike, width3Std);
  }, [selectedStrike, underlyingPrice, daysToExpiry]);

  // Filter scenarios that match the calculated strikes
  // Since scenarios have width, we need to match: width ≈ (upperStrike - lowerStrike)
  const matchingScenarios = useMemo(() => {
    const strikeWidth = boxSpreadStrikes.upperStrike - boxSpreadStrikes.lowerStrike;
    const tolerance = 0.5; // Allow 0.5 point tolerance

    return scenarios.filter((scenario) => {
      // Match by width (within tolerance)
      const widthMatch = Math.abs(scenario.width - strikeWidth) <= tolerance;

      // Match by expiration if available
      const expirationMatch =
        !scenario.expiration_date ||
        scenario.expiration_date === selectedExpiration ||
        scenario.days_to_expiry === daysToExpiry;

      return widthMatch && expirationMatch;
    });
  }, [scenarios, boxSpreadStrikes, selectedExpiration, daysToExpiry]);

  // Group by expiration date
  const groupedByExpiration = useMemo(() => {
    const groups = new Map<string, BoxSpreadScenario[]>();

    matchingScenarios.forEach((scenario) => {
      const key =
        scenario.expiration_date ||
        (scenario.days_to_expiry ? `~${scenario.days_to_expiry} days` : 'unknown');
      if (!groups.has(key)) {
        groups.set(key, []);
      }
      groups.get(key)!.push(scenario);
    });

    return Array.from(groups.entries()).map(([expiration, scenarios]) => ({
      expiration,
      scenarios: scenarios.sort((a, b) => a.width - b.width)
    }));
  }, [matchingScenarios]);

  if (matchingScenarios.length === 0) {
    return (
      <div className="panel">
        <div className="panel__header">
          <div>
            <h3>Box Spread Combinations</h3>
            <p>No box spread combinations found for selected strike</p>
          </div>
        </div>
        <div style={{ padding: '20px', textAlign: 'center', color: '#666' }}>
          <p>
            <strong>Selected Strike:</strong> {selectedStrike}
          </p>
          <p>
            <strong>Calculated Strikes (3std):</strong> {boxSpreadStrikes.lowerStrike} / {boxSpreadStrikes.upperStrike}
          </p>
          <p>
            <strong>Width:</strong> {boxSpreadStrikes.upperStrike - boxSpreadStrikes.lowerStrike} points
          </p>
          <p style={{ marginTop: '16px', fontSize: '0.9em' }}>
            No box spread scenarios match these parameters. Try selecting a different strike or check if box spread data
            is available for this symbol.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="panel">
      <div className="panel__header">
        <div>
          <h3>Box Spread Combinations</h3>
          <p>
            Strike {selectedStrike} as center | Lower: {boxSpreadStrikes.lowerStrike} | Upper:{' '}
            {boxSpreadStrikes.upperStrike} | Width: {boxSpreadStrikes.upperStrike - boxSpreadStrikes.lowerStrike} pts
            (3std)
          </p>
        </div>
      </div>

      {groupedByExpiration.map(({ expiration, scenarios }) => (
        <div key={expiration} style={{ marginBottom: '24px' }}>
          <h4 style={{ margin: '0 0 12px 0', fontSize: '1rem', color: '#cbd5f5' }}>
            Expiration: {expiration}
          </h4>
          <div className="table-wrapper">
            <table className="data-table" aria-label={`Box Spreads for ${expiration}`}>
              <thead>
                <tr>
                  <th>Width</th>
                  <th>Style</th>
                  <th>Buy Profit</th>
                  <th>Sell Profit</th>
                  <th>Buy Yield</th>
                  <th>Sell Yield</th>
                  <th>Avg Yield</th>
                  <th>Fill Prob</th>
                </tr>
              </thead>
              <tbody>
                {scenarios.map((scenario, idx) => {
                  const buyProfit = scenario.buy_profit ?? 0;
                  const sellProfit = scenario.sell_profit ?? 0;
                  const buyYield = scenario.buy_implied_rate ?? scenario.annualized_return;
                  const sellYield = scenario.sell_implied_rate ?? scenario.annualized_return;
                  const avgYield =
                    scenario.buy_implied_rate !== undefined && scenario.sell_implied_rate !== undefined
                      ? (scenario.buy_implied_rate + scenario.sell_implied_rate) / 2
                      : buyYield;

                  return (
                    <tr key={`${expiration}-${scenario.width}-${idx}`}>
                      <td>{scenario.width.toFixed(2)}</td>
                      <td>{scenario.option_style}</td>
                      <td
                        style={{
                          color: buyProfit > 0 ? '#4caf50' : '#f44336',
                          fontWeight: buyProfit > sellProfit ? 'bold' : 'normal'
                        }}
                      >
                        {buyProfit !== undefined ? formatCurrency(buyProfit) : '—'}
                      </td>
                      <td
                        style={{
                          color: sellProfit > 0 ? '#4caf50' : '#f44336',
                          fontWeight: sellProfit > buyProfit ? 'bold' : 'normal'
                        }}
                      >
                        {sellProfit !== undefined ? formatCurrency(sellProfit) : '—'}
                      </td>
                      <td>{formatPercent(buyYield)}</td>
                      <td>{formatPercent(sellYield)}</td>
                      <td
                        style={{
                          color: avgYield > 0 ? '#4caf50' : '#f44336',
                          fontWeight: 'bold'
                        }}
                      >
                        {formatPercent(avgYield)}
                      </td>
                      <td>{formatPercent(scenario.fill_probability)}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      ))}
    </div>
  );
}
