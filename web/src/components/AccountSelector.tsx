import { useState, useEffect } from 'react';

export interface AlpacaAccount {
  id: string;
  account_number: string;
  status: string;
  currency: string;
  buying_power: number;
  cash: number;
  portfolio_value: number;
  pattern_day_trader: boolean;
  trading_blocked: boolean;
}

interface AccountSelectorProps {
  currentAccountId: string | null;
  onAccountChange: (accountId: string | null) => void;
  apiBaseUrl?: string;
}

export function AccountSelector({ currentAccountId, onAccountChange, apiBaseUrl }: AccountSelectorProps) {
  const [accounts, setAccounts] = useState<AlpacaAccount[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isOpen, setIsOpen] = useState(false);

  const baseUrl = apiBaseUrl || 'http://127.0.0.1:8000';

  useEffect(() => {
    loadAccounts();
  }, []);

  const loadAccounts = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(`${baseUrl}/api/accounts`);
      const data = await response.json();
      if (data.accounts && Array.isArray(data.accounts)) {
        setAccounts(data.accounts);
      } else {
        setError('Failed to load accounts');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const handleAccountSelect = async (accountId: string | null) => {
    try {
      const response = await fetch(`${baseUrl}/api/account`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ account_id: accountId })
      });

      if (response.ok) {
        onAccountChange(accountId);
        setIsOpen(false);
      } else {
        setError('Failed to switch account');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  };

  // Match account by ID or account_number (handle both formats)
  const currentAccount = accounts.find(acc => {
    const accId = acc.id || acc.account_number;
    const accNum = acc.account_number || acc.id;
    return accId === currentAccountId || accNum === currentAccountId ||
           currentAccountId?.includes(accId) || currentAccountId?.includes(accNum);
  });

  if (loading && accounts.length === 0) {
    return <div className="account-selector">Loading accounts...</div>;
  }

  if (error && accounts.length === 0) {
    return (
      <div className="account-selector account-selector--error">
        <span>Account: {currentAccountId || 'Default'}</span>
        <button onClick={loadAccounts} className="account-selector__retry">Retry</button>
      </div>
    );
  }

  return (
    <div className="account-selector">
      <button
        className="account-selector__trigger"
        onClick={() => setIsOpen(!isOpen)}
        title="Select Alpaca account"
      >
        <span className="account-selector__label">Account:</span>
        <span className="account-selector__value">
          {currentAccount?.account_number || currentAccount?.id || currentAccountId || 'Default'}
        </span>
        <span className="account-selector__arrow">{isOpen ? '▲' : '▼'}</span>
      </button>

      {isOpen && (
        <>
          <div className="account-selector__overlay" onClick={() => setIsOpen(false)}></div>
          <div className="account-selector__dropdown">
            <div className="account-selector__header">
              <h3>Select Account</h3>
              <button
                className="account-selector__close"
                onClick={() => setIsOpen(false)}
              >
                ×
              </button>
            </div>
            <div className="account-selector__list">
              <button
                className={`account-selector__item ${!currentAccountId ? 'account-selector__item--active' : ''}`}
                onClick={() => handleAccountSelect(null)}
              >
                <div className="account-selector__item-main">
                  <span className="account-selector__item-id">Default Account</span>
                </div>
              </button>
              {accounts.map((account) => {
                const accountId = account.id || account.account_number;
                const accountNum = account.account_number || account.id;
                const isActive = accountId === currentAccountId ||
                                accountNum === currentAccountId ||
                                currentAccountId?.includes(accountId) ||
                                currentAccountId?.includes(accountNum);
                return (
                <button
                  key={accountId}
                  className={`account-selector__item ${isActive ? 'account-selector__item--active' : ''}`}
                  onClick={() => handleAccountSelect(accountId)}
                >
                  <div className="account-selector__item-main">
                    <span className="account-selector__item-id">{accountNum}</span>
                    <span className={`account-selector__item-status account-selector__item-status--${account.status?.toLowerCase()}`}>
                      {account.status}
                    </span>
                  </div>
                  <div className="account-selector__item-details">
                    <span>Value: ${account.portfolio_value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
                    <span>Buying Power: ${account.buying_power.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
                  </div>
                </button>
                );
              })}
            </div>
            {error && (
              <div className="account-selector__error">
                {error}
              </div>
            )}
          </div>
        </>
      )}
    </div>
  );
}
