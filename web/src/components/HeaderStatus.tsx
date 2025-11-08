import type { SnapshotPayload } from '../types/snapshot';
import { formatCurrency } from '../utils/formatters';

interface HeaderStatusProps {
  snapshot: SnapshotPayload | null;
}

function statusBadge(ok: boolean, label: string) {
  return (
    <span className={`status-badge ${ok ? 'status-badge--ok' : 'status-badge--warn'}`}>
      {label}
    </span>
  );
}

export function HeaderStatus({ snapshot }: HeaderStatusProps) {
  if (!snapshot) {
    return (
      <header className="header">
        <div className="header__title">IB Box Spread Terminal</div>
        <div className="header__meta">Awaiting snapshot…</div>
      </header>
    );
  }

  const { metrics } = snapshot;

  return (
    <header className="header">
      <div className="header__title">
        <span>IB Box Spread Terminal</span>
        <span className="header__timestamp">Time: {new Date(snapshot.generated_at).toLocaleTimeString()}</span>
      </div>
      <div className="header__meta">
        <span>Mode: <strong>{snapshot.mode}</strong></span>
        <span>Strategy: <strong>{snapshot.strategy}</strong></span>
        <span>Account: <strong>{snapshot.account_id}</strong></span>
      </div>
      <div className="header__status-line">
        {statusBadge(metrics.tws_ok, 'TWS')}
        {statusBadge(metrics.orats_ok, 'ORATS')}
        {statusBadge(metrics.portal_ok, 'Portal')}
        {statusBadge(metrics.questdb_ok, 'QuestDB')}
      </div>
      <div className="header__metrics">
        <span>NetLiq: <strong>{formatCurrency(metrics.net_liq)}</strong></span>
        <span>Buying Power: <strong>{formatCurrency(metrics.buying_power)}</strong></span>
        <span>Margin Req: <strong>{formatCurrency(metrics.margin_requirement)}</strong></span>
        <span>Commissions: <strong>{formatCurrency(metrics.commissions)}</strong></span>
      </div>
    </header>
  );
}
