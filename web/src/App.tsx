import { useCallback, useEffect, useMemo, useState } from 'react';
import { useSnapshot } from './hooks/useSnapshot';
import { useBoxSpreadData } from './hooks/useBoxSpreadData';
import { useSymbolWatchlist } from './hooks/useSymbolWatchlist';
import { HeaderStatus } from './components/HeaderStatus';
import { TabNavigation } from './components/TabNavigation';
import { DashboardTab } from './components/DashboardTab';
import { BankAccountsPanel } from './components/BankAccountsPanel';
import { PositionsTable } from './components/PositionsTable';
import { OrdersPanel } from './components/OrdersPanel';
import { AlertsPanel } from './components/AlertsPanel';
import { ActionBar } from './components/ActionBar';
import { DetailModal } from './components/DetailModal';
import ScenarioSummary from './components/ScenarioSummary';
import BoxSpreadTable from './components/BoxSpreadTable';
import { YieldCurveTable } from './components/YieldCurveTable';
import { OptionsChainTable } from './components/OptionsChainTable';
import { BoxSpreadCombinations } from './components/BoxSpreadCombinations';
import { CandlestickChart } from './components/CandlestickChart';
import { useChartData } from './hooks/useChartData';
import type { SnapshotPayload, SymbolSnapshot, PositionSnapshot } from './types/snapshot';
import type { BoxSpreadSummary } from './types';
import type { Timeframe } from './types/chart';

const TABS = [
  { id: 'dashboard', title: 'Dashboard' },
  { id: 'current', title: 'Current Positions' },
  { id: 'historic', title: 'Historic Positions' },
  { id: 'orders', title: 'Orders' },
  { id: 'alerts', title: 'Alerts' }
] as const;

type TabId = typeof TABS[number]['id'];

type ModalState =
  | { type: 'symbol'; payload: SymbolSnapshot }
  | { type: 'position'; payload: PositionSnapshot }
  | { type: 'action'; payload: { title: string; message: string } }
  | null;

function renderTabContent(
  tab: TabId,
  snapshot: SnapshotPayload | null,
  onSelectSymbol: (symbol: SymbolSnapshot) => void,
  onSelectPosition: (position: PositionSnapshot) => void,
  watchlist: string[],
  onAddSymbol: (symbol: string) => { success: boolean; error?: string },
  onRemoveSymbol: (symbol: string) => void,
  isDefaultSymbol: (symbol: string) => boolean,
  onCancelOrder: (orderId: string) => Promise<void>,
  apiBaseUrl: string
) {
  if (!snapshot) {
    return <div className="panel panel--fill">Awaiting live data…</div>;
  }

  // Filter symbols to only show those in watchlist
  const filteredSymbols = snapshot.symbols.filter((symbol) =>
    watchlist.includes(symbol.symbol.toUpperCase())
  );

  switch (tab) {
    case 'dashboard':
      return (
        <>
          <DashboardTab
            symbols={filteredSymbols}
            onSelectSymbol={onSelectSymbol}
            watchlist={watchlist}
            onAddSymbol={onAddSymbol}
            onRemoveSymbol={onRemoveSymbol}
            isDefaultSymbol={isDefaultSymbol}
          />
          <BankAccountsPanel serviceUrl="http://localhost:8003" />
        </>
      );
    case 'current':
      return (
        <PositionsTable
          title="Current Positions"
          positions={snapshot.positions}
          onSelectPosition={onSelectPosition}
        />
      );
    case 'historic':
      return (
        <PositionsTable
          title="Historic Positions"
          positions={snapshot.historic}
          onSelectPosition={onSelectPosition}
        />
      );
      case 'orders':
        return <OrdersPanel orders={snapshot.orders} onCancelOrder={onCancelOrder} apiBaseUrl={apiBaseUrl} />;
    case 'alerts':
      return <AlertsPanel alerts={snapshot.alerts} />;
    default:
      return null;
  }
}

