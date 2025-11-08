interface ActionBarProps {
  onBuyCombo: () => void;
  onSellCombo: () => void;
}

export function ActionBar({ onBuyCombo, onSellCombo }: ActionBarProps) {
  return (
    <div className="action-bar">
      <div>
        <h3>Controls</h3>
        <p>Mirror of the TUI quick actions for rapid order entry.</p>
      </div>
      <div className="action-bar__actions">
        <button type="button" onClick={onBuyCombo} className="btn btn--primary">
          Buy Combo (B)
        </button>
        <button type="button" onClick={onSellCombo} className="btn btn--secondary">
          Sell Combo (Shift+S)
        </button>
      </div>
    </div>
  );
}
