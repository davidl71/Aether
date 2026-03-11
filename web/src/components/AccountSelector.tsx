import { useState, useEffect } from 'react';
import { getServiceUrl, SERVICE_PORTS } from '../config/ports';

export interface UnifiedAccount {
  id: string;
  account_number?: string;
  source: 'IB' | 'Discount Bank';
  status?: string;
  currency?: string;
  buying_power?: number;
  cash?: number;
  portfolio_value?: number;
  net_liquidation?: number;
  excess_liquidity?: number;
  pattern_day_trader?: boolean;
  trading_blocked?: boolean;
}

interface AccountSelectorProps {
  currentAccountId: string | null;
  onAccountChange: (accountId: string | null) => void;
  apiBaseUrl?: string;
}

export function AccountSelector({ currentAccountId, onAccountChange, apiBaseUrl }: AccountSelectorProps) {
  const [accounts, setAccounts] = useState<UnifiedAccount[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isOpen, setIsOpen] = useState(false);

  const baseUrl = apiBaseUrl || getServiceUrl('ib');

  useEffect(() => {
    loadAccounts();
  }, []);

  const loadAccounts = async () => {
    setLoading(true);
    setError(null);
    const allAccounts: UnifiedAccount[] = [];

    try {
      // Fetch from all services in parallel so one slow backend doesn't block the rest
      const [ibResult, dbResult] = await Promise.allSettled([
        fetch(`${getServiceUrl('ib')}/api/accounts`, { signal: AbortSignal.timeout(2000) }),
        fetch(`${getServiceUrl('discountBank')}/api/bank-accounts`, { signal: AbortSignal.timeout(2000) }),
      ]);

      // Process IB response
      if (ibResult.status === 'fulfilled' && ibResult.value.ok) {
        const ibData = await ibResult.value.json();
        if (ibData.accounts && Array.isArray(ibData.accounts)) {
          ibData.accounts.forEach((acc: any) => {
            if (acc.account_id || acc.id) {
              allAccounts.push({
                id: acc.account_id || acc.id,
                source: 'IB',
                net_liquidation: acc.net_liquidation,
                buying_power: acc.buying_power,
                excess_liquidity: acc.excess_liquidity,
              });
            }
          });
        }
      }

      // Process Discount Bank response
      if (dbResult.status === 'fulfilled' && dbResult.value.ok) {
        const dbData = await dbResult.value.json();
        if (Array.isArray(dbData)) {
          dbData.forEach((acc: any) => {
            allAccounts.push({
              id: acc.account_path || acc.id || `DISCOUNT-${acc.name}`,
              account_number: acc.account_path,
              source: 'Discount Bank',
              cash: acc.balance,
              portfolio_value: acc.balance,
            });
          });
        }
      }

      setAccounts(allAccounts);
      if (allAccounts.length === 0) {
        setError('No accounts found from any service');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const handleAccountSelect = async (accountId: string | null) => {
    try {
      // Find the account to determine which service to use
      const selectedAccount = accounts.find(acc =>
        acc.id === accountId ||
        acc.account_number === accountId ||
        accountId?.includes(acc.id) ||
        accountId?.includes(acc.account_number || '')
      );

      if (selectedAccount) {
        // Determine service URL based on account source
        let serviceUrl = baseUrl;
        if (selectedAccount.source === 'Discount Bank') {
          serviceUrl = getServiceUrl('discountBank');
        } else {
          serviceUrl = getServiceUrl('ib');
        }

        const response = await fetch(`${serviceUrl}/api/account`, {
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
      } else {
        // Default account selection
        onAccountChange(accountId);
        setIsOpen(false);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  };

  // Match account by ID or account_number (handle both formats)
  const currentAccount = accounts.find(acc => {
    const accId = acc.id || acc.account_number;
    const accNum = acc.account_number || acc.id;
    if (!currentAccountId || !accId || !accNum) return false;
    return accId === currentAccountId || accNum === currentAccountId ||
           currentAccountId.includes(accId) || currentAccountId.includes(accNum);
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
        title="Select account"
      >
        <span className="account-selector__label">Account:</span>
        <span className="account-selector__value">
          {currentAccount
            ? `${currentAccount.account_number || currentAccount.id} (${currentAccount.source})`
            : currentAccountId || 'Default'}
        </span>
        <span className="account-selector__arrow">{isOpen ? '▲' : '▼'}</span>
      </button>

      {isOpen && (
        <>
          <div className="account-selector__overlay" onClick={() => setIsOpen(false)}></div>
          <div className="account-selector__dropdown">
            <div className="account-selector__header">
              <div className="account-selector__header-row">
                <h3>Select Account</h3>
                <button
                  className="account-selector__close"
                  onClick={() => setIsOpen(false)}
                >
                  ×
                </button>
              </div>
              <p className="account-selector__hint">Independent backends — accounts and positions can exist in parallel.</p>
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
                                (currentAccountId && accountId && currentAccountId.includes(accountId)) ||
                                (currentAccountId && accountNum && currentAccountId.includes(accountNum));
                const displayValue = account.portfolio_value || account.net_liquidation || account.cash || 0;
                const buyingPower = account.buying_power || account.excess_liquidity || 0;
                return (
                <button
                  key={`${account.source}-${accountId}`}
                  className={`account-selector__item ${isActive ? 'account-selector__item--active' : ''}`}
                  onClick={() => handleAccountSelect(accountId || null)}
                >
                  <div className="account-selector__item-main">
                    <span className="account-selector__item-id">{accountNum}</span>
                    <span className="account-selector__item-source" style={{
                      fontSize: '0.85em',
                      opacity: 0.8,
                      marginLeft: '8px'
                    }}>
                      {account.source}
                    </span>
                    {account.status && (
                      <span className={`account-selector__item-status account-selector__item-status--${account.status?.toLowerCase()}`}>
                        {account.status}
                      </span>
                    )}
                  </div>
                  <div className="account-selector__item-details">
                    {displayValue > 0 && (
                      <span>Value: ${displayValue.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
                    )}
                    {buyingPower > 0 && (
                      <span>Buying Power: ${buyingPower.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
                    )}
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
