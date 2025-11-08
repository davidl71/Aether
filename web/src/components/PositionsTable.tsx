import type { PositionSnapshot } from '../types/snapshot';
import { formatCurrency, formatPercent } from '../utils/formatters';
import { Sparkline } from './Sparkline';

interface PositionsTableProps {
  title: string;
  positions: PositionSnapshot[];
  onSelectPosition: (position: PositionSnapshot) => void;
}

export function PositionsTable({ title, positions, onSelectPosition }: PositionsTableProps) {
  return (
    <div className="panel panel--fill">
      <div className="panel__header">
        <div>
          <h2>{title}</h2>
          <p>ROI, maker/taker mix, and fair value deltas.</p>
        </div>
      </div>
      <div className="table-wrapper">
        <table className="data-table" aria-label={title + ' Table'}>
          <thead>
            <tr>
              <th>Name</th>
              <th>Quantity</th>
              <th>ROI</th>
              <th>Maker/Taker</th>
              <th>Rebate</th>
              <th>Vega</th>
              <th>Theta</th>
              <th>Fair Δ</th>
              <th>Range</th>
            </tr>
          </thead>
          <tbody>
            {positions.map((position) => (
              <tr key={position.name} onClick={() => onSelectPosition(position)}>
                <td>{position.name}</td>
                <td>{position.quantity}</td>
                <td className="data-table__positive">{formatPercent(position.roi)}</td>
                <td><span className="mk">{position.maker_count}</span>/<span className="tk">{position.taker_count}</span></td>
                <td>{formatCurrency(position.rebate_estimate)}</td>
                <td>{position.vega.toFixed(3)}</td>
                <td>{position.theta.toFixed(3)}</td>
                <td>{position.fair_diff.toFixed(3)}</td>
                <td><Sparkline candle={position.candle} /></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
