/**
 * Maps israeli-bank-scrapers result to ledger transactions and writes to SQLite.
 * Account path format: Assets:Bank:{BankName}:{accountNumber}
 * Double-entry: each scrape txn becomes one ledger txn with bank + Equity:Capital postings.
 */

import Database from 'better-sqlite3';
import { randomUUID } from 'crypto';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

/** CompanyId to ledger bank name (segment in Assets:Bank:{name}:...) */
const COMPANY_TO_BANK_NAME = {
  hapoalim: 'Hapoalim',
  leumi: 'Leumi',
  discount: 'Discount',
  mercantile: 'Mercantile',
  mizrahi: 'Mizrahi',
  otsarHahayal: 'OtsarHahayal',
  visaCal: 'VisaCal',
  max: 'Max',
  isracard: 'Isracard',
  amex: 'Amex',
  union: 'Union',
  beinleumi: 'Beinleumi',
  massad: 'Massad',
  yahav: 'Yahav',
  beyhadBishvilha: 'BeyhadBishvilha',
  onezero: 'OneZero',
  behatsdaa: 'Behatsdaa',
};

/**
 * Resolve ledger database path (same logic as Python discount_bank_service / ledger_queries).
 * @param {string} [projectRoot] - Project root (default: parent of services/)
 * @returns {string}
 */
export function getLedgerDbPathSync(projectRoot) {
  // Default: project root is parent of services/ (so 3 levels up from this file in src/)
  const root = projectRoot || path.resolve(__dirname, '../../..');
  const envPath = process.env.LEDGER_DATABASE_PATH;
  if (envPath) {
    const expanded = envPath.startsWith('~') ? path.join(process.env.HOME || '', envPath.slice(1)) : envPath;
    return path.resolve(expanded);
  }

  const candidates = [
    path.join(root, 'ledger.db'),
    path.join(root, 'agents', 'backend', 'ledger.db'),
    path.join(process.env.HOME || '', '.ledger', 'ledger.db'),
  ];
  for (const p of candidates) {
    try {
      if (fs.existsSync(p)) return p;
    } catch {
      // ignore
    }
  }
  return candidates[0];
}

/**
 * Ensure transactions table exists (same schema as Rust/Python).
 * @param {import('better-sqlite3').Database} db
 */
function ensureSchema(db) {
  db.exec(`
    CREATE TABLE IF NOT EXISTS transactions (
      id TEXT PRIMARY KEY,
      date TEXT NOT NULL,
      description TEXT NOT NULL,
      cleared INTEGER NOT NULL DEFAULT 1,
      transaction_json TEXT NOT NULL,
      account_paths TEXT NOT NULL,
      created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date);
    CREATE INDEX IF NOT EXISTS idx_transactions_accounts ON transactions(account_paths);
  `);
}

/**
 * Map one scraped transaction to one ledger transaction (double-entry).
 * @param {object} txn - Scraped txn: { date, processedDate, chargedAmount, originalCurrency, description, memo }
 * @param {string} bankAccountPath - e.g. Assets:Bank:Discount:123456
 * @returns {object} - { transactionJson, account_paths }
 */
function scrapedTxnToLedgerTxn(txn, bankAccountPath) {
  const id = randomUUID();
  const dateStr = txn.processedDate || txn.date || new Date().toISOString();
  const date = dateStr.slice(0, 19).replace('T', ' ');
  const currency = txn.originalCurrency || 'ILS';
  const rawAmount = Number(txn.chargedAmount);
  const amount = Math.abs(rawAmount);
  const isCredit = rawAmount >= 0; // credit = money in

  const description = [txn.description, txn.memo].filter(Boolean).join(' — ') || 'Israeli bank transaction';

  const equityPath = 'Equity:Capital';
  let postings;
  if (isCredit) {
    postings = [
      { account: bankAccountPath, amount: { amount: String(amount), currency } },
      { account: equityPath, amount: { amount: String(-amount), currency } },
    ];
  } else {
    postings = [
      { account: equityPath, amount: { amount: String(amount), currency } },
      { account: bankAccountPath, amount: { amount: String(-amount), currency } },
    ];
  }

  const transactionJson = {
    id,
    date,
    description,
    cleared: true,
    postings,
    metadata: {
      source: 'israeli_bank_scrapers',
      reference: txn.description || '',
      identifier: txn.identifier,
    },
  };

  const account_paths = [bankAccountPath, equityPath].join('|');
  return { transactionJson, account_paths };
}

/**
 * Write scraped accounts and transactions to the ledger DB.
 * @param {object} scrapeResult - Result from israeli-bank-scrapers: { success, accounts: [{ accountNumber, balance, txns }] }
 * @param {string} companyId - e.g. 'discount', 'leumi'
 * @param {string} [ledgerPath] - Override ledger DB path
 * @returns {{ written: number, accounts: number, error?: string }}
 */
export function writeScrapeResultToLedger(scrapeResult, companyId, ledgerPath) {
  const dbPath = ledgerPath || getLedgerDbPathSync();
  const db = new Database(dbPath);
  ensureSchema(db);

  const insert = db.prepare(`
    INSERT OR IGNORE INTO transactions (id, date, description, cleared, transaction_json, account_paths)
    VALUES (@id, @date, @description, @cleared, @transaction_json, @account_paths)
  `);

  const bankName = COMPANY_TO_BANK_NAME[companyId] || companyId;
  let written = 0;

  try {
    if (!scrapeResult.success || !Array.isArray(scrapeResult.accounts)) {
      return { written: 0, accounts: 0, error: scrapeResult.errorMessage || 'Scrape failed or no accounts' };
    }

    const insertMany = db.transaction(() => {
      for (const acc of scrapeResult.accounts) {
        const accountNumber = (acc.accountNumber || '').replace(/\s/g, '') || 'unknown';
        const bankAccountPath = `Assets:Bank:${bankName}:${accountNumber}`;

        for (const txn of acc.txns || []) {
          const { transactionJson, account_paths } = scrapedTxnToLedgerTxn(txn, bankAccountPath);
          try {
            insert.run({
              id: transactionJson.id,
              date: transactionJson.date,
              description: transactionJson.description,
              cleared: 1,
              transaction_json: JSON.stringify(transactionJson),
              account_paths,
            });
            written++;
          } catch (e) {
            if (!e.message?.includes('UNIQUE')) throw e;
          }
        }
      }
    });

    insertMany();
    return { written, accounts: scrapeResult.accounts.length };
  } finally {
    db.close();
  }
}
