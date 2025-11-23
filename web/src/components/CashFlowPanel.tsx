import { useMemo, useState } from 'react';
import type { PositionSnapshot } from '../types/snapshot';
import { formatCurrency, formatPercent } from '../utils/formatters';
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

interface CashFlowEvent {
  date: string; // ISO date string
  amount: number; // Positive for inflows, negative for outflows
  description: string;
  positionName: string;
  type: 'loan_payment' | 'maturity' | 'interest' | 'dividend' | 'other';
}

interface MonthlyCashFlow {
  month: string; // YYYY-MM
  inflows: number;
  outflows: number;
  net: number;
  events: CashFlowEvent[];
}

export function CashFlowPanel({
  positions,
  bankAccounts = [],
}: CashFlowPanelProps) {
  const [projectionMonths, setProjectionMonths] = useState(12);
  const [selectedCurrency, setSelectedCurrency] = useState<string>('USD');

  // Calculate cash flows from positions
  const cashFlows = useMemo(() => {
    const events: CashFlowEvent[] = [];
    const now = new Date();

    // Process positions
    for (const position of positions) {
      if (position.maturity_date) {
        const maturityDate = new Date(position.maturity_date);
        const monthsAhead = Math.ceil(
          (maturityDate.getTime() - now.getTime()) / (1000 * 60 * 60 * 24 * 30)
        );

        if (monthsAhead >= 0 && monthsAhead <= projectionMonths) {
          // Maturity cash flow
          const cashFlowAmount = position.cash_flow || position.candle.close || 0;
          events.push({
            date: position.maturity_date,
            amount: cashFlowAmount,
            description: `${position.instrument_type || 'Position'} maturity`,
            positionName: position.name,
            type: 'maturity',
          });

          // Monthly interest/rate payments for loans
          if (position.instrument_type === 'bank_loan' || position.instrument_type === 'pension_loan') {
            const rate = position.rate || 0;
            const principal = position.cash_flow || position.candle.close || 0;
            const monthlyPayment = (principal * rate) / 12;

            for (let month = 1; month <= Math.min(monthsAhead, projectionMonths); month++) {
              const paymentDate = new Date(now);
              paymentDate.setMonth(paymentDate.getMonth() + month);
              events.push({
                date: paymentDate.toISOString().split('T')[0],
                amount: -monthlyPayment, // Outflow
                description: `Monthly interest payment`,
                positionName: position.name,
                type: 'loan_payment',
              });
            }
          }
        }
      }

      // Current cash flow (if available)
      if (position.cash_flow !== undefined && position.cash_flow !== 0) {
        events.push({
          date: now.toISOString().split('T')[0],
          amount: position.cash_flow,
          description: `Current ${position.instrument_type || 'position'} cash flow`,
          positionName: position.name,
          type: 'other',
        });
      }
    }

    // Process bank accounts (as loans if debit_rate exists)
    for (const account of bankAccounts) {
      if (account.debit_rate && account.debit_rate > 0) {
        const principal = account.balance;
        const monthlyPayment = (principal * account.debit_rate) / 12;

        for (let month = 1; month <= projectionMonths; month++) {
          const paymentDate = new Date(now);
          paymentDate.setMonth(paymentDate.getMonth() + month);
          events.push({
            date: paymentDate.toISOString().split('T')[0],
            amount: -monthlyPayment, // Outflow
            description: `Monthly interest payment`,
            positionName: account.account_name,
            type: 'loan_payment',
          });
        }
      }
    }

    return events;
  }, [positions, bankAccounts, projectionMonths]);

  // Group cash flows by month
  const monthlyCashFlows = useMemo(() => {
    const monthly: Map<string, MonthlyCashFlow> = new Map();

    for (const event of cashFlows) {
      const month = event.date.substring(0, 7); // YYYY-MM
      if (!monthly.has(month)) {
        monthly.set(month, {
          month,
          inflows: 0,
          outflows: 0,
          net: 0,
          events: [],
        });
      }

      const monthlyFlow = monthly.get(month)!;
      monthlyFlow.events.push(event);

      if (event.amount > 0) {
        monthlyFlow.inflows += event.amount;
      } else {
        monthlyFlow.outflows += Math.abs(event.amount);
      }
      monthlyFlow.net = monthlyFlow.inflows - monthlyFlow.outflows;
    }

    // Sort by month
    return Array.from(monthly.values()).sort((a, b) => a.month.localeCompare(b.month));
  }, [cashFlows]);

  // Calculate totals
  const totals = useMemo(() => {
    let totalInflows = 0;
    let totalOutflows = 0;

    for (const monthly of monthlyCashFlows) {
      totalInflows += monthly.inflows;
      totalOutflows += monthly.outflows;
    }

    return {
      inflows: totalInflows,
      outflows: totalOutflows,
      net: totalInflows - totalOutflows,
    };
  }, [monthlyCashFlows]);

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
            >
              <option value={6}>6 months</option>
              <option value={12}>12 months</option>
              <option value={24}>24 months</option>
              <option value={36}>36 months</option>
            </select>
          </label>
        </div>
      </div>

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
    </div>
  );
}
