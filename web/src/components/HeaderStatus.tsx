import { useState, useEffect, type ChangeEvent } from 'react';
import type { SnapshotPayload } from '../types/snapshot';
import { formatCurrency } from '../utils/formatters';
import { ModeSwitcher, type TradingMode } from './ModeSwitcher';
import { AccountSelector } from './AccountSelector';
import { BrokerSelector, type BrokerType } from './BrokerSelector';
import { useWebSocketStatus } from '../hooks/useWebSocket';
import { useNATS } from '../hooks/useNATS';
import { useBackendServices, type BackendServiceStatus } from '../hooks/useBackendServices';
import { ServiceConfigModal } from './ServiceConfigModal';

interface HeaderStatusProps {
  snapshot: SnapshotPayload | null;
  onModeChange?: (mode: TradingMode) => void;
  onAccountChange?: (accountId: string | null) => void;
  onBrokerChange?: (broker: BrokerType) => void;
  onStrategyStart?: () => void;
  onStrategyStop?: () => void;
  onDryRunToggle?: (enabled: boolean) => void;
  apiBaseUrl?: string;
  currentBroker?: BrokerType;
}

function statusBadge(
  ok: boolean,
  label: string,
  onClick?: () => void,
  attentionType?: string,
  enabled?: boolean
) {
  let badgeClass = ok ? 'status-badge--ok' : 'status-badge--warn';
  let indicator = '';

  // Check if service is disabled
  if (enabled === false) {
    badgeClass = 'status-badge--disabled';
    indicator = '🚫';
  } else if (attentionType && attentionType !== 'none') {
    // Add attention indicator based on type (only if not disabled)
    badgeClass = 'status-badge--attention';
    switch (attentionType) {
      case 'authentication':
        indicator = '🔐';
        break;
      case 'credentials':
        indicator = '🔑';
        break;
      case 'configuration':
        indicator = '⚙️';
        break;
      case 'error':
        indicator = '⚠️';
        break;
    }
  }

  const title = enabled === false
    ? 'Service is disabled'
    : attentionType && attentionType !== 'none'
      ? `Requires attention: ${attentionType}`
      : undefined;

  return (
    <span
      className={`status-badge ${badgeClass} ${onClick ? 'status-badge--clickable' : ''}`}
      onClick={onClick}
      title={title}
    >
      {indicator && <span className="status-badge__indicator">{indicator}</span>}
      {label}
    </span>
  );
}

