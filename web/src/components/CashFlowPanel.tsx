import { useMemo, useState, useEffect } from 'react';
import type { PositionSnapshot } from '../types/snapshot';
import { formatCurrency, formatPercent } from '../utils/formatters';
import {
  calculateCashFlowTimeline,
  type CashFlowTimelineResponse,
  type MonthlyCashFlow,
} from '../api/calculations';
import '../styles/cash-flow.css';

interface CashFlowPanelProps {
  positions: PositionSnapshot[];
  bankAccounts?: Array<{
    account_path: string;
    account_name: string;
    bank_name: string | null;
    balance: number;
    currency: string;
    credit_rate: number | null;
    debit_rate: number | null;
  }>;
}

// Types imported from calculations API

export function CashFlowPanel({
  positions,
  bankAccounts = [],
}: CashFlowPanelProps) {
  const [projectionMonths, setProjectionMonths] = useState(12);
  const [selectedCurrency, setSelectedCurrency] = useState<string>('USD');
  const [timelineData, setTimelineData] = useState<CashFlowTimelineResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Fetch cash flow timeline from API
  useEffect(() => {
    let cancelled = false;

    async function fetchTimeline() {
      setLoading(true);
      setError(null);

      try {
        const result = await calculateCashFlowTimeline(
          positions,
          bankAccounts,
          projectionMonths
        );
        if (!cancelled) {
          setTimelineData(result);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : 'Failed to calculate cash flow timeline');
          console.error('Cash flow calculation error:', err);
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    fetchTimeline();

    return () => {
      cancelled = true;
    };
  }, [positions, bankAccounts, projectionMonths]);

  // Convert monthly flows to array and sort
  const monthlyCashFlows = useMemo(() => {
    if (!timelineData) return [];

    return Object.values(timelineData.monthly_flows).sort((a, b) =>
      a.month.localeCompare(b.month)
    );
  }, [timelineData]);

  // Calculate totals
  const totals = useMemo(() => {
    if (!timelineData) {
      return { inflows: 0, outflows: 0, net: 0 };
    }

    return {
      inflows: timelineData.total_inflows,
      outflows: timelineData.total_outflows,
      net: timelineData.net_cash_flow,
    };
  }, [timelineData]);

  return (
    <div className="cash-flow-panel">
      <div className="cash-flow-panel__header">
        <h2>Cash Flow Projection</h2>
        <div className="cash-flow-panel__controls">
          <label>
            Projection Period:
            <select
              value={projectionMonths}
              onChange={(e) => setProjectionMonths(Number(e.target.value))}
              disabled={loading}
            >
              <option value={6}>6 months</option>
              <option value={12}>12 months</option>
              <option value={24}>24 months</option>
              <option value={36}>36 months</option>
            </select>
          </label>
        </div>
      </div>

      {error && (
        <div className="cash-flow-panel__error" style={{
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

      {loading && (
        <div style={{ textAlign: 'center', padding: '20px', color: '#9ca3af' }}>
          Calculating cash flow timeline...
        </div>
      )}

      <div className="cash-flow-panel__summary">
        <div className="cash-flow-summary-card">
          <div className="cash-flow-summary-card__label">Total Inflows</div>
          <div className="cash-flow-summary-card__value cash-flow-summary-card__value--positive">
            {formatCurrency(totals.inflows)}
          </div>
        </div>
        <div className="cash-flow-summary-card">
          <div className="cash-flow-summary-card__label">Total Outflows</div>
          <div className="cash-flow-summary-card__value cash-flow-summary-card__value--negative">
            {formatCurrency(totals.outflows)}
          </div>
        </div>
        <div className="cash-flow-summary-card">
          <div className="cash-flow-summary-card__label">Net Cash Flow</div>
          <div
            className={`cash-flow-summary-card__value ${
              totals.net >= 0
                ? 'cash-flow-summary-card__value--positive'
                : 'cash-flow-summary-card__value--negative'
            }`}
          >
            {formatCurrency(totals.net)}
          </div>
        </div>
      </div>

      {!loading && !error && (
        <div className="cash-flow-panel__table">
          <table className="cash-flow-table">
            <thead>
              <tr>
                <th>Month</th>
                <th>Inflows</th>
                <th>Outflows</th>
                <th>Net</th>
                <th>Events</th>
              </tr>
            </thead>
            <tbody>
              {monthlyCashFlows.length === 0 ? (
                <tr>
                  <td colSpan={5} style={{ textAlign: 'center', padding: '20px', color: '#9ca3af' }}>
                    No cash flow events in projection period
                  </td>
                </tr>
              ) : (
                monthlyCashFlows.map((monthly) => (
                  <tr key={monthly.month}>
                    <td>{new Date(monthly.month + '-01').toLocaleDateString('en-US', { month: 'short', year: 'numeric' })}</td>
                    <td className="cash-flow-table__inflow">
                      {monthly.inflows > 0 ? formatCurrency(monthly.inflows) : '—'}
                    </td>
                    <td className="cash-flow-table__outflow">
                      {monthly.outflows > 0 ? formatCurrency(monthly.outflows) : '—'}
                    </td>
                    <td
                      className={
                        monthly.net >= 0
                          ? 'cash-flow-table__net--positive'
                          : 'cash-flow-table__net--negative'
                      }
                    >
                      {formatCurrency(monthly.net)}
                    </td>
                    <td>
                      <details>
                        <summary>{monthly.events.length} event{monthly.events.length !== 1 ? 's' : ''}</summary>
                        <ul className="cash-flow-events-list">
                          {monthly.events.map((event, idx) => (
                            <li key={idx} className={`cash-flow-event cash-flow-event--${event.type}`}>
                              <span className="cash-flow-event__date">
                                {new Date(event.date).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
                              </span>
                              <span className="cash-flow-event__description">{event.description}</span>
                              <span className="cash-flow-event__position">{event.position_name}</span>
                              <span
                                className={
                                  event.amount >= 0
                                    ? 'cash-flow-event__amount--positive'
                                    : 'cash-flow-event__amount--negative'
                                }
                              >
                                {formatCurrency(event.amount)}
                              </span>
                            </li>
                          ))}
                        </ul>
                      </details>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
