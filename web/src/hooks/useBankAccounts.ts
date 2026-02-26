import type { BankAccount } from '../components/BankAccountsPanel';
import { getServiceUrl } from '../config/ports';
import { useFetchJSON } from './useFetchJSON';

interface UseBankAccountsResult {
  accounts: BankAccount[];
  loading: boolean;
  error: string | null;
}

export function useBankAccounts(serviceUrl?: string): UseBankAccountsResult {
  const url = `${serviceUrl || getServiceUrl('discountBank')}/api/bank-accounts`;

  const { data, isLoading, error } = useFetchJSON<BankAccount[]>(
    url,
    (json) => (json as { accounts?: BankAccount[] }).accounts ?? [],
    { pollIntervalMs: 30_000 },
  );

  return { accounts: data ?? [], loading: isLoading, error };
}
