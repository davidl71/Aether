import type { SnapshotPayload } from '../types/snapshot';
import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import type { BackendServicesStatus } from '../hooks/useBackendServices';
import { BACKEND_ROLE_ORDER, BACKEND_ROLE_LABELS, getBackendsByRole } from '../hooks/useBackendServices';
import { formatCurrency } from '../utils/formatters';
import { ModeSwitcher, type TradingMode } from './ModeSwitcher';
import { AccountSelector } from './AccountSelector';
import { useWebSocketStatus } from '../hooks/useWebSocket';

interface HeaderStatusProps {
  snapshot: SnapshotPayload | null;
  backendStatuses?: BackendServicesStatus | null;
  onModeChange?: (mode: TradingMode) => void;
  onAccountChange?: (accountId: string | null) => void;
  onStrategyStart?: () => void;
  onStrategyStop?: () => void;
  onDryRunToggle?: (enabled: boolean) => void;
  apiBaseUrl?: string;
}

const STALE_MS = 60_000;
function snapshotTimeAndStale(snapshot: SnapshotPayload): { timeStr: string; isStale: boolean; ageLabel: string } {
  if (!snapshot?.generated_at) return { timeStr: '--:--:--', isStale: false, ageLabel: '' };
  const date = new Date(snapshot.generated_at);
  const ageMs = Date.now() - date.getTime();
  const ageSec = Math.floor(ageMs / 1000);
  return {
    timeStr: date.toLocaleTimeString(),
    isStale: ageMs > STALE_MS,
    ageLabel: ageSec >= 3600 ? `${Math.floor(ageSec / 3600)}h ago` : ageSec >= 60 ? `${ageSec}s ago` : 'stale'
  };
}

function statusBadge(ok: boolean, label: string) {
  return (
    <span className={`status-badge ${ok ? 'status-badge--ok' : 'status-badge--warn'}`}>
      {label}
    </span>
  );
}

