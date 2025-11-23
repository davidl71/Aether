import { useEffect, useState } from 'react';
import type { BankAccount } from '../components/BankAccountsPanel';
import { getServiceUrl } from '../config/ports';

interface UseBankAccountsResult {
  accounts: BankAccount[];
  loading: boolean;
  error: string | null;
}

export function useBankAccounts(serviceUrl?: string): UseBankAccountsResult {
  const defaultServiceUrl = serviceUrl || getServiceUrl('discountBank');
  const [accounts, setAccounts] = useState<BankAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchAccounts = async () => {
      try {
        setLoading(true);
        const url = serviceUrl || getServiceUrl('discountBank');
        const response = await fetch(`${url}/api/bank-accounts`, {
          headers: { 'cache-control': 'no-cache' }
        });

        if (!response.ok) {
          throw new Error(`Failed to fetch bank accounts: ${response.status}`);
        }

        const data = await response.json();
        setAccounts(data.accounts || []);
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

  return { accounts, loading, error };
}
