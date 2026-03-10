import { useEffect, useMemo, useState } from 'react';
import type { PositionSnapshot } from '../types/snapshot';
import { formatCurrency } from '../utils/formatters';
import '../styles/relationship-visualization.css';
import type { BankAccount } from '../types/banking';
import { fetchRelationships, type FrontendRelationship } from '../api/calculations';

interface RelationshipVisualizationPanelProps {
  positions: PositionSnapshot[];
  bankAccounts?: BankAccount[];
}

export function RelationshipVisualizationPanel({
  positions,
  bankAccounts = [],
}: RelationshipVisualizationPanelProps) {
  const [relationships, setRelationships] = useState<FrontendRelationship[]>([]);
  const [nodes, setNodes] = useState<string[]>([]);

  useEffect(() => {
    let cancelled = false;

    async function loadRelationships() {
      try {
        const result = await fetchRelationships(positions, bankAccounts);
        if (!cancelled) {
          setRelationships(result.relationships);
          setNodes(result.nodes);
        }
      } catch (error) {
        if (!cancelled) {
          console.error('Failed to fetch relationships:', error);
          setRelationships([]);
          setNodes([]);
        }
      }
    }

    void loadRelationships();
    return () => {
      cancelled = true;
    };
  }, [positions, bankAccounts]);

  // Group relationships by type
  const relationshipsByType = useMemo(() => {
    const grouped: Record<string, FrontendRelationship[]> = {
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
