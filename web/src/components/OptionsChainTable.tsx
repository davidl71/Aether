import { useState, useMemo } from 'react';
import type { OptionSeries, OptionStrike } from '../types/snapshot';
import { formatCurrency } from '../utils/formatters';
import { calculate3StdWidth, calculateBoxSpreadStrikes } from '../utils/volatility';

interface OptionsChainTableProps {
  optionChains: OptionSeries[];
  underlyingPrice: number;
  onStrikeSelect: (strike: number, expiration: string) => void;
  selectedStrike: number | null;
  selectedExpiration: string | null;
}

export function OptionsChainTable({
  optionChains,
  underlyingPrice,
  onStrikeSelect,
  selectedStrike,
  selectedExpiration
}: OptionsChainTableProps) {
  // Group strikes across all expirations
  const allStrikes = useMemo(() => {
    const strikeSet = new Set<number>();
    optionChains.forEach((series) => {
      series.strikes.forEach((strike) => {
        strikeSet.add(strike.strike);
      });
    });
    return Array.from(strikeSet).sort((a, b) => a - b);
  }, [optionChains]);

  // Get strikes near ATM (at-the-money)
  const atmStrikes = useMemo(() => {
    if (allStrikes.length === 0) return [];

    // Find closest strike to underlying price
    const closestIndex = allStrikes.reduce((closest, strike, index) => {
      const currentDiff = Math.abs(strike - underlyingPrice);
      const closestDiff = Math.abs(allStrikes[closest] - underlyingPrice);
      return currentDiff < closestDiff ? index : closest;
    }, 0);

    // Show 10 strikes around ATM (5 above, 5 below, or as available)
    const start = Math.max(0, closestIndex - 5);
    const end = Math.min(allStrikes.length, closestIndex + 6);
    return allStrikes.slice(start, end);
  }, [allStrikes, underlyingPrice]);

  if (optionChains.length === 0 || allStrikes.length === 0) {
    return (
      <div className="panel">
        <div className="panel__header">
          <div>
            <h3>Options Chain</h3>
            <p>No options chain data available</p>
          </div>
        </div>
      </div>
    );
  }

  // Get first expiration for default display (or use selected)
  const displayExpiration = selectedExpiration || optionChains[0]?.expiration;
  const displaySeries = optionChains.find((s) => s.expiration === displayExpiration);

  if (!displaySeries) {
    return (
      <div className="panel">
        <div className="panel__header">
          <div>
            <h3>Options Chain</h3>
            <p>No data for selected expiration</p>
          </div>
        </div>
      </div>
    );
  }

  // Create strike map for quick lookup
  const strikeMap = new Map<number, OptionStrike>();
  displaySeries.strikes.forEach((strike) => {
    strikeMap.set(strike.strike, strike);
  });

  // Use ATM strikes or all strikes if fewer than 10
  const strikesToShow = atmStrikes.length > 0 ? atmStrikes : allStrikes;

  return (
    <div className="panel">
      <div className="panel__header">
        <div>
          <h3>Options Chain</h3>
          <p>Expiration: {displayExpiration} | Click a strike to see box spread combinations</p>
        </div>
        {optionChains.length > 1 && (
          <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
            {optionChains.map((series) => (
              <button
                key={series.expiration}
                type="button"
                className={`btn btn--small ${displayExpiration === series.expiration ? 'btn--primary' : 'btn--secondary'}`}
                onClick={() => {
                  // Reset selection when changing expiration
                  if (selectedStrike) {
                    onStrikeSelect(selectedStrike, series.expiration);
                  }
                }}
              >
                {series.expiration}
              </button>
            ))}
          </div>
        )}
      </div>
      <div className="table-wrapper">
        <table className="data-table" aria-label="Options Chain">
          <thead>
            <tr>
              <th>Call Bid</th>
              <th>Call Ask</th>
              <th>Strike</th>
              <th>Put Bid</th>
              <th>Put Ask</th>
            </tr>
          </thead>
          <tbody>
            {strikesToShow.map((strike) => {
              const optionData = strikeMap.get(strike);
              const isSelected = selectedStrike === strike && selectedExpiration === displayExpiration;
              const isAtm = Math.abs(strike - underlyingPrice) < 1; // Within $1 of underlying

              if (!optionData) {
                return (
                  <tr key={strike}>
                    <td colSpan={5} style={{ textAlign: 'center', color: '#666' }}>
                      No data for strike {strike}
                    </td>
                  </tr>
                );
              }

              return (
                <tr
                  key={strike}
                  onClick={() => onStrikeSelect(strike, displayExpiration)}
                  style={{
                    cursor: 'pointer',
                    backgroundColor: isSelected
                      ? 'rgba(59, 130, 246, 0.2)'
                      : isAtm
                        ? 'rgba(148, 163, 184, 0.08)'
                        : 'transparent',
                    border: isSelected ? '2px solid rgba(59, 130, 246, 0.6)' : undefined
                  }}
                  title={isSelected ? 'Selected strike' : isAtm ? 'At-the-money' : `Click to select strike ${strike}`}
                >
                  <td>{formatCurrency(optionData.call_bid)}</td>
                  <td>{formatCurrency(optionData.call_ask)}</td>
                  <td
                    style={{
                      fontWeight: isSelected || isAtm ? 'bold' : 'normal',
                      color: isSelected ? '#60a5fa' : isAtm ? '#cbd5f5' : '#e2e8f0'
                    }}
                  >
                    {strike.toFixed(2)}
                    {isAtm && !isSelected && <span style={{ fontSize: '0.75em', marginLeft: '4px', color: '#94a3b8' }}>(ATM)</span>}
                    {isSelected && <span style={{ fontSize: '0.75em', marginLeft: '4px', color: '#60a5fa' }}>✓</span>}
                  </td>
                  <td>{formatCurrency(optionData.put_bid)}</td>
                  <td>{formatCurrency(optionData.put_ask)}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      {selectedStrike && (
        <div style={{ padding: '12px', fontSize: '0.85rem', color: '#94a3b8', borderTop: '1px solid rgba(148, 163, 184, 0.2)' }}>
          <strong>Selected:</strong> Strike {selectedStrike} @ {displayExpiration}
          <br />
          <span style={{ fontSize: '0.8rem' }}>
            Box spreads will use this strike as the center (middle) with 3 standard deviation width
          </span>
        </div>
      )}
    </div>
  );
}
