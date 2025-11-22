import { useState, useEffect } from 'react';

export type BrokerType = 'TWS' | 'IB_CLIENT_PORTAL' | 'ALPACA' | 'AUTO';

export interface BrokerOption {
  id: BrokerType;
  name: string;
  description: string;
  available: boolean;
}

interface BrokerSelectorProps {
  currentBroker: BrokerType;
  onBrokerChange: (broker: BrokerType) => void;
  apiBaseUrl?: string;
}

const BROKER_OPTIONS: BrokerOption[] = [
  {
    id: 'TWS',
    name: 'Interactive Brokers TWS',
    description: 'TWS API (socket-based, low latency)',
    available: true,
  },
  {
    id: 'IB_CLIENT_PORTAL',
    name: 'IB Client Portal',
    description: 'IB Client Portal REST API (no TWS required)',
    available: true,
  },
  {
    id: 'ALPACA',
    name: 'Alpaca Markets',
    description: 'Alpaca REST API (paper and live trading)',
    available: true,
  },
  {
    id: 'AUTO',
    name: 'Auto (Priority Order)',
    description: 'Automatically select from configured broker priorities',
    available: true,
  },
];

export function BrokerSelector({ currentBroker, onBrokerChange, apiBaseUrl }: BrokerSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [availableBrokers, setAvailableBrokers] = useState<BrokerType[]>(['TWS', 'AUTO']);

  const baseUrl = apiBaseUrl || 'http://127.0.0.1:8000';

  useEffect(() => {
    // Check which brokers are available
    checkBrokerAvailability();
  }, []);

  const checkBrokerAvailability = async () => {
    const available: BrokerType[] = ['AUTO']; // Auto is always available

    // Check TWS (assume available if backend is running)
    try {
      const response = await fetch(`${baseUrl}/health`, { signal: AbortSignal.timeout(2000) });
      if (response.ok) {
        available.push('TWS');
      }
    } catch {
      // TWS not available
    }

    // Check Alpaca (check if Alpaca service is running)
    try {
      const response = await fetch(`${baseUrl}/api/account`, { signal: AbortSignal.timeout(2000) });
      if (response.ok) {
        const data = await response.json();
        if (data.account_id?.startsWith('ALPACA') || data.account_id === 'ALPACA') {
          available.push('ALPACA');
        }
      }
    } catch {
      // Alpaca not available
    }

    // Check IB Client Portal (assume available if TWS is available)
    if (available.includes('TWS')) {
      available.push('IB_CLIENT_PORTAL');
    }

    setAvailableBrokers(available);
  };

  const handleBrokerSelect = async (broker: BrokerType) => {
    try {
      // Send broker change request to backend
      const response = await fetch(`${baseUrl}/api/v1/config`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          broker: broker === 'AUTO' ? null : broker.toLowerCase(),
        }),
      });

      if (response.ok) {
        onBrokerChange(broker);
        setIsOpen(false);
      } else {
        console.error('Failed to change broker:', response.statusText);
      }
    } catch (error) {
      console.error('Error changing broker:', error);
      // Still allow UI change for development
      onBrokerChange(broker);
      setIsOpen(false);
    }
  };

  const currentBrokerOption = BROKER_OPTIONS.find(b => b.id === currentBroker);

  return (
    <div className="broker-selector">
      <button
        className="broker-selector__trigger"
        onClick={() => setIsOpen(!isOpen)}
        title="Select broker"
      >
        <span className="broker-selector__label">Broker:</span>
        <span className="broker-selector__value">
          {currentBrokerOption?.name || currentBroker}
        </span>
        <span className="broker-selector__arrow">{isOpen ? '▲' : '▼'}</span>
      </button>

      {isOpen && (
        <>
          <div className="broker-selector__overlay" onClick={() => setIsOpen(false)}></div>
          <div className="broker-selector__dropdown">
            <div className="broker-selector__header">
              <h3>Select Broker</h3>
              <button
                className="broker-selector__close"
                onClick={() => setIsOpen(false)}
              >
                ×
              </button>
            </div>
            <div className="broker-selector__list">
              {BROKER_OPTIONS.map((broker) => {
                const isAvailable = availableBrokers.includes(broker.id);
                const isActive = broker.id === currentBroker;
                return (
                  <button
                    key={broker.id}
                    className={`broker-selector__item ${isActive ? 'broker-selector__item--active' : ''} ${!isAvailable ? 'broker-selector__item--disabled' : ''}`}
                    onClick={() => isAvailable && handleBrokerSelect(broker.id)}
                    disabled={!isAvailable}
                    title={!isAvailable ? 'Broker not available' : broker.description}
                  >
                    <div className="broker-selector__item-main">
                      <span className="broker-selector__item-name">{broker.name}</span>
                      {!isAvailable && (
                        <span className="broker-selector__item-unavailable">(Not Available)</span>
                      )}
                    </div>
                    <div className="broker-selector__item-description">{broker.description}</div>
                  </button>
                );
              })}
            </div>
          </div>
        </>
      )}
    </div>
  );
}
