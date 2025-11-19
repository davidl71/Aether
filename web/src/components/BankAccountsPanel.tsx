import { useEffect, useState } from 'react';

export interface BankAccount {
  account_path: string;
  account_name: string;
  bank_name: string | null;
  account_number: string | null;
  balance: number;
  currency: string;
  balance_date: string | null;
  credit_rate: number | null;
  debit_rate: number | null;
}

interface BankAccountsResponse {
  accounts: BankAccount[];
  total_count: number;
}

interface BankAccountsPanelProps {
  serviceUrl?: string;
}

export function BankAccountsPanel({ serviceUrl = 'http://localhost:8003' }: BankAccountsPanelProps) {
  const [accounts, setAccounts] = useState<BankAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchAccounts = async () => {
      try {
        setLoading(true);
        const response = await fetch(`${serviceUrl}/api/bank-accounts`, {
          headers: { 'cache-control': 'no-cache' }
        });

        if (!response.ok) {
          throw new Error(`Failed to fetch bank accounts: ${response.status}`);
        }

        const data = (await response.json()) as BankAccountsResponse;
        setAccounts(data.accounts);
        setError(null);
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Unknown error';
        setError(errorMessage);
        setAccounts([]);
      } finally {
        setLoading(false);
      }
    };

    fetchAccounts();

    // Refresh every 30 seconds
    const interval = setInterval(fetchAccounts, 30000);
    return () => clearInterval(interval);
  }, [serviceUrl]);

  const formatBalance = (balance: number, currency: string): string => {
    const sign = balance >= 0 ? '' : '-';
    const absBalance = Math.abs(balance);
    return `${sign}${absBalance.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })} ${currency}`;
  };

  const formatRate = (rate: number | null): string => {
    if (rate === null) return '—';
    return `${(rate * 100).toFixed(2)}%`;
  };

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
        <p className="text-muted">Make sure the Discount Bank service is running on port 8003</p>
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

  const totalBalance = accounts.reduce((sum, acc) => sum + acc.balance, 0);
  const primaryCurrency = accounts[0]?.currency || 'USD';

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
          <span className="value">{formatBalance(totalBalance, primaryCurrency)}</span>
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
                {formatBalance(account.balance, account.currency)}
              </td>
              <td>{account.currency}</td>
              <td>{formatRate(account.credit_rate)}</td>
              <td>{formatRate(account.debit_rate)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
