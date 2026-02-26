import { useState } from 'react';
import type { BoxSpreadScenario } from '../types';
import { useFinancingComparison, type ComparisonRow } from '../hooks/useFinancingComparison';
import { formatPercent } from '../utils/formatters';

interface FinancingComparisonTableProps {
  scenarios: BoxSpreadScenario[];
  symbol: string;
}

function WinnerBadge({ winner }: { winner: ComparisonRow['winner'] }) {
  if (!winner) return <span style={{ color: '#999' }}>N/A</span>;
  const styles: Record<string, { bg: string; text: string; label: string }> = {
    box_spread: { bg: 'rgba(76, 175, 80, 0.15)', text: '#4caf50', label: 'Box Spread' },
    treasury: { bg: 'rgba(33, 150, 243, 0.15)', text: '#2196f3', label: 'Treasury' },
    tie: { bg: 'rgba(158, 158, 158, 0.15)', text: '#9e9e9e', label: 'Tie' },
  };
  const s = styles[winner];
  return (
    <span style={{
      background: s.bg, color: s.text, padding: '2px 8px',
      borderRadius: '4px', fontSize: '0.8em', fontWeight: 600,
    }}>
      {s.label}
    </span>
  );
}

export function FinancingComparisonTable({ scenarios, symbol }: FinancingComparisonTableProps) {
  const [marginType, setMarginType] = useState<'reg_t' | 'portfolio'>('reg_t');
  const { data, isLoading } = useFinancingComparison(scenarios, {}, marginType);

  if (isLoading) {
    return (
      <div className="panel">
        <div className="panel__header"><h3>Financing Comparison</h3></div>
        <div style={{ padding: '20px', textAlign: 'center', color: '#9ca3af' }}>
          Loading Treasury benchmark data...
        </div>
      </div>
    );
  }

  if (!data || data.rows.length === 0) {
    return (
      <div className="panel">
        <div className="panel__header">
          <div>
            <h3>Financing Comparison</h3>
            <p>Box spread vs U.S. Treasury for {symbol}</p>
          </div>
        </div>
        <div style={{ padding: '20px', textAlign: 'center', color: '#666' }}>
          No European-style scenarios with expiration data available.
        </div>
      </div>
    );
  }

  return (
    <div className="panel">
      <div className="panel__header">
        <div>
          <h3>Financing Comparison</h3>
          <p>
            Box spread (Sec. 1256) vs U.S. Treasury for {symbol}
            {' · '}
            <span style={{ fontSize: '0.85em', color: '#9ca3af' }}>
              Fed: {(data.taxConfig.federalRate * 100).toFixed(0)}% · 1256 Blended: {(data.taxConfig.section1256Blended * 100).toFixed(1)}%
            </span>
          </p>
        </div>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
          <label style={{ fontSize: '0.85rem', color: '#9ca3af' }}>Margin:</label>
          <select
            value={marginType}
            onChange={e => setMarginType(e.target.value as 'reg_t' | 'portfolio')}
            style={{
              padding: '4px 8px', borderRadius: '6px',
              border: '1px solid rgba(148, 163, 184, 0.3)',
              background: 'rgba(30, 41, 59, 0.8)', color: '#e2e8f0',
              fontSize: '0.85rem',
            }}
          >
            <option value="reg_t">Reg-T (1x)</option>
            <option value="portfolio">Portfolio (4x)</option>
          </select>
        </div>
      </div>

      {/* Summary badges */}
      <div style={{
        display: 'flex', gap: '16px', padding: '12px 16px',
        background: 'rgba(148, 163, 184, 0.05)', borderBottom: '1px solid rgba(148, 163, 184, 0.1)',
      }}>
        <div style={{ textAlign: 'center' }}>
          <div style={{ fontSize: '1.4em', fontWeight: 700, color: '#4caf50' }}>{data.boxSpreadWins}</div>
          <div style={{ fontSize: '0.75em', color: '#9ca3af' }}>Box Spread Wins</div>
        </div>
        <div style={{ textAlign: 'center' }}>
          <div style={{ fontSize: '1.4em', fontWeight: 700, color: '#2196f3' }}>{data.treasuryWins}</div>
          <div style={{ fontSize: '0.75em', color: '#9ca3af' }}>Treasury Wins</div>
        </div>
        <div style={{ textAlign: 'center' }}>
          <div style={{ fontSize: '1.4em', fontWeight: 700, color: '#9e9e9e' }}>{data.ties}</div>
          <div style={{ fontSize: '0.75em', color: '#9ca3af' }}>Ties</div>
        </div>
      </div>

      <div className="table-wrapper">
        <table className="data-table" aria-label="Financing Comparison">
          <thead>
            <tr>
              <th>Tenor</th>
              <th>BS Gross</th>
              <th>BS After-Tax</th>
              <th>BS RoC</th>
              <th>Tsy Gross</th>
              <th>Tsy After-Tax</th>
              <th>Spread</th>
              <th>Leverage</th>
              <th>Winner</th>
            </tr>
          </thead>
          <tbody>
            {data.rows.map((row) => (
              <tr key={row.tenorDays}>
                <td>
                  <strong>{row.tenorLabel}</strong>
                  <span style={{ fontSize: '0.8em', color: '#9ca3af', marginLeft: '4px' }}>
                    ({row.tenorDays}d)
                  </span>
                </td>
                <td style={{ color: '#e2e8f0' }}>
                  {row.boxSpread ? formatPercent(row.boxSpread.grossRate) : '—'}
                </td>
                <td style={{ color: row.boxSpread ? '#4caf50' : '#666', fontWeight: 500 }}>
                  {row.boxSpread ? formatPercent(row.boxSpread.afterTaxRate) : '—'}
                </td>
                <td style={{
                  color: row.boxSpread && row.boxSpread.returnOnCapital > (row.boxSpread?.afterTaxRate ?? 0)
                    ? '#ff9800' : '#e2e8f0',
                  fontWeight: 500,
                }}
                  title="After-tax return on capital (adjusted for margin leverage)"
                >
                  {row.boxSpread ? formatPercent(row.boxSpread.returnOnCapital) : '—'}
                </td>
                <td style={{ color: '#e2e8f0' }}>
                  {row.treasury ? formatPercent(row.treasury.grossRate) : '—'}
                </td>
                <td style={{ color: row.treasury ? '#2196f3' : '#666', fontWeight: 500 }}>
                  {row.treasury ? formatPercent(row.treasury.afterTaxRate) : '—'}
                </td>
                <td style={{
                  color: row.spreadBpsAftertax !== null
                    ? (row.spreadBpsAftertax > 0 ? '#4caf50' : row.spreadBpsAftertax < 0 ? '#f44336' : '#666')
                    : '#666',
                  fontWeight: Math.abs(row.spreadBpsAftertax ?? 0) > 10 ? 600 : 400,
                }}>
                  {row.spreadBpsAftertax !== null
                    ? `${row.spreadBpsAftertax > 0 ? '+' : ''}${row.spreadBpsAftertax.toFixed(0)} bps`
                    : '—'}
                </td>
                <td style={{ color: '#e2e8f0', fontSize: '0.9em' }}>
                  {row.boxSpread ? `${row.boxSpread.marginLeverage.toFixed(1)}x` : '1.0x'}
                </td>
                <td><WinnerBadge winner={row.winner} /></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div style={{ padding: '8px 16px', fontSize: '0.75em', color: '#666' }}>
        <p>
          <strong>RoC</strong> = Return on Capital: after-tax rate adjusted for margin leverage.
          Under portfolio margin (4x), box spreads require ~25% of notional as margin,
          freeing 75% of capital for other uses.
        </p>
        <p>
          <strong>Section 1256:</strong> Qualified index options (SPX, XSP) receive 60% LTCG / 40% STCG treatment.
          Treasuries are state-tax exempt but federally taxed as ordinary income.
        </p>
      </div>
    </div>
  );
}
