import type { SnapshotPayload } from '../types/snapshot';
import { formatCurrency } from '../utils/formatters';
import { ModeSwitcher, type TradingMode } from './ModeSwitcher';
import { AccountSelector } from './AccountSelector';

interface HeaderStatusProps {
  snapshot: SnapshotPayload | null;
  onModeChange?: (mode: TradingMode) => void;
  onAccountChange?: (accountId: string | null) => void;
  apiBaseUrl?: string;
}

function statusBadge(ok: boolean, label: string) {
  return (
    <span className={`status-badge ${ok ? 'status-badge--ok' : 'status-badge--warn'}`}>
      {label}
    </span>
  );
}

export function HeaderStatus({ snapshot, onModeChange, onAccountChange, apiBaseUrl }: HeaderStatusProps) {
  const isLive = snapshot?.mode === 'LIVE' || snapshot?.mode === 'LIVE_TRADING';
  const modeClass = isLive ? 'mode-indicator--live' : 'mode-indicator--paper';

  if (!snapshot) {
    return (
      <header className="header">
        <div className="header__title">IB Box Spread Terminal</div>
        <div className="header__meta">Awaiting snapshot…</div>
      </header>
    );
  }

  const { metrics } = snapshot;

  const handleModeChange = (mode: TradingMode) => {
    if (onModeChange) {
      onModeChange(mode);
    } else {
      // Fallback: just log (for now, until backend integration)
      console.log('Mode change requested:', mode);
    }
  };

  return (
    <header className="header">
      <div className="header__title">
        <span>IB Box Spread Terminal</span>
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <span className={`mode-indicator ${modeClass}`}>
            <span className="mode-indicator__dot"></span>
            <span>{snapshot.mode}</span>
          </span>
          {onModeChange && (
            <ModeSwitcher currentMode={snapshot.mode} onModeChange={handleModeChange} />
          )}
          {snapshot.account_id && (
            <span className="header__account-badge" title={`Account: ${snapshot.account_id}`}>
              <span className="header__account-label">Account:</span>
              <span className="header__account-value">{snapshot.account_id}</span>
            </span>
          )}
          <span className="header__timestamp">Time: {new Date(snapshot.generated_at).toLocaleTimeString()}</span>
        </div>
      </div>
      <div className="header__meta">
        <span>Strategy: <strong>{snapshot.strategy}</strong></span>
        {onAccountChange ? (
          <AccountSelector
            currentAccountId={snapshot.account_id}
            onAccountChange={onAccountChange}
            apiBaseUrl={apiBaseUrl}
          />
        ) : (
          <span>Account: <strong>{snapshot.account_id}</strong></span>
        )}
        {(snapshot.account_id?.startsWith('ALPACA') || snapshot.account_id === 'ALPACA' ||
          (snapshot.account_id && snapshot.account_id !== 'DU123456' && snapshot.account_id !== 'TRADESTATION')) ? (
          <span className="header__data-source">Data: <strong>Alpaca</strong></span>
        ) : null}
        {snapshot.account_id === 'TRADESTATION' && (
          <span className="header__data-source">Data: <strong>TradeStation</strong></span>
        )}
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