function App() {
  console.log('App component rendering...');
  const [activeTab, setActiveTab] = useState<TabId>('dashboard');
  const [modal, setModal] = useState<ModalState>(null);
  const [selectedStrike, setSelectedStrike] = useState<number | null>(null);
  const [selectedExpiration, setSelectedExpiration] = useState<string | null>(null);
  const [chartTimeframe, setChartTimeframe] = useState<Timeframe>('1D');
  const {
    snapshot,
    isLoading: snapshotLoading,
    error: snapshotError
  } = useSnapshot();

  console.log('App state:', { snapshotLoading, snapshotError, hasSnapshot: !!snapshot });
  const { data: scenarioData, isLoading: scenarioLoading, error: scenarioError } = useBoxSpreadData();
  const { watchlist, addSymbol, removeSymbol, isDefault } = useSymbolWatchlist();

  const scenarioSummary = useMemo<BoxSpreadSummary | null>(() => {
    if (!scenarioData || scenarioData.scenarios.length === 0) {
      return null;
    }

    // Filter to European-style scenarios only for summary (default behavior)
    const europeanScenarios = scenarioData.scenarios.filter(
      (scenario) => scenario.option_style === 'European'
    );

    const scenariosToUse = europeanScenarios.length > 0 ? europeanScenarios : scenarioData.scenarios;

    const avgApr =
      scenariosToUse.reduce((acc, scenario) => acc + scenario.annualized_return, 0) /
      scenariosToUse.length;

    return {
      totalScenarios: scenariosToUse.length,
      avgApr,
      probableCount: scenariosToUse.filter((scenario) => scenario.fill_probability > 0).length,
      maxAprScenario: scenariosToUse.reduce((best, scenario) => {
        if (!best || scenario.annualized_return > best.annualized_return) {
          return scenario;
        }
        return best;
      }, scenariosToUse[0])
    };
  }, [scenarioData]);

  const handleSelectSymbol = (symbol: SymbolSnapshot) => {
    console.log('Symbol selected:', symbol.symbol);
    setModal({ type: 'symbol', payload: symbol });
    // Reset strike selection when opening new symbol
    setSelectedStrike(null);
    setSelectedExpiration(null);
  };

  const handleStrikeSelect = (strike: number, expiration: string) => {
    setSelectedStrike(strike);
    setSelectedExpiration(expiration);
  };

  // Calculate days to expiry from expiration date string
  const calculateDaysToExpiry = (expiration: string): number => {
    try {
      // Try parsing YYYYMMDD format
      if (/^\d{8}$/.test(expiration)) {
        const year = parseInt(expiration.substring(0, 4));
        const month = parseInt(expiration.substring(4, 6)) - 1;
        const day = parseInt(expiration.substring(6, 8));
        const expiryDate = new Date(year, month, day);
        const today = new Date();
        return Math.ceil((expiryDate.getTime() - today.getTime()) / (1000 * 60 * 60 * 24));
      }
      // Try parsing ISO format
      const expiryDate = new Date(expiration);
      if (!isNaN(expiryDate.getTime())) {
        const today = new Date();
        return Math.ceil((expiryDate.getTime() - today.getTime()) / (1000 * 60 * 60 * 24));
      }
    } catch {
      // Fallback
    }
    return 30; // Default to 30 days if parsing fails
  };

  const handleSelectPosition = (position: PositionSnapshot) => {
    setModal({ type: 'position', payload: position });
  };

  const handleBuyCombo = useCallback(() => {
    setModal({
      type: 'action',
      payload: {
        title: 'Buy Combo',
        message:
          'Submitting maker-priced combo order (mock). Integrate REST/WS call to order manager here.'
      }
    });
  }, []);

  const handleSellCombo = useCallback(() => {
    setModal({
      type: 'action',
      payload: {
        title: 'Sell Combo',
        message:
          'Submitting offsetting combo to flatten position (mock). Wire to strategy endpoint once ready.'
      }
    });
  }, []);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Ignore if typing in input/textarea
      if (
        event.target instanceof HTMLInputElement ||
        event.target instanceof HTMLTextAreaElement ||
        (event.target instanceof HTMLElement && event.target.isContentEditable)
      ) {
        return;
      }

      // Strategy control shortcuts
      if (event.key.toLowerCase() === 's' && !event.shiftKey && !event.ctrlKey && !event.metaKey && !event.altKey) {
        event.preventDefault();
        if (snapshot?.strategy !== 'RUNNING') {
          handleStrategyStart();
        }
      }
      if (event.key.toLowerCase() === 't' && !event.ctrlKey && !event.metaKey && !event.altKey) {
        event.preventDefault();
        if (snapshot?.strategy === 'RUNNING') {
          handleStrategyStop();
        }
      }

      // Toggle dry-run mode
      if (event.key.toLowerCase() === 'd' && !event.ctrlKey && !event.metaKey && !event.altKey) {
        event.preventDefault();
        const isDryRun = snapshot?.mode === 'DRY-RUN' || snapshot?.mode === 'PAPER';
        handleDryRunToggle(!isDryRun);
      }

      // Combo actions
      if (event.key.toLowerCase() === 'b' && !event.ctrlKey && !event.metaKey && !event.altKey) {
        event.preventDefault();
        handleBuyCombo();
      }
      if (event.key.toLowerCase() === 's' && event.shiftKey && !event.ctrlKey && !event.metaKey && !event.altKey) {
        event.preventDefault();
        handleSellCombo();
      }

      // Tab navigation shortcuts (1-5)
      const numKey = parseInt(event.key, 10);
      if (numKey >= 1 && numKey <= TABS.length && !event.ctrlKey && !event.metaKey && !event.altKey) {
        event.preventDefault();
        const targetTab = TABS[numKey - 1];
        if (targetTab) {
          setActiveTab(targetTab.id);
        }
      }

      // Tab/Shift+Tab for cycling through tabs
      if (event.key === 'Tab' && !event.ctrlKey && !event.metaKey) {
        // Only prevent default if we're not in a form element (already handled above)
        if (!event.shiftKey) {
          // Tab: move to next tab
          event.preventDefault();
          const currentIndex = TABS.findIndex(tab => tab.id === activeTab);
          const nextIndex = (currentIndex + 1) % TABS.length;
          setActiveTab(TABS[nextIndex].id);
        } else {
          // Shift+Tab: move to previous tab
          event.preventDefault();
          const currentIndex = TABS.findIndex(tab => tab.id === activeTab);
          const prevIndex = (currentIndex - 1 + TABS.length) % TABS.length;
          setActiveTab(TABS[prevIndex].id);
        }
      }

      // Cancel orders (Ctrl+K to avoid conflict with help shortcut)
      if (event.key.toLowerCase() === 'k' && event.ctrlKey && !event.metaKey && !event.altKey) {
        event.preventDefault();
        // Cancel the most recent active order if available
        if (snapshot?.orders && snapshot.orders.length > 0) {
          const activeOrders = snapshot.orders.filter(order =>
            order.status === 'SUBMITTED' || order.status === 'PENDING'
          );
          if (activeOrders.length > 0) {
            const mostRecentOrder = activeOrders[activeOrders.length - 1];
            // Extract order ID from order text (heuristic)
            const orderIdMatch = mostRecentOrder.text.match(/order[_\s]*#?(\d+)/i) ||
                                 mostRecentOrder.text.match(/id[:\s]+(\d+)/i);
            if (orderIdMatch) {
              handleCancelOrder(orderIdMatch[1]).catch(err => {
                console.error('Failed to cancel order:', err);
              });
            }
          }
        }
      }

      // Escape to close modal
      if (event.key === 'Escape' && modal) {
        event.preventDefault();
        setModal(null);
      }

      // Keyboard shortcuts help (K or ?)
      if ((event.key.toLowerCase() === 'k' || event.key === '?') && !event.ctrlKey && !event.metaKey && !event.altKey) {
        event.preventDefault();
        setModal({
          type: 'action',
          payload: {
            title: 'Keyboard Shortcuts',
            message: `Keyboard Shortcuts:
• S - Start strategy (when stopped)
• T - Stop strategy (when running)
• D - Toggle dry-run mode
• B - Buy combo
• Shift+S - Sell combo
• 1-5 - Switch tabs (Dashboard, Positions, Historic, Orders, Alerts)
• Tab/Shift+Tab - Cycle through tabs
• Ctrl+K - Cancel most recent active order
• Escape - Close modal
• K or ? - Show this help`
          }
        });
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleBuyCombo, handleSellCombo, handleStrategyStart, handleStrategyStop, handleDryRunToggle, handleCancelOrder, modal, snapshot, activeTab]);

  const handleModeChange = useCallback(async (mode: 'PAPER' | 'LIVE') => {
    try {
      // Get API URL from environment or use default
      const apiUrl = (import.meta as unknown as { env?: Record<string, unknown> }).env?.VITE_API_URL as string | undefined;
      const baseUrl = apiUrl ? apiUrl.replace('/api/snapshot', '') : 'http://127.0.0.1:8000';

      // Send mode change request to backend
      const response = await fetch(`${baseUrl}/api/mode`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode })
      });

      if (response.ok) {
        const data = await response.json();
        console.log('Mode changed to:', data.mode);
        // The snapshot will update on next poll with the new mode
      } else {
        console.error('Failed to change mode:', response.statusText);
      }
    } catch (error) {
      console.error('Error changing mode:', error);
      // If API call fails, still allow UI change (for development/offline)
    }
  }, []);

  const handleAccountChange = useCallback(async (accountId: string | null) => {
    try {
      // Get API URL from environment or use default
      const apiUrl = (import.meta as unknown as { env?: Record<string, unknown> }).env?.VITE_API_URL as string | undefined;
      const baseUrl = apiUrl ? apiUrl.replace('/api/snapshot', '') : 'http://127.0.0.1:8000';

      // Send account change request to backend
      const response = await fetch(`${baseUrl}/api/account`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ account_id: accountId })
      });

      if (response.ok) {
        const data = await response.json();
        console.log('Account changed to:', data.account_id);
        // The snapshot will update on next poll with the new account
      } else {
        console.error('Failed to change account:', response.statusText);
      }
    } catch (error) {
      console.error('Error changing account:', error);
    }
  }, []);

  const handleStrategyStart = useCallback(async () => {
    try {
      const apiUrl = (import.meta as unknown as { env?: Record<string, unknown> }).env?.VITE_API_URL as string | undefined;
      const baseUrl = apiUrl ? apiUrl.replace('/api/snapshot', '') : 'http://127.0.0.1:8000';

      const response = await fetch(`${baseUrl}/api/v1/strategy/start`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });

      if (response.ok) {
        console.log('Strategy started');
        // Snapshot will update on next poll
      } else {
        console.error('Failed to start strategy:', response.statusText);
        alert(`Failed to start strategy: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Error starting strategy:', error);
      alert(`Error starting strategy: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }, []);

  const handleStrategyStop = useCallback(async () => {
    try {
      const apiUrl = (import.meta as unknown as { env?: Record<string, unknown> }).env?.VITE_API_URL as string | undefined;
      const baseUrl = apiUrl ? apiUrl.replace('/api/snapshot', '') : 'http://127.0.0.1:8000';

      const response = await fetch(`${baseUrl}/api/v1/strategy/stop`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });

      if (response.ok) {
        console.log('Strategy stopped');
        // Snapshot will update on next poll
      } else {
        console.error('Failed to stop strategy:', response.statusText);
        alert(`Failed to stop strategy: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Error stopping strategy:', error);
      alert(`Error stopping strategy: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }, []);

  const handleDryRunToggle = useCallback(async (enabled: boolean) => {
    try {
      const apiUrl = (import.meta as unknown as { env?: Record<string, unknown> }).env?.VITE_API_URL as string | undefined;
      const baseUrl = apiUrl ? apiUrl.replace('/api/snapshot', '') : 'http://127.0.0.1:8000';

      const response = await fetch(`${baseUrl}/api/mode`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode: enabled ? 'DRY-RUN' : 'LIVE' })
      });

      if (response.ok) {
        console.log('Dry-run mode:', enabled ? 'enabled' : 'disabled');
        // Snapshot will update on next poll
      } else {
        console.error('Failed to toggle dry-run:', response.statusText);
        alert(`Failed to toggle dry-run: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Error toggling dry-run:', error);
      alert(`Error toggling dry-run: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }, []);

  const handleCancelOrder = useCallback(async (orderId: string) => {
    try {
      const apiUrl = (import.meta as unknown as { env?: Record<string, unknown> }).env?.VITE_API_URL as string | undefined;
      const baseUrl = apiUrl ? apiUrl.replace('/api/snapshot', '') : 'http://127.0.0.1:8000';

      const response = await fetch(`${baseUrl}/api/v1/orders/cancel`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ order_id: orderId })
      });

      if (response.ok) {
        console.log(`Order ${orderId} cancelled`);
        // Snapshot will update on next poll
      } else {
        console.error('Failed to cancel order:', response.statusText);
        throw new Error(response.statusText);
      }
    } catch (error) {
      console.error('Error cancelling order:', error);
      throw error;
    }
  }, []);

  // Get API base URL for account selector
  const apiUrl = (import.meta as unknown as { env?: Record<string, unknown> }).env?.VITE_API_URL as string | undefined;
  const apiBaseUrl = apiUrl ? apiUrl.replace('/api/snapshot', '') : 'http://127.0.0.1:8000';

  // Chart data for symbol modal
  const chartSymbol = modal?.type === 'symbol' ? modal.payload.symbol : '';
  const { data: chartData, isLoading: chartLoading } = useChartData({
    symbol: chartSymbol,
    timeframe: chartTimeframe,
    apiBaseUrl
  });

  return (
    <div className={`app-shell ${snapshot?.mode === 'LIVE' || snapshot?.mode === 'LIVE_TRADING' ? 'app-shell--live' : ''}`}>
      <HeaderStatus
        snapshot={snapshot}
        onModeChange={handleModeChange}
        onAccountChange={handleAccountChange}
        onBrokerChange={setCurrentBroker}
        onStrategyStart={handleStrategyStart}
        onStrategyStop={handleStrategyStop}
        onDryRunToggle={handleDryRunToggle}
        apiBaseUrl={apiBaseUrl}
        currentBroker={currentBroker}
      />

      <div className="scenario-section">
        {scenarioLoading && <div className="app-status">Loading box spread scenarios…</div>}
        {scenarioError && <div className="app-status app-status--error">{scenarioError}</div>}
        {!scenarioLoading && !scenarioError && scenarioData && scenarioSummary && (
          <>
            <ScenarioSummary summary={scenarioSummary} />
            <BoxSpreadTable
              scenarios={scenarioData.scenarios}
              asOf={scenarioData.as_of}
              underlying={scenarioData.underlying}
            />
          </>
        )}
      </div>

      <TabNavigation tabs={TABS} activeTab={activeTab} onSelect={setActiveTab} />

      <main className="app-main">
        {snapshotLoading && (
          <div className="panel panel--fill" style={{ padding: '40px', textAlign: 'center' }}>
            <div>Loading live snapshot…</div>
            <div style={{ marginTop: '16px', fontSize: '14px', color: '#9ca3af' }}>
              Connecting to backend service...
            </div>
          </div>
        )}
        {!snapshotLoading && snapshotError && (
          <div className="panel panel--fill app-status app-status--error">
            <div style={{ padding: '20px' }}>
              <h3 style={{ marginTop: 0, color: '#ef4444' }}>Connection Error</h3>
              <p>{snapshotError}</p>
              <details style={{ marginTop: '16px', fontSize: '14px', color: '#9ca3af' }}>
                <summary style={{ cursor: 'pointer', marginBottom: '8px' }}>Troubleshooting</summary>
                <ul style={{ marginLeft: '20px', marginTop: '8px' }}>
                  <li>Check if the backend service is running</li>
                  <li>Verify the API endpoint URL is correct</li>
                  <li>Check browser console for CORS errors</li>
                  <li>If using a local backend, ensure it's listening on the expected port</li>
                  <li>Try using fallback data: <code>/data/snapshot.json</code></li>
                </ul>
              </details>
            </div>
          </div>
        )}
        {!snapshotLoading && !snapshotError && (
          renderTabContent(
            activeTab,
            snapshot,
            handleSelectSymbol,
            handleSelectPosition,
            watchlist,
            addSymbol,
            removeSymbol,
            isDefault,
            handleCancelOrder,
            apiBaseUrl
          ) || (
            <div className="panel panel--fill" style={{ padding: '40px', textAlign: 'center' }}>
              <div>No data available</div>
              <div style={{ marginTop: '16px', fontSize: '14px', color: '#9ca3af' }}>
                Waiting for snapshot data...
              </div>
            </div>
          )
        )}
      </main>

      <ActionBar onBuyCombo={handleBuyCombo} onSellCombo={handleSellCombo} />

      {modal && (
        <DetailModal
          title={
            modal.type === 'symbol'
              ? modal.payload.symbol
              : modal.type === 'position'
                ? modal.payload.name
                : modal.payload.title
          }
          onClose={() => setModal(null)}
        >
          {modal.type === 'symbol' && (
            <>
              <div style={{ marginBottom: '16px', padding: '12px', background: 'rgba(148, 163, 184, 0.1)', borderRadius: '8px' }}>
                <ul className="modal-list" style={{ margin: 0 }}>
                  <li>
                    Last: <strong>{modal.payload.last.toFixed(2)}</strong>
                  </li>
                  <li>
                    Bid/Ask: <strong>{modal.payload.bid.toFixed(2)}</strong> /
                    <strong> {modal.payload.ask.toFixed(2)}</strong>
                  </li>
                  <li>
                    Spread: <strong>{modal.payload.spread.toFixed(2)}</strong>
                  </li>
                  <li>
                    ROI: <strong>{modal.payload.roi.toFixed(2)}%</strong>
                  </li>
                  <li>
                    Maker/Taker: <strong>{modal.payload.maker_count}</strong>/<strong>{modal.payload.taker_count}</strong>
                  </li>
                  <li>
                    Volume: <strong>{modal.payload.volume.toLocaleString()}</strong>
                  </li>
                </ul>
              </div>

              {/* Candlestick Chart */}
              <div style={{ marginBottom: '24px' }}>
                {chartLoading ? (
                  <div style={{ padding: '40px', textAlign: 'center', color: '#9ca3af' }}>
                    Loading chart data...
                  </div>
                ) : (
                  <CandlestickChart
                    symbol={modal.payload.symbol}
                    data={chartData}
                    timeframe={chartTimeframe}
                    onTimeframeChange={setChartTimeframe}
                    height={350}
                  />
                )}
              </div>

              {/* Options Chain Table */}
              {modal.payload.option_chains && modal.payload.option_chains.length > 0 ? (
                <>
                  <OptionsChainTable
                    optionChains={modal.payload.option_chains}
                    underlyingPrice={modal.payload.last}
                    onStrikeSelect={handleStrikeSelect}
                    selectedStrike={selectedStrike}
                    selectedExpiration={selectedExpiration}
                  />

                  {/* Box Spread Combinations when strike is selected */}
                  {selectedStrike && selectedExpiration && scenarioData && scenarioData.underlying === modal.payload.symbol && (
                    <div style={{ marginTop: '24px' }}>
                      <BoxSpreadCombinations
                        scenarios={scenarioData.scenarios}
                        selectedStrike={selectedStrike}
                        selectedExpiration={selectedExpiration}
                        underlyingPrice={modal.payload.last}
                        daysToExpiry={calculateDaysToExpiry(selectedExpiration)}
                      />
                    </div>
                  )}
                </>
              ) : (
                <div style={{ padding: '20px', textAlign: 'center', color: '#666' }}>
                  <p>No options chain data available for {modal.payload.symbol}</p>
                </div>
              )}

              {/* Yield Curve Table (always show if data available) */}
              {scenarioData && scenarioData.underlying === modal.payload.symbol && (
                <div style={{ marginTop: '24px' }}>
                  <YieldCurveTable scenarios={scenarioData.scenarios} symbol={modal.payload.symbol} />
                </div>
              )}
            </>
          )}
          {modal.type === 'position' && (
            <ul className="modal-list">
              <li>
                Quantity: <strong>{modal.payload.quantity}</strong>
              </li>
              <li>
                ROI: <strong>{modal.payload.roi.toFixed(2)}%</strong>
              </li>
              <li>
                Maker/Taker: <strong>{modal.payload.maker_count}</strong>/<strong>{modal.payload.taker_count}</strong>
              </li>
              <li>
                Rebate Estimate: <strong>${modal.payload.rebate_estimate.toFixed(2)}</strong>
              </li>
              <li>
                Vega: <strong>{modal.payload.vega.toFixed(4)}</strong>
              </li>
              <li>
                Theta: <strong>{modal.payload.theta.toFixed(4)}</strong>
              </li>
              <li>
                Fair Diff: <strong>{modal.payload.fair_diff.toFixed(4)}</strong>
              </li>
            </ul>
          )}
          {modal.type === 'action' && <p>{modal.payload.message}</p>}
        </DetailModal>
      )}
    </div>
  );
}

export default App;
