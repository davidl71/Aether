import type { SymbolSnapshot } from '../types/snapshot';
import { formatPercent } from '../utils/formatters';
import { Sparkline } from './Sparkline';

interface DashboardTabProps {
  symbols: SymbolSnapshot[];
  onSelectSymbol: (symbol: SymbolSnapshot) => void;
}

export function DashboardTab({ symbols, onSelectSymbol }: DashboardTabProps) {
  return (
    <div className="panel panel--fill">
      <div className="panel__header">
        <div>
          <h2>Dashboard</h2>
          <p>Live symbol metrics and combo health.</p>
        </div>
      </div>
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
            </tr>
          </thead>
          <tbody>
            {symbols.map((symbol) => (
              <tr key={symbol.symbol} onClick={() => onSelectSymbol(symbol)}>
                <td>{symbol.symbol}</td>
                <td>{symbol.last.toFixed(2)}</td>
                <td>{symbol.bid.toFixed(2)}</td>
                <td>{symbol.ask.toFixed(2)}</td>
                <td>{symbol.spread.toFixed(2)}</td>
                <td className="data-table__positive">{formatPercent(symbol.roi)}</td>
                <td><span className="mk">{symbol.maker_count}</span>/<span className="tk">{symbol.taker_count}</span></td>
                <td>{symbol.volume.toLocaleString()}</td>
                <td><Sparkline candle={symbol.candle} /></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