export function HeaderStatus({
  snapshot,
  backendStatuses,
  onModeChange,
  onAccountChange,
  onStrategyStart,
  onStrategyStop,
  onDryRunToggle,
  apiBaseUrl
}: HeaderStatusProps) {
  const { t, i18n } = useTranslation();
  const isLive = snapshot?.mode === 'LIVE' || snapshot?.mode === 'LIVE_TRADING';
  const modeClass = isLive ? 'mode-indicator--live' : 'mode-indicator--paper';
  const isStrategyRunning = snapshot?.strategy === 'RUNNING';
  const { status: connectionStatus } = useWebSocketStatus();

  if (!snapshot) {
    return (
      <header className="header">
        <div className="header__title">IB Box Spread Terminal</div>
        <div className="header__meta">{t('app.status.awaitingData')}</div>
        {backendStatuses && (
          <div className="header__status-line" aria-label="Backend services">
            {BACKEND_ROLE_ORDER.map((role) => {
              const keys = getBackendsByRole(backendStatuses)[role];
              if (!keys?.length) return null;
              return (
                <span key={role} className="header__status-group">
                  <span className="header__status-group-label">{BACKEND_ROLE_LABELS[role]}:</span>
                  {keys.map((key) => {
                    const s = backendStatuses[key];
                    return (
                      <span
                        key={key}
                        className={`status-badge ${s.healthy ? 'status-badge--ok' : 'status-badge--warn'}`}
                        title={s.error ? `${s.name}: ${s.error}` : s.checking ? `${s.name}: checking…` : `${s.name}: ${s.healthy ? 'up' : 'down'}`}
                      >
                        {s.name}
                      </span>
                    );
                  })}
                </span>
              );
            }).filter(Boolean).reduce<ReactNode[]>((acc, node, i) => {
              if (i > 0) acc.push(<span key={`sep-${i}`} className="header__status-sep" aria-hidden>|</span>);
              acc.push(node);
              return acc;
            }, [])}
          </div>
        )}
      </header>
    );
  }

  const { metrics } = snapshot;
  const timeStale = snapshotTimeAndStale(snapshot);

  const handleModeChange = (mode: TradingMode) => {
    if (onModeChange) {
      onModeChange(mode);
    } else {
      // Fallback: just log (for now, until backend integration)
      console.log('Mode change requested:', mode);
    }
  };

  const handleStrategyStart = () => {
    if (onStrategyStart) {
      onStrategyStart();
    } else {
      console.log('Strategy start requested');
    }
  };

  const handleStrategyStop = () => {
    if (onStrategyStop) {
      onStrategyStop();
    } else {
      console.log('Strategy stop requested');
    }
  };

  const handleDryRunToggle = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (onDryRunToggle) {
      onDryRunToggle(event.target.checked);
    } else {
      console.log('Dry-run toggle:', event.target.checked);
    }
  };

  return (
    <header className="header">
      <div className="header__title">
        <span>IB Box Spread Terminal</span>
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <span className="header__lang" style={{ marginRight: '8px' }}>
            <button
              type="button"
              className={`btn btn--small ${i18n.language === 'en' ? 'btn--primary' : 'btn--secondary'}`}
              onClick={() => i18n.changeLanguage('en')}
              title="English"
              aria-pressed={i18n.language === 'en'}
            >
              EN
            </button>
            <button
              type="button"
              className={`btn btn--small ${i18n.language === 'he' ? 'btn--primary' : 'btn--secondary'}`}
              onClick={() => i18n.changeLanguage('he')}
              title="עברית"
              aria-pressed={i18n.language === 'he'}
            >
              עברית
            </button>
          </span>
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
          <span className="header__timestamp">
            Time: {timeStale.timeStr}
            {timeStale.isStale && (
              <span className="header__stale" title={`Data ${timeStale.ageLabel}`}>
                {' '}(stale)
              </span>
            )}
          </span>
        </div>
      </div>
      <div className="header__meta">
        <span>Strategy: <strong>{snapshot.strategy}</strong></span>
        {(onStrategyStart || onStrategyStop) && (
          <div className="header__strategy-controls" style={{ display: 'inline-flex', gap: '8px', marginLeft: '16px' }}>
            {!isStrategyRunning ? (
              <button
                type="button"
                className="btn btn--primary btn--small"
                onClick={handleStrategyStart}
                title={t('app.strategy.start')}
              >
                ▶ {t('app.strategy.start')}
              </button>
            ) : (
              <button
                type="button"
                className="btn btn--secondary btn--small"
                onClick={handleStrategyStop}
                title={t('app.strategy.stop')}
              >
                ⏹ {t('app.strategy.stop')}
              </button>
            )}
          </div>
        )}
        {onDryRunToggle && (
          <label className="header__dry-run-toggle" style={{ display: 'inline-flex', alignItems: 'center', gap: '8px', marginLeft: '16px' }}>
            <input
              type="checkbox"
              checked={snapshot.mode === 'DRY-RUN' || snapshot.mode === 'PAPER'}
              onChange={handleDryRunToggle}
              title="Toggle dry-run mode"
            />
            <span>Dry-Run</span>
          </label>
        )}
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
        {(snapshot.account_id?.startsWith('TASTY') || snapshot.account_id === 'TASTYTRADE') && (
          <span className="header__data-source">Data: <strong>Tastytrade</strong></span>
        )}
      </div>
      <div className="header__status-line">
        {backendStatuses && (
          <>
            {BACKEND_ROLE_ORDER.map((role) => {
              const keys = getBackendsByRole(backendStatuses)[role];
              if (!keys?.length) return null;
              return (
                <span key={role} className="header__status-group">
                  <span className="header__status-group-label">{BACKEND_ROLE_LABELS[role]}:</span>
                  {keys.map((key) => {
                    const s = backendStatuses[key];
                    return (
                      <span
                        key={key}
                        className={`status-badge ${s.healthy ? 'status-badge--ok' : 'status-badge--warn'}`}
                        title={s.error ? `${s.name}: ${s.error}` : s.checking ? `${s.name}: checking…` : `${s.name}: ${s.healthy ? 'up' : 'down'}`}
                      >
                        {s.name}
                      </span>
                    );
                  })}
                </span>
              );
            }).filter(Boolean).reduce<ReactNode[]>((acc, node, i) => {
              if (i > 0) acc.push(<span key={`sep-${i}`} className="header__status-sep" aria-hidden>|</span>);
              acc.push(node);
              return acc;
            }, [])}
            <span className="header__status-sep" aria-hidden>|</span>
          </>
        )}
        {statusBadge(metrics.tws_ok, 'TWS')}
        {statusBadge(metrics.orats_ok, 'ORATS')}
        {statusBadge(metrics.portal_ok, 'Portal')}
        {(() => {
          const gatewayUrl = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_IB_GATEWAY_URL ?? 'https://localhost:5001';
          return (
            <a
              href={gatewayUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="header__gateway-link"
              title="Open IB Gateway or TWS (Client Portal) to log in"
            >
              Open IB Gateway
            </a>
          );
        })()}
        {statusBadge(metrics.questdb_ok, 'QuestDB')}
        <span
          className={`status-badge ${
            connectionStatus === 'connected'
              ? 'status-badge--ok'
              : connectionStatus === 'polling'
                ? 'status-badge--ok' // Polling is a valid, working state
                : 'status-badge--warn'
          }`}
          title={
            connectionStatus === 'connected'
              ? 'WebSocket connected - Real-time updates'
              : connectionStatus === 'polling'
                ? 'Using polling - Updates every 2 seconds (WebSocket server not available)'
                : connectionStatus === 'connecting'
                  ? 'Connecting to WebSocket...'
                  : t('app.strategy.connectionError')
          }
        >
          {connectionStatus === 'connected' ? 'WS' : connectionStatus === 'polling' ? 'Poll' : 'Conn'}
        </span>
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
