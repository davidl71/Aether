import { useState, type KeyboardEvent } from 'react';
import type { SymbolSnapshot } from '../types/snapshot';
import { formatPercent } from '../utils/formatters';
import { Sparkline } from './Sparkline';

interface DashboardTabProps {
  symbols: SymbolSnapshot[];
  onSelectSymbol: (symbol: SymbolSnapshot) => void;
  watchlist: string[];
  onAddSymbol: (symbol: string) => { success: boolean; error?: string };
  onRemoveSymbol: (symbol: string) => void;
  isDefaultSymbol: (symbol: string) => boolean;
}

export function DashboardTab({
  symbols,
  onSelectSymbol,
  watchlist,
  onAddSymbol,
  onRemoveSymbol,
  isDefaultSymbol
}: DashboardTabProps) {
  const [newSymbol, setNewSymbol] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const handleAddSymbol = () => {
    if (!newSymbol.trim()) {
      setError('Please enter a symbol');
      return;
    }

    const result = onAddSymbol(newSymbol);
    if (result.success) {
      setNewSymbol('');
      setError(null);
      setSuccess(`Symbol ${newSymbol.toUpperCase()} added`);
      setTimeout(() => setSuccess(null), 3000);
    } else {
      setError(result.error ?? 'Failed to add symbol');
      setSuccess(null);
    }
  };

  const handleKeyPress = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleAddSymbol();
    }
  };

  // Get symbols in watchlist but not in snapshot (missing symbols)
  const missingSymbols = watchlist.filter(
    (symbol) => !symbols.some((s) => s.symbol.toUpperCase() === symbol.toUpperCase())
  );

  return (
    <div className="panel panel--fill">
      <div className="panel__header">
        <div>
          <h2>Dashboard</h2>
          <p>Live symbol metrics and combo health.</p>
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', alignItems: 'flex-end' }}>
          <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
            <input
              type="text"
              value={newSymbol}
              onChange={(e) => {
                setNewSymbol(e.target.value.toUpperCase());
                setError(null);
              }}
              onKeyPress={handleKeyPress}
              placeholder="Add symbol (e.g., SPY)"
              maxLength={5}
              style={{
                padding: '6px 12px',
                borderRadius: '6px',
                border: '1px solid rgba(148, 163, 184, 0.3)',
                background: 'rgba(30, 41, 59, 0.8)',
                color: '#e2e8f0',
                fontSize: '0.9rem',
                width: '120px'
              }}
            />
            <button
              type="button"
              className="btn btn--primary btn--small"
              onClick={handleAddSymbol}
              title="Add symbol to watchlist"
            >
              + Add
            </button>
          </div>
          {error && (
            <span style={{ fontSize: '0.85rem', color: '#f44336', marginTop: '4px' }}>{error}</span>
          )}
          {success && (
            <span style={{ fontSize: '0.85rem', color: '#4caf50', marginTop: '4px' }}>{success}</span>
          )}
        </div>
      </div>

      {missingSymbols.length > 0 && (
        <div
          style={{
            padding: '8px 16px',
            background: 'rgba(148, 163, 184, 0.1)',
            borderBottom: '1px solid rgba(148, 163, 184, 0.2)',
            fontSize: '0.85rem',
            color: '#94a3b8'
          }}
        >
          <strong>Note:</strong> The following symbols are in your watchlist but not available in the current snapshot:{' '}
          {missingSymbols.join(', ')}
        </div>
      )}

      <div className="table-wrapper">
        <table className="data-table" aria-label="Symbol Metrics">
          <thead>
            <tr>
              <th>Symbol</th>
              <th>Last</th>
              <th>Bid</th>
              <th>Ask</th>
              <th>Spread</th>
              <th>ROI</th>
              <th>Maker/Taker</th>
              <th>Volume</th>
              <th>Range</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {symbols.length === 0 ? (
              <tr>
                <td colSpan={10} style={{ textAlign: 'center', padding: '40px', color: '#666' }}>
                  No symbols available. Add symbols to your watchlist above.
                </td>
              </tr>
            ) : (
              symbols.map((symbol) => {
                const isDefault = isDefaultSymbol(symbol.symbol);
                return (
                  <tr
                    key={symbol.symbol}
                    onClick={(e) => {
                      // Don't trigger symbol selection if clicking remove button
                      if ((e.target as HTMLElement).closest('button')) {
                        return;
                      }
                      e.preventDefault();
                      e.stopPropagation();
                      onSelectSymbol(symbol);
                    }}
                    style={{ cursor: 'pointer' }}
                  >
                    <td>
                      {symbol.symbol}
                      {isDefault && (
                        <span
                          style={{
                            fontSize: '0.75em',
                            marginLeft: '6px',
                            color: '#94a3b8',
                            fontWeight: 'normal'
                          }}
                          title="Default symbol"
                        >
                          (default)
                        </span>
                      )}
                    </td>
                    <td>{symbol.last.toFixed(2)}</td>
                    <td>{symbol.bid.toFixed(2)}</td>
                    <td>{symbol.ask.toFixed(2)}</td>
                    <td>{symbol.spread.toFixed(2)}</td>
                    <td className="data-table__positive">{formatPercent(symbol.roi)}</td>
                    <td>
                      <span className="mk">{symbol.maker_count}</span>/<span className="tk">{symbol.taker_count}</span>
                    </td>
                    <td>{symbol.volume.toLocaleString()}</td>
                    <td>
                      <Sparkline candle={symbol.candle} />
                    </td>
                    <td
                      onClick={(e) => {
                        e.stopPropagation();
                      }}
                    >
                      {!isDefault && (
                        <button
                          type="button"
                          className="btn btn--small"
                          onClick={() => onRemoveSymbol(symbol.symbol)}
                          title={`Remove ${symbol.symbol} from watchlist`}
                          style={{
                            background: 'rgba(239, 68, 68, 0.2)',
                            color: '#fca5a5',
                            border: '1px solid rgba(239, 68, 68, 0.4)',
                            padding: '4px 8px',
                            fontSize: '0.75rem'
                          }}
                        >
                          ×
                        </button>
                      )}
                    </td>
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
