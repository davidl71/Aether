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
import { FinancingComparisonTable } from './components/FinancingComparisonTable';
import { OptionsChainTable } from './components/OptionsChainTable';
import { BoxSpreadCombinations } from './components/BoxSpreadCombinations';
import { CandlestickChart } from './components/CandlestickChart';
import { useChartData } from './hooks/useChartData';
import { useTastytrade } from './hooks/useTastytrade';
import { TastytradeDashboard } from './components/TastytradeDashboard';
import { getRustBackendUrl } from './config/ports';
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
  const { data: scenarioData, isLoading: scenarioLoading, error: scenarioError } = useBoxSpreadData();
  const { watchlist, addSymbol, removeSymbol, isDefault } = useSymbolWatchlist();
  const {
    snapshot: tastySnapshot,
    isLoading: tastyLoading,
    error: tastyError,
    isAvailable: tastyAvailable,
    refresh: tastyRefresh,
  } = useTastytrade();

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
      if (event.key.toLowerCase() === 'b') {
        event.preventDefault();
        handleBuyCombo();
      }
      if (event.key.toLowerCase() === 's' && event.shiftKey) {
        event.preventDefault();
        handleSellCombo();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleBuyCombo, handleSellCombo]);

  const apiBaseUrl = getRustBackendUrl();

  const handleModeChange = useCallback(async (mode: 'PAPER' | 'LIVE') => {
    try {
      const response = await fetch(`${apiBaseUrl}/api/mode`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode })
      });
      if (response.ok) {
        console.log('Mode changed to:', (await response.json()).mode);
      } else {
        console.error('Failed to change mode:', response.statusText);
      }
    } catch (error) {
      console.error('Error changing mode:', error);
    }
  }, [apiBaseUrl]);

  const handleAccountChange = useCallback(async (accountId: string | null) => {
    try {
      const response = await fetch(`${apiBaseUrl}/api/account`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ account_id: accountId })
      });
      if (response.ok) {
        console.log('Account changed to:', (await response.json()).account_id);
      } else {
        console.error('Failed to change account:', response.statusText);
      }
    } catch (error) {
      console.error('Error changing account:', error);
    }
  }, [apiBaseUrl]);

  const handleStrategyStart = useCallback(async () => {
    try {
      const response = await fetch(`${apiBaseUrl}/api/v1/strategy/start`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });
      if (!response.ok) {
        alert(`Failed to start strategy: ${response.statusText}`);
      }
    } catch (error) {
      alert(`Error starting strategy: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }, [apiBaseUrl]);

  const handleStrategyStop = useCallback(async () => {
    try {
      const response = await fetch(`${apiBaseUrl}/api/v1/strategy/stop`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });
      if (!response.ok) {
        alert(`Failed to stop strategy: ${response.statusText}`);
      }
    } catch (error) {
      alert(`Error stopping strategy: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }, [apiBaseUrl]);

  const handleDryRunToggle = useCallback(async (enabled: boolean) => {
    try {
      const response = await fetch(`${apiBaseUrl}/api/mode`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode: enabled ? 'DRY-RUN' : 'LIVE' })
      });
      if (!response.ok) {
        alert(`Failed to toggle dry-run: ${response.statusText}`);
      }
    } catch (error) {
      alert(`Error toggling dry-run: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }, [apiBaseUrl]);

  const handleCancelOrder = useCallback(async (orderId: string) => {
    try {
      const response = await fetch(`${apiBaseUrl}/api/v1/orders/cancel`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ order_id: orderId })
      });
      if (!response.ok) {
        throw new Error(response.statusText);
      }
    } catch (error) {
      console.error('Error cancelling order:', error);
      throw error;
    }
  }, [apiBaseUrl]);

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
        onStrategyStart={handleStrategyStart}
        onStrategyStop={handleStrategyStop}
        onDryRunToggle={handleDryRunToggle}
        apiBaseUrl={apiBaseUrl}
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
        {snapshotLoading && <div className="panel panel--fill">Loading live snapshot…</div>}
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
                </ul>
              </details>
            </div>
          </div>
        )}
        {!snapshotLoading && !snapshotError &&
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
          )}
        {activeTab === 'dashboard' && (
          <TastytradeDashboard
            snapshot={tastySnapshot}
            isLoading={tastyLoading}
            error={tastyError}
            isAvailable={tastyAvailable}
            onRefresh={tastyRefresh}
          />
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

              {/* Financing Comparison: Box Spread vs Treasury */}
              {scenarioData && scenarioData.underlying === modal.payload.symbol && (
                <div style={{ marginTop: '24px' }}>
                  <FinancingComparisonTable scenarios={scenarioData.scenarios} symbol={modal.payload.symbol} />
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
