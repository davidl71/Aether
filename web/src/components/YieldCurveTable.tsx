import { useMemo } from 'react';
import type { BoxSpreadScenario, TreasuryBenchmark } from '../types';
import { formatPercent } from '../utils/formatters';
import { useTreasuryYields, findClosestBenchmark } from '../hooks/useTreasuryYields';

interface YieldCurveTableProps {
  scenarios: BoxSpreadScenario[];
  symbol: string;
}

interface YieldCurveRow {
  expirationDate: string;
  daysToExpiry: number;
  strikeWidths: Array<{
    width: number;
    yield: number;
    buyYield?: number;
    sellYield?: number;
  }>;
  avgYield: number;
  benchmark: TreasuryBenchmark | null;
  avgSpread: number; // Spread in basis points (box spread yield - benchmark yield)
}

export function YieldCurveTable({ scenarios, symbol }: YieldCurveTableProps) {
  // Fetch Treasury yields
  const { data: treasuryData, isLoading: treasuryLoading } = useTreasuryYields();

  // Group scenarios by expiration date and calculate yields
  const yieldCurveData = useMemo<YieldCurveRow[]>(() => {
    // Filter to European-style only (more reliable for yield curves)
    const europeanScenarios = scenarios.filter((s) => s.option_style === 'European');

    if (europeanScenarios.length === 0) {
      return [];
    }

    // Group by expiration date
    const groupedByExpiry = new Map<string, BoxSpreadScenario[]>();

    for (const scenario of europeanScenarios) {
      // Use expiration_date if available, otherwise create a placeholder key
      const expiryKey = scenario.expiration_date || scenario.days_to_expiry?.toString() || 'unknown';

      if (!groupedByExpiry.has(expiryKey)) {
        groupedByExpiry.set(expiryKey, []);
      }
      groupedByExpiry.get(expiryKey)!.push(scenario);
    }

    // Convert to array of YieldCurveRow
    const rows: YieldCurveRow[] = [];

    for (const [expiryKey, expiryScenarios] of groupedByExpiry.entries()) {
      // Sort scenarios by strike width
      const sortedScenarios = [...expiryScenarios].sort((a, b) => a.width - b.width);

      // Calculate yields for each strike width
      const strikeWidths = sortedScenarios.map((scenario) => {
        // Prefer buy_implied_rate or sell_implied_rate, fallback to annualized_return
        const buyYield = scenario.buy_implied_rate ?? scenario.annualized_return;
        const sellYield = scenario.sell_implied_rate ?? scenario.annualized_return;
        // Use average of buy/sell if both available, otherwise use what's available
        const yieldValue =
          scenario.buy_implied_rate !== undefined && scenario.sell_implied_rate !== undefined
            ? (scenario.buy_implied_rate + scenario.sell_implied_rate) / 2
            : scenario.buy_implied_rate ?? scenario.sell_implied_rate ?? scenario.annualized_return;

        return {
          width: scenario.width,
          yield: yieldValue,
          buyYield: scenario.buy_implied_rate,
          sellYield: scenario.sell_implied_rate
        };
      });

      // Calculate average yield for this expiration
      const avgYield =
        strikeWidths.reduce((sum, sw) => sum + sw.yield, 0) / strikeWidths.length;

      // Parse expiration date or use days_to_expiry
      let expirationDate = expiryKey;
      let daysToExpiry = 0;

      if (expiryKey !== 'unknown') {
        // Try to parse as YYYYMMDD
        if (/^\d{8}$/.test(expiryKey)) {
          const year = expiryKey.substring(0, 4);
          const month = expiryKey.substring(4, 6);
          const day = expiryKey.substring(6, 8);
          expirationDate = `${year}-${month}-${day}`;
          // Calculate days to expiry
          const expiryDate = new Date(parseInt(year), parseInt(month) - 1, parseInt(day));
          const today = new Date();
          daysToExpiry = Math.ceil((expiryDate.getTime() - today.getTime()) / (1000 * 60 * 60 * 24));
        } else if (!isNaN(Number(expiryKey))) {
          // It's a days_to_expiry number
          daysToExpiry = Number(expiryKey);
          expirationDate = `~${daysToExpiry} days`;
        }
      } else {
        // Try to get days_to_expiry from first scenario
        daysToExpiry = expiryScenarios[0]?.days_to_expiry ?? 0;
        expirationDate = daysToExpiry > 0 ? `~${daysToExpiry} days` : 'Unknown';
      }

      // Find closest Treasury benchmark
      const benchmark = treasuryData
        ? findClosestBenchmark(daysToExpiry, treasuryData.benchmarks)
        : null;

      // Calculate average spread in basis points (1 basis point = 0.01%)
      let avgSpread = 0;
      if (benchmark && avgYield > 0) {
        avgSpread = (avgYield - benchmark.yield) * 100; // Convert percentage difference to basis points
      }

      rows.push({
        expirationDate,
        daysToExpiry,
        strikeWidths,
        avgYield,
        benchmark,
        avgSpread
      });
    }

    // Sort by days to expiry (ascending)
    rows.sort((a, b) => a.daysToExpiry - b.daysToExpiry);

    return rows;
  }, [scenarios, treasuryData]);

  if (yieldCurveData.length === 0) {
    return (
      <div className="panel">
        <div className="panel__header">
          <div>
            <h3>Yield Curve</h3>
            <p>No yield curve data available for {symbol}</p>
          </div>
        </div>
        <div style={{ padding: '20px', textAlign: 'center', color: '#666' }}>
          {scenarios.length === 0
            ? 'No box spread scenarios available'
            : 'No expiration date data available in scenarios. Backend needs to include expiration_date or days_to_expiry fields.'}
        </div>
      </div>
    );
  }

  // Get all unique strike widths across all expirations for column headers
  const allStrikeWidths = useMemo(() => {
    const widths = new Set<number>();
    for (const row of yieldCurveData) {
      for (const sw of row.strikeWidths) {
        widths.add(sw.width);
      }
    }
    return Array.from(widths).sort((a, b) => a - b);
  }, [yieldCurveData]);

  return (
    <div className="panel">
      <div className="panel__header">
        <div>
          <h3>Yield Curve by Expiration</h3>
          <p>Implied interest rates from box spreads for {symbol}</p>
        </div>
      </div>
      <div className="table-wrapper">
        <table className="data-table" aria-label={`Yield Curve for ${symbol}`}>
          <thead>
            <tr>
              <th>Expiration</th>
              <th>Days</th>
              {allStrikeWidths.map((width) => (
                <th key={width} title={`Strike Width: ${width}`}>
                  {width}pt
                </th>
              ))}
              <th>Avg Yield</th>
              {treasuryData && treasuryData.benchmarks.length > 0 && (
                <>
                  <th>Benchmark</th>
                  <th>Spread (bps)</th>
                </>
              )}
            </tr>
          </thead>
          <tbody>
            {yieldCurveData.map((row, idx) => (
              <tr key={row.expirationDate + idx}>
                <td>
                  <strong>{row.expirationDate}</strong>
                </td>
                <td>{row.daysToExpiry > 0 ? row.daysToExpiry : '—'}</td>
                {allStrikeWidths.map((width) => {
                  const strikeData = row.strikeWidths.find((sw) => sw.width === width);
                  if (!strikeData) {
                    return <td key={width}>—</td>;
                  }

                  // Show buy/sell yields if both available, otherwise show single yield
                  const hasBothYields = strikeData.buyYield !== undefined && strikeData.sellYield !== undefined;

                  return (
                    <td
                      key={width}
                      style={{
                        color: strikeData.yield > 0 ? '#4caf50' : '#f44336',
                        fontWeight: '500'
                      }}
                      title={
                        hasBothYields
                          ? `Buy: ${formatPercent(strikeData.buyYield!)}, Sell: ${formatPercent(strikeData.sellYield!)}`
                          : undefined
                      }
                    >
                      {formatPercent(strikeData.yield)}
                      {hasBothYields && (
                        <span style={{ fontSize: '0.75em', color: '#666', marginLeft: '4px' }}>
                          (B:{formatPercent(strikeData.buyYield!)} / S:{formatPercent(strikeData.sellYield!)})
                        </span>
                      )}
                    </td>
                  );
                })}
                <td
                  style={{
                    color: row.avgYield > 0 ? '#4caf50' : '#f44336',
                    fontWeight: 'bold'
                  }}
                >
                  {formatPercent(row.avgYield)}
                </td>
                {treasuryData && treasuryData.benchmarks.length > 0 && (
                  <>
                    <td
                      style={{
                        color: '#666',
                        fontSize: '0.9em'
                      }}
                      title={row.benchmark ? `${row.benchmark.type} ${row.benchmark.maturity}` : 'No benchmark available'}
                    >
                      {row.benchmark ? (
                        <>
                          <strong>{formatPercent(row.benchmark.yield)}</strong>
                          <span style={{ fontSize: '0.85em', marginLeft: '4px', color: '#999' }}>
                            ({row.benchmark.maturity} {row.benchmark.type})
                          </span>
                        </>
                      ) : (
                        <span style={{ color: '#999' }}>N/A</span>
                      )}
                    </td>
                    <td
                      style={{
                        color:
                          row.avgSpread > 0
                            ? '#4caf50' // Positive spread = box spread better
                            : row.avgSpread < 0
                              ? '#f44336' // Negative spread = benchmark better
                              : '#666',
                        fontWeight: Math.abs(row.avgSpread) > 10 ? 'bold' : 'normal'
                      }}
                      title={
                        row.benchmark
                          ? `Box spread yield ${row.avgSpread > 0 ? 'higher' : 'lower'} than ${row.benchmark.type} ${row.benchmark.maturity} by ${Math.abs(row.avgSpread).toFixed(1)} basis points`
                          : undefined
                      }
                    >
                      {row.benchmark ? (
                        <>
                          {row.avgSpread > 0 ? '+' : ''}
                          {row.avgSpread.toFixed(1)} bps
                        </>
                      ) : (
                        <span style={{ color: '#999' }}>—</span>
                      )}
                    </td>
                  </>
                )}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div style={{ padding: '8px', fontSize: '12px', color: '#666' }}>
        <p>
          <strong>Note:</strong> Yields are calculated from box spread implied rates. When both buy and sell yields are
          available, the average is shown with individual values in parentheses (B=Buy, S=Sell).
        </p>
        {treasuryData && treasuryData.benchmarks.length > 0 && (
          <p style={{ marginTop: '4px' }}>
            <strong>Benchmark:</strong> Treasury yields matched by closest maturity. Spread shows box spread yield minus
            benchmark yield in basis points (100 bps = 1%). Positive spread indicates box spread offers higher yield.
          </p>
        )}
        {treasuryLoading && (
          <p style={{ marginTop: '4px', fontStyle: 'italic' }}>
            Loading Treasury benchmark data...
          </p>
        )}
        {!treasuryLoading && (!treasuryData || treasuryData.benchmarks.length === 0) && (
          <p style={{ marginTop: '4px', fontStyle: 'italic', color: '#999' }}>
            Treasury benchmark data unavailable. Benchmarks require backend API integration or FRED API access.
          </p>
        )}
        {yieldCurveData.some((row) => row.strikeWidths.some((sw) => sw.buyYield !== undefined && sw.sellYield !== undefined)) && (
          <p style={{ marginTop: '4px' }}>
            <em>Hover over cells to see buy/sell breakdown when available.</em>
          </p>
        )}
      </div>
    </div>
  );
}