export function HeaderStatus({
  snapshot,
  onModeChange,
  onAccountChange,
  onBrokerChange,
  onStrategyStart,
  onStrategyStop,
  onDryRunToggle,
  apiBaseUrl,
  currentBroker = 'AUTO'
}: HeaderStatusProps) {
  const isLive = snapshot?.mode === 'LIVE' || snapshot?.mode === 'LIVE_TRADING';
  const modeClass = isLive ? 'mode-indicator--live' : 'mode-indicator--paper';
  const isStrategyRunning = snapshot?.strategy === 'RUNNING';
  const { status: connectionStatus } = useWebSocketStatus();
  const { connected: natsConnected } = useNATS({
    autoConnect: true,
    subscribeMarketData: false, // Only connection status for now
    subscribeStrategySignals: false,
    subscribeStrategyDecisions: false,
  });
  const { statuses: backendStatuses, checkAllServices } = useBackendServices({ intervalMs: 10000, enabled: true });
  const [selectedService, setSelectedService] = useState<BackendServiceStatus | null>(null);
  const [natsServiceStatus, setNatsServiceStatus] = useState<BackendServiceStatus>({
    name: 'NATS',
    port: 4222,
    healthy: natsConnected,
    checking: false,
    enabled: true,
    running: natsConnected,
  });

  // Check NATS server health periodically
  useEffect(() => {
    const checkNatsServerHealth = async () => {
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 2000);

        const response = await fetch('http://localhost:8222/healthz', {
          method: 'GET',
          signal: controller.signal,
          headers: { 'Cache-Control': 'no-cache' },
        });

        clearTimeout(timeoutId);

        const serverHealthy = response.ok;
        const clientHealthy = natsConnected;

        setNatsServiceStatus({
          name: 'NATS',
          port: 4222,
          healthy: serverHealthy && clientHealthy,
          checking: false,
          error: serverHealthy ? undefined : 'Server health check failed',
          enabled: true,
          running: serverHealthy,
          attentionRequired: serverHealthy && clientHealthy ? 'none' : 'error',
          attentionMessage: !serverHealthy ? 'NATS server not responding' : !clientHealthy ? 'Client not connected' : undefined,
          lastChecked: new Date(),
        });
      } catch (error) {
        setNatsServiceStatus({
          name: 'NATS',
          port: 4222,
          healthy: false,
          checking: false,
          error: error instanceof Error ? error.message : 'Unknown error',
          enabled: true,
          running: false,
          attentionRequired: 'error',
          attentionMessage: 'NATS server health check failed',
          lastChecked: new Date(),
        });
      }
    };

    // Initial check
    checkNatsServerHealth();

    // Check every 10 seconds
    const intervalId = setInterval(checkNatsServerHealth, 10000);

    return () => clearInterval(intervalId);
  }, [natsConnected]);

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

  const handleDryRunToggle = (event: ChangeEvent<HTMLInputElement>) => {
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
        <div className="header__title-controls">
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
        {(onStrategyStart ?? onStrategyStop) && (
          <div className="header__strategy-controls">
            {!isStrategyRunning ? (
              <button
                type="button"
                className="btn btn--primary btn--small"
                onClick={handleStrategyStart}
                title="Start strategy"
              >
                ▶ Start
              </button>
            ) : (
              <button
                type="button"
                className="btn btn--secondary btn--small"
                onClick={handleStrategyStop}
                title="Stop strategy"
              >
                ⏹ Stop
              </button>
            )}
          </div>
        )}
        {onDryRunToggle && (
          <label className="header__dry-run-toggle">
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
        {onBrokerChange && (
          <BrokerSelector
            currentBroker={currentBroker}
            onBrokerChange={onBrokerChange}
            apiBaseUrl={apiBaseUrl}
          />
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
                  : 'Connection error'
          }
        >
          {connectionStatus === 'connected' ? 'WS' : connectionStatus === 'polling' ? 'Poll' : 'Conn'}
        </span>
        {statusBadge(
          natsServiceStatus.healthy,
          'NATS',
          () => {
            console.log('[Service Config] Opening NATS configuration');
            setSelectedService(natsServiceStatus);
          },
          natsServiceStatus.attentionRequired ?? undefined,
          natsServiceStatus.enabled
        )}
        {statusBadge(
          backendStatuses.rustBackend.healthy,
          'Rust',
          () => {
            console.log('[Service Config] Opening Rust Backend configuration');
            setSelectedService(backendStatuses.rustBackend);
          },
          backendStatuses.rustBackend.attentionRequired ?? undefined,
          backendStatuses.rustBackend.enabled
        )}
        {statusBadge(
          backendStatuses.alpaca.healthy,
          'Alpaca',
          () => {
            console.log('[Service Config] Opening Alpaca configuration');
            setSelectedService(backendStatuses.alpaca);
          },
          backendStatuses.alpaca.attentionRequired ?? undefined,
          backendStatuses.alpaca.enabled ?? undefined
        )}
        {statusBadge(
          backendStatuses.tradestation.healthy,
          'TS',
          () => {
            console.log('[Service Config] Opening TradeStation configuration');
            setSelectedService(backendStatuses.tradestation);
          },
          backendStatuses.tradestation.attentionRequired ?? undefined,
          backendStatuses.tradestation.enabled ?? undefined
        )}
        {statusBadge(
          backendStatuses.ib.healthy,
          'IB',
          () => {
            console.log('[Service Config] Opening IB configuration');
            setSelectedService(backendStatuses.ib);
          },
          backendStatuses.ib.attentionRequired ?? undefined,
          backendStatuses.ib.enabled ?? undefined
        )}
        {statusBadge(
          backendStatuses.discountBank.healthy,
          'DB',
          () => {
            console.log('[Service Config] Opening Discount Bank configuration');
            setSelectedService(backendStatuses.discountBank);
          },
          backendStatuses.discountBank.attentionRequired ?? undefined,
          backendStatuses.discountBank.enabled ?? undefined
        )}
        {statusBadge(
          backendStatuses.riskFreeRate.healthy,
          'RFR',
          () => {
            console.log('[Service Config] Opening Risk-Free Rate configuration');
            setSelectedService(backendStatuses.riskFreeRate);
          },
          backendStatuses.riskFreeRate.attentionRequired ?? undefined,
          backendStatuses.riskFreeRate.enabled ?? undefined
        )}
        {statusBadge(
          backendStatuses.tastytrade.healthy,
          'TT',
          () => {
            console.log('[Service Config] Opening Tastytrade configuration');
            setSelectedService(backendStatuses.tastytrade);
          },
          backendStatuses.tastytrade.attentionRequired ?? undefined,
          backendStatuses.tastytrade.enabled ?? undefined
        )}
      </div>
      <div className="header__metrics">
        <span>NetLiq: <strong>{formatCurrency(metrics.net_liq)}</strong></span>
        <span>Buying Power: <strong>{formatCurrency(metrics.buying_power)}</strong></span>
        <span>Margin Req: <strong>{formatCurrency(metrics.margin_requirement)}</strong></span>
        <span>Commissions: <strong>{formatCurrency(metrics.commissions)}</strong></span>
      </div>
      {selectedService && (
        <ServiceConfigModal
          service={selectedService}
          onClose={() => {
            console.log('[Service Config] Closing service configuration modal');
            setSelectedService(null);
          }}
          onRefresh={() => {
            console.log('[Service Config] Refreshing service status');
            void checkAllServices();
          }}
        />
      )}
    </header>
  );
}
