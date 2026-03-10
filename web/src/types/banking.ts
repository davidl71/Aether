export interface BankAccount {
  account_path: string;
  account_name: string;
  bank_name: string | null;
  account_number: string | null;
  balance: number;
  currency: string;
  balances_by_currency?: Record<string, number> | null;
  is_mixed_currency?: boolean;
  balance_date: string | null;
  credit_rate: number | null;
  debit_rate: number | null;
}

export interface BankAccountsResponse {
  accounts: BankAccount[];
  total_count: number;
}
