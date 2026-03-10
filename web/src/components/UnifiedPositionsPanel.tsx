import { useEffect, useMemo, useState } from 'react';
import type { PositionSnapshot, InstrumentType } from '../types/snapshot';
import { formatCurrency, formatPercent } from '../utils/formatters';
import { Sparkline } from './Sparkline';
import type { BankAccount } from '../types/banking';
import { fetchUnifiedPositions } from '../api/calculations';

interface UnifiedPositionsPanelProps {
  positions: PositionSnapshot[];
  bankAccounts?: BankAccount[];
  onSelectPosition: (position: PositionSnapshot) => void;
}

type GroupedPositions = {
  [key in InstrumentType]: PositionSnapshot[];
};

const INSTRUMENT_TYPE_LABELS: Record<InstrumentType, string> = {
  box_spread: 'Box Spreads',
  bank_loan: 'Bank Loans',
  pension_loan: 'Pension Loans',
  bond: 'Bonds',
  t_bill: 'T-Bills',
  futures: 'Futures',
  other: 'Other',
};

const INSTRUMENT_TYPE_ORDER: InstrumentType[] = [
  'box_spread',
  'bank_loan',
  'pension_loan',
  'bond',
  't_bill',
  'futures',
  'other',
];

export function UnifiedPositionsPanel({
  positions,
  bankAccounts = [],
  onSelectPosition,
}: UnifiedPositionsPanelProps) {
  const [expandedGroups, setExpandedGroups] = useState<Set<InstrumentType>>(
    new Set(INSTRUMENT_TYPE_ORDER) // All groups expanded by default
  );
  const [filterType, setFilterType] = useState<InstrumentType | 'all'>('all');

  const [allPositions, setAllPositions] = useState<PositionSnapshot[]>(positions);
  const [loadingUnifiedPositions, setLoadingUnifiedPositions] = useState(false);

  useEffect(() => {
    let cancelled = false;

    async function loadUnifiedPositions() {
      setLoadingUnifiedPositions(true);
      try {
        const result = await fetchUnifiedPositions(positions, bankAccounts);
        if (!cancelled) {
          setAllPositions(result.positions);
        }
      } catch (error) {
        if (!cancelled) {
          console.error('Failed to fetch unified positions:', error);
          setAllPositions(positions);
        }
      } finally {
        if (!cancelled) {
          setLoadingUnifiedPositions(false);
        }
      }
    }

    void loadUnifiedPositions();
    return () => {
      cancelled = true;
    };
  }, [positions, bankAccounts]);

  // Group positions by instrument type
  const groupedPositions = useMemo(() => {
    const grouped: GroupedPositions = {
      box_spread: [],
      bank_loan: [],
      pension_loan: [],
      bond: [],
      t_bill: [],
      futures: [],
      other: [],
    };

    allPositions.forEach((position) => {
      const type = position.instrument_type || 'other';
      if (type in grouped) {
        grouped[type as InstrumentType].push(position);
      } else {
        grouped.other.push(position);
      }
    });

    return grouped;
  }, [allPositions]);

  // Filter positions based on selected filter
  const filteredGroups = useMemo(() => {
    if (filterType === 'all') {
      return groupedPositions;
    }
    // Create empty groups structure
    const emptyGroups: GroupedPositions = {
      box_spread: [],
      bank_loan: [],
      pension_loan: [],
      bond: [],
      t_bill: [],
      futures: [],
      other: [],
    };
    return {
      ...emptyGroups,
      [filterType]: groupedPositions[filterType],
    };
  }, [groupedPositions, filterType]);

  // Calculate totals
  const totals = useMemo(() => {
    const totalsByType: Record<InstrumentType, { count: number; totalValue: number }> = {
      box_spread: { count: 0, totalValue: 0 },
      bank_loan: { count: 0, totalValue: 0 },
      pension_loan: { count: 0, totalValue: 0 },
      bond: { count: 0, totalValue: 0 },
      t_bill: { count: 0, totalValue: 0 },
      futures: { count: 0, totalValue: 0 },
      other: { count: 0, totalValue: 0 },
    };

    allPositions.forEach((position) => {
      const type = position.instrument_type || 'other';
      if (type in totalsByType) {
        totalsByType[type as InstrumentType].count++;
        totalsByType[type as InstrumentType].totalValue +=
          position.cash_flow || position.candle.close || 0;
      }
    });

    return totalsByType;
  }, [allPositions]);

  const toggleGroup = (type: InstrumentType) => {
    setExpandedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(type)) {
        next.delete(type);
      } else {
        next.add(type);
      }
      return next;
    });
  };

  const formatMaturityDate = (dateStr?: string): string => {
    if (!dateStr) return '—';
    try {
      const date = new Date(dateStr);
      return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      });
    } catch {
      return dateStr;
    }
  };

  const totalPositions = allPositions.length;

  if (totalPositions === 0) {
    return (
      <div className="panel panel--fill">
        <div className="panel__header">
          <div>
            <h2>Unified Positions</h2>
            <p>All positions across all instrument types</p>
          </div>
        </div>
        <div className="panel__body">
          <p className="text-muted">No positions found</p>
        </div>
      </div>
    );
  }

  return (
    <div className="panel panel--fill">
      <div className="panel__header">
        <div>
          <h2>Unified Positions</h2>
          <p>All positions across all instrument types ({totalPositions} total)</p>
          {loadingUnifiedPositions && <p className="text-muted">Refreshing unified positions…</p>}
        </div>
        <div className="panel__filters">
          <select
            value={filterType}
            onChange={(e) => setFilterType(e.target.value as InstrumentType | 'all')}
            className="filter-select"
          >
            <option value="all">All Types</option>
            {INSTRUMENT_TYPE_ORDER.map((type) => (
              <option key={type} value={type}>
                {INSTRUMENT_TYPE_LABELS[type]} ({totals[type].count})
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="unified-positions">
        {INSTRUMENT_TYPE_ORDER.map((type) => {
          const groupPositions = filteredGroups[type];
          const isExpanded = expandedGroups.has(type);
          const groupTotal = totals[type];

          if (groupPositions.length === 0) {
            return null;
          }

          return (
            <div key={type} className="position-group">
              <div
                className="position-group__header"
                onClick={() => toggleGroup(type)}
                style={{ cursor: 'pointer' }}
              >
                <div className="position-group__title">
                  <span className="position-group__toggle">
                    {isExpanded ? '▼' : '▶'}
                  </span>
                  <h3>{INSTRUMENT_TYPE_LABELS[type]}</h3>
                  <span className="position-group__count">
                    ({groupPositions.length})
                  </span>
                </div>
                <div className="position-group__summary">
                  <span>Total Value: {formatCurrency(groupTotal.totalValue)}</span>
                </div>
              </div>

              {isExpanded && (
                <div className="position-group__body">
                  <table className="data-table">
                    <thead>
                      <tr>
                        <th>Name</th>
                        <th>Quantity</th>
                        <th>Rate/ROI</th>
                        <th>Maturity</th>
                        <th>Cash Flow</th>
                        <th>Currency</th>
                        {type === 'box_spread' && (
                          <>
                            <th>Vega</th>
                            <th>Theta</th>
                            <th>Fair Δ</th>
                            <th>Range</th>
                          </>
                        )}
                      </tr>
                    </thead>
                    <tbody>
                      {groupPositions.map((position, idx) => (
                        <tr
                          key={`${type}-${position.name}-${idx}`}
                          onClick={() => onSelectPosition(position)}
                          style={{ cursor: 'pointer' }}
                        >
                          <td>{position.name}</td>
                          <td>{position.quantity}</td>
                          <td className="data-table__positive">
                            {position.rate !== undefined
                              ? formatPercent(position.rate)
                              : formatPercent(position.roi)}
                          </td>
                          <td>{formatMaturityDate(position.maturity_date)}</td>
                          <td>
                            {position.cash_flow !== undefined
                              ? formatCurrency(position.cash_flow)
                              : '—'}
                          </td>
                          <td>{position.currency || 'USD'}</td>
                          {type === 'box_spread' && (
                            <>
                              <td>{position.vega.toFixed(3)}</td>
                              <td>{position.theta.toFixed(3)}</td>
                              <td>{position.fair_diff.toFixed(3)}</td>
                              <td>
                                <Sparkline candle={position.candle} />
                              </td>
                            </>
                          )}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
