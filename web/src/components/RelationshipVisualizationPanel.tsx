import { useMemo } from 'react';
import type { PositionSnapshot } from '../types/snapshot';
import { formatCurrency } from '../utils/formatters';
import '../styles/relationship-visualization.css';

interface RelationshipVisualizationPanelProps {
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

interface Relationship {
  from: string;
  to: string;
  type: 'collateral' | 'margin' | 'financing' | 'investment';
  description: string;
  value: number;
}

export function RelationshipVisualizationPanel({
  positions,
  bankAccounts = [],
}: RelationshipVisualizationPanelProps) {
  // Build relationship graph from positions
  const relationships = useMemo(() => {
    const rels: Relationship[] = [];

    // Find loans
    const loans = positions.filter(
      p => p.instrument_type === 'bank_loan' || p.instrument_type === 'pension_loan'
    );
    const bankLoans = bankAccounts.filter(a => a.debit_rate && a.debit_rate > 0);

    // Find box spreads
    const boxSpreads = positions.filter(p => p.instrument_type === 'box_spread');

    // Find bonds/funds
    const bonds = positions.filter(
      p => p.instrument_type === 'bond' || p.instrument_type === 't_bill'
    );

    // Relationship 1: Loan → Box Spread (margin)
    for (const loan of loans) {
      for (const boxSpread of boxSpreads) {
        rels.push({
          from: loan.name,
          to: boxSpread.name,
          type: 'margin',
          description: 'Loan used as margin for box spread',
          value: loan.cash_flow || loan.candle.close || 0,
        });
      }
    }

    // Relationship 2: Loan → Investment (fund/bond)
    for (const loan of loans) {
      for (const bond of bonds) {
        rels.push({
          from: loan.name,
          to: bond.name,
          type: 'investment',
          description: 'Loan proceeds invested in bond',
          value: loan.cash_flow || loan.candle.close || 0,
        });
      }
    }

    // Relationship 3: Investment → Collateral (for cheaper loan)
    for (const bond of bonds) {
      for (const loan of loans) {
        if ((bond.rate || 0) > (loan.rate || 0)) {
          rels.push({
            from: bond.name,
            to: loan.name,
            type: 'collateral',
            description: 'Bond used as collateral for loan',
            value: bond.collateral_value || bond.cash_flow || bond.candle.close || 0,
          });
        }
      }
    }

    // Relationship 4: Box Spread → Financing (synthetic financing)
    for (const boxSpread of boxSpreads) {
      rels.push({
        from: boxSpread.name,
        to: 'Synthetic Financing',
        type: 'financing',
        description: 'Box spread provides synthetic financing',
        value: boxSpread.cash_flow || boxSpread.candle.close || 0,
      });
    }

    return rels;
  }, [positions, bankAccounts]);

  // Get unique nodes
  const nodes = useMemo(() => {
    const nodeSet = new Set<string>();
    relationships.forEach(rel => {
      nodeSet.add(rel.from);
      nodeSet.add(rel.to);
    });
    positions.forEach(pos => nodeSet.add(pos.name));
    bankAccounts.forEach(acc => nodeSet.add(acc.account_name));
    return Array.from(nodeSet);
  }, [relationships, positions, bankAccounts]);

  // Group relationships by type
  const relationshipsByType = useMemo(() => {
    const grouped: Record<string, Relationship[]> = {
      collateral: [],
      margin: [],
      financing: [],
      investment: [],
    };
    relationships.forEach(rel => {
      grouped[rel.type].push(rel);
    });
    return grouped;
  }, [relationships]);

  return (
    <div className="relationship-visualization-panel">
      <div className="relationship-visualization-panel__header">
        <h2>Multi-Instrument Relationships</h2>
        <p className="relationship-visualization-panel__subtitle">
          Visualize relationships between loans, box spreads, bonds, and other instruments
        </p>
      </div>

      <div className="relationship-visualization-panel__content">
        <div className="relationship-visualization-panel__summary">
          <div className="relationship-summary-card">
            <div className="relationship-summary-card__label">Total Relationships</div>
            <div className="relationship-summary-card__value">{relationships.length}</div>
          </div>
          <div className="relationship-summary-card">
            <div className="relationship-summary-card__label">Instruments</div>
            <div className="relationship-summary-card__value">{nodes.length}</div>
          </div>
        </div>

        <div className="relationship-visualization-panel__relationships">
          {Object.entries(relationshipsByType).map(([type, rels]) => {
            if (rels.length === 0) return null;

            return (
              <div key={type} className="relationship-group">
                <h3 className="relationship-group__title">
                  {type.charAt(0).toUpperCase() + type.slice(1)} Relationships ({rels.length})
                </h3>
                <div className="relationship-list">
                  {rels.map((rel, idx) => (
                    <div key={idx} className="relationship-item">
                      <div className="relationship-item__from">{rel.from}</div>
                      <div className="relationship-item__arrow">
                        <span className={`relationship-item__arrow--${rel.type}`}>
                          →
                        </span>
                      </div>
                      <div className="relationship-item__to">{rel.to}</div>
                      <div className="relationship-item__description">{rel.description}</div>
                      <div className="relationship-item__value">
                        {formatCurrency(rel.value)}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            );
          })}
        </div>

        {relationships.length === 0 && (
          <div className="relationship-visualization-panel__empty">
            <p>No relationships detected between current positions.</p>
            <p className="relationship-visualization-panel__empty-hint">
              Relationships are automatically detected when instruments are connected
              (e.g., loan used as margin, bond used as collateral).
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
