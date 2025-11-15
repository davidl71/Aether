import { useCallback, useEffect, useMemo, useState } from 'react';
import { useSnapshot } from './hooks/useSnapshot';
import { useBoxSpreadData } from './hooks/useBoxSpreadData';
import { HeaderStatus } from './components/HeaderStatus';
import { TabNavigation } from './components/TabNavigation';
import { DashboardTab } from './components/DashboardTab';
import { PositionsTable } from './components/PositionsTable';
import { OrdersPanel } from './components/OrdersPanel';
import { AlertsPanel } from './components/AlertsPanel';
import { ActionBar } from './components/ActionBar';
import { DetailModal } from './components/DetailModal';
import ScenarioSummary from './components/ScenarioSummary';
import BoxSpreadTable from './components/BoxSpreadTable';
import type { SnapshotPayload, SymbolSnapshot, PositionSnapshot } from './types/snapshot';
import type { BoxSpreadSummary } from './types';

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
  onSelectPosition: (position: PositionSnapshot) => void
) {
  if (!snapshot) {
    return <div className="panel panel--fill">Awaiting live data…</div>;
  }

  switch (tab) {
    case 'dashboard':
      return (
        <>
          <DashboardTab symbols={snapshot.symbols} onSelectSymbol={onSelectSymbol} />
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
      return <OrdersPanel orders={snapshot.orders} />;
    case 'alerts':
      return <AlertsPanel alerts={snapshot.alerts} />;
    default:
      return null;
  }
}

function App() {
  const [activeTab, setActiveTab] = useState<TabId>('dashboard');
  const [modal, setModal] = useState<ModalState>(null);
  const {
    snapshot,
    isLoading: snapshotLoading,
    error: snapshotError
  } = useSnapshot();
  const { data: scenarioData, isLoading: scenarioLoading, error: scenarioError } = useBoxSpreadData();

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
    setModal({ type: 'symbol', payload: symbol });
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

  return (
    <div className="app-shell">
      <HeaderStatus snapshot={snapshot} />

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
          <div className="panel panel--fill app-status app-status--error">{snapshotError}</div>
        )}
        {!snapshotLoading && !snapshotError &&
          renderTabContent(activeTab, snapshot, handleSelectSymbol, handleSelectPosition)}
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
            <ul className="modal-list">
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
