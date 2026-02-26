import type { TastytradeSnapshot, TastytradePosition } from '../hooks/useTastytrade';

interface TastytradeDashboardProps {
  snapshot: TastytradeSnapshot | null;
  isLoading: boolean;
  error: string | null;
  isAvailable: boolean;
  onRefresh: () => void;
}

function formatCurrency(value: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
  }).format(value);
}

function PnlBadge({ value }: { value: number }) {
  const cls = value >= 0 ? 'pnl--positive' : 'pnl--negative';
  return <span className={cls}>{formatCurrency(value)}</span>;
}

function PositionRow({ position }: { position: TastytradePosition }) {
  return (
    <tr>
      <td>{position.symbol}</td>
      <td>{position.instrumentType}</td>
      <td className="num">{position.quantity}</td>
      <td className="num">{formatCurrency(position.averageOpenPrice)}</td>
      <td className="num">{formatCurrency(position.currentPrice)}</td>
      <td className="num">
        <PnlBadge value={position.unrealizedPnl} />
      </td>
    </tr>
  );
}

export function TastytradeDashboard({
  snapshot,
  isLoading,
  error,
  isAvailable,
  onRefresh,
}: TastytradeDashboardProps) {
  if (!isAvailable) {
    return (
      <div className="panel" style={{ padding: '16px', opacity: 0.6 }}>
        <h3>Tastytrade</h3>
        <p style={{ color: '#9ca3af' }}>
          Service not available. Start the Tastytrade backend on port 8005.
        </p>
      </div>
    );
  }

  if (isLoading && !snapshot) {
    return (
      <div className="panel" style={{ padding: '16px' }}>
        <h3>Tastytrade</h3>
        <p>Loading...</p>
      </div>
    );
  }

  if (error && !snapshot) {
    return (
      <div className="panel" style={{ padding: '16px' }}>
        <h3>Tastytrade</h3>
        <p style={{ color: '#ef4444' }}>{error}</p>
        <button onClick={onRefresh} style={{ marginTop: '8px' }}>
          Retry
        </button>
      </div>
    );
  }

  if (!snapshot) return null;

  const { account, balance, positions } = snapshot;

  return (
    <div className="panel" style={{ padding: '16px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
        <h3 style={{ margin: 0 }}>
          Tastytrade
          {account.nickname ? ` — ${account.nickname}` : ''}
        </h3>
        <button
          onClick={onRefresh}
          title="Refresh"
          style={{ background: 'none', border: '1px solid #374151', borderRadius: '4px', padding: '4px 10px', cursor: 'pointer', color: '#d1d5db' }}
        >
          Refresh
        </button>
      </div>

      {/* Balance summary */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))', gap: '12px', marginBottom: '16px' }}>
        <div>
          <div style={{ fontSize: '11px', color: '#9ca3af', textTransform: 'uppercase' }}>Net Liq</div>
          <div style={{ fontSize: '18px', fontWeight: 600 }}>{formatCurrency(balance.netLiquidatingValue)}</div>
        </div>
        <div>
          <div style={{ fontSize: '11px', color: '#9ca3af', textTransform: 'uppercase' }}>Cash</div>
          <div style={{ fontSize: '18px', fontWeight: 600 }}>{formatCurrency(balance.cashBalance)}</div>
        </div>
        <div>
          <div style={{ fontSize: '11px', color: '#9ca3af', textTransform: 'uppercase' }}>Buying Power</div>
          <div style={{ fontSize: '18px', fontWeight: 600 }}>{formatCurrency(balance.buyingPower)}</div>
        </div>
        <div>
          <div style={{ fontSize: '11px', color: '#9ca3af', textTransform: 'uppercase' }}>Maintenance</div>
          <div style={{ fontSize: '18px', fontWeight: 600 }}>{formatCurrency(balance.maintenanceRequirement)}</div>
        </div>
      </div>

      {/* Positions table */}
      {positions.length > 0 ? (
        <div style={{ overflowX: 'auto' }}>
          <table className="table" style={{ width: '100%', fontSize: '13px' }}>
            <thead>
              <tr>
                <th>Symbol</th>
                <th>Type</th>
                <th className="num">Qty</th>
                <th className="num">Avg Price</th>
                <th className="num">Current</th>
                <th className="num">P&L</th>
              </tr>
            </thead>
            <tbody>
              {positions.map((pos, i) => (
                <PositionRow key={`${pos.symbol}-${i}`} position={pos} />
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <p style={{ color: '#9ca3af', fontSize: '13px' }}>No open positions.</p>
      )}

      <div style={{ marginTop: '8px', fontSize: '11px', color: '#6b7280' }}>
        Last updated: {new Date(snapshot.timestamp).toLocaleTimeString()}
      </div>
    </div>
  );
}
