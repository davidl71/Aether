import { useMemo } from 'react';
import { useBankAccounts } from '../hooks/useBankAccounts';
import type { BankAccount } from '../types/banking';

export type { BankAccount } from '../types/banking';

interface BankAccountsPanelProps {
  serviceUrl?: string;
}

export function BankAccountsPanel({ serviceUrl }: BankAccountsPanelProps) {
  const { accounts, loading, error } = useBankAccounts(serviceUrl);

  const formatBalance = (balance: number, currency: string): string => {
    const sign = balance >= 0 ? '' : '-';
    const absBalance = Math.abs(balance);
    return `${sign}${absBalance.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })} ${currency}`;
  };

  const formatRate = (rate: number | null): string => {
    if (rate === null) return '—';
    return `${(rate * 100).toFixed(2)}%`;
  };

  const totalBalanceLabel = useMemo(() => {
    const totalsByCurrency = new Map<string, number>();
    accounts.forEach((account) => {
      if (account.is_mixed_currency && account.balances_by_currency) {
        Object.entries(account.balances_by_currency).forEach(([currency, amount]) => {
          totalsByCurrency.set(currency, (totalsByCurrency.get(currency) || 0) + amount);
        });
        return;
      }
      totalsByCurrency.set(
        account.currency,
        (totalsByCurrency.get(account.currency) || 0) + account.balance
      );
    });

    if (totalsByCurrency.size === 0) {
      return formatBalance(0, 'USD');
    }
    if (totalsByCurrency.size === 1) {
      const [currency, amount] = Array.from(totalsByCurrency.entries())[0];
      return formatBalance(amount, currency);
    }
    return Array.from(totalsByCurrency.entries())
      .map(([currency, amount]) => formatBalance(amount, currency))
      .join(' | ');
  }, [accounts]);

  if (loading) {
    return (
      <div className="panel">
        <h2>Bank Accounts</h2>
        <p>Loading bank accounts...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="panel">
        <h2>Bank Accounts</h2>
        <p className="error">Error loading bank accounts: {error}</p>
        <p className="text-muted">Make sure the bank-account service is running and reachable.</p>
      </div>
    );
  }

  if (accounts.length === 0) {
    return (
      <div className="panel">
        <h2>Bank Accounts</h2>
        <p className="text-muted">No bank accounts found in ledger</p>
      </div>
    );
  }

  return (
    <div className="panel">
      <h2>Bank Accounts</h2>
      <div className="bank-accounts-summary">
        <div className="summary-item">
          <span className="label">Total Accounts:</span>
          <span className="value">{accounts.length}</span>
        </div>
        <div className="summary-item">
          <span className="label">Total Balance:</span>
          <span className="value">{totalBalanceLabel}</span>
        </div>
      </div>

      <table className="data-table">
        <thead>
          <tr>
            <th>Account</th>
            <th>Bank</th>
            <th>Balance</th>
            <th>Currency</th>
            <th>Credit Rate</th>
            <th>Debit Rate</th>
          </tr>
        </thead>
        <tbody>
          {accounts.map((account) => (
            <tr key={account.account_path}>
              <td>
                <div className="account-info">
                  <div className="account-name">{account.account_name}</div>
                  {account.account_number && (
                    <div className="account-number">{account.account_number}</div>
                  )}
                </div>
              </td>
              <td>{account.bank_name || '—'}</td>
              <td className={account.balance >= 0 ? 'positive' : 'negative'}>
                {account.is_mixed_currency && account.balances_by_currency
                  ? Object.entries(account.balances_by_currency)
                      .map(([currency, amount]) => formatBalance(amount, currency))
                      .join(' | ')
                  : formatBalance(account.balance, account.currency)}
              </td>
              <td>{account.is_mixed_currency ? 'MULTI' : account.currency}</td>
              <td>{formatRate(account.credit_rate)}</td>
              <td>{formatRate(account.debit_rate)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
