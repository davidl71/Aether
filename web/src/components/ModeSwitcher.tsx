import { useState, useEffect } from 'react';

export type TradingMode = 'PAPER' | 'LIVE';

interface ModeSwitcherProps {
  currentMode: string;
  onModeChange: (mode: TradingMode) => void;
  disabled?: boolean;
}

export function ModeSwitcher({ currentMode, onModeChange, disabled = false }: ModeSwitcherProps) {
  const [isLive, setIsLive] = useState(currentMode === 'LIVE' || currentMode === 'LIVE_TRADING');
  const [showConfirm, setShowConfirm] = useState(false);

  useEffect(() => {
    setIsLive(currentMode === 'LIVE' || currentMode === 'LIVE_TRADING');
  }, [currentMode]);

  const handleToggle = () => {
    if (isLive) {
      // Switching from LIVE to PAPER - no confirmation needed
      setIsLive(false);
      onModeChange('PAPER');
      setShowConfirm(false);
    } else {
      // Switching from PAPER to LIVE - require confirmation
      setShowConfirm(true);
    }
  };

  const handleConfirmLive = () => {
    setIsLive(true);
    onModeChange('LIVE');
    setShowConfirm(false);
  };

  const handleCancel = () => {
    setShowConfirm(false);
  };

  return (
    <div className="mode-switcher">
      {showConfirm && (
        <div className="mode-switcher__confirm">
          <div className="mode-switcher__confirm-content">
            <h3>⚠️ Switch to LIVE Trading?</h3>
            <p>You are about to switch to <strong>LIVE</strong> trading mode.</p>
            <p>This will use real money and execute real trades.</p>
            <div className="mode-switcher__confirm-actions">
              <button
                className="mode-switcher__confirm-btn mode-switcher__confirm-btn--danger"
                onClick={handleConfirmLive}
                disabled={disabled}
              >
                Yes, Switch to LIVE
              </button>
              <button
                className="mode-switcher__confirm-btn"
                onClick={handleCancel}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
      <button
        className={`mode-switcher__toggle ${isLive ? 'mode-switcher__toggle--live' : 'mode-switcher__toggle--paper'}`}
        onClick={handleToggle}
        disabled={disabled}
        title={isLive ? 'Currently in LIVE mode - Click to switch to PAPER' : 'Currently in PAPER mode - Click to switch to LIVE'}
      >
        <span className="mode-switcher__indicator"></span>
        <span className="mode-switcher__label">
          {isLive ? 'LIVE' : 'PAPER'}
        </span>
      </button>
    </div>
  );
}
