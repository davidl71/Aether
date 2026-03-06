#!/usr/bin/env node
/**
 * CLI: Run israeli-bank-scrapers for one company and write results to the ledger.
 *
 * Env:
 *   SCRAPER_COMPANY_ID   - Company (discount, leumi, hapoalim, ...)
 *   SCRAPER_START_DATE   - YYYY-MM-DD (default: 1 year ago)
 *   LEDGER_DATABASE_PATH - Ledger SQLite path (optional)
 *
 * Credentials: set SCRAPER_* env vars, or 1Password op:// refs in OP_SCRAPER_*_SECRET
 * (resolved via 1Password SDK when OP_SERVICE_ACCOUNT_TOKEN or OP_1PASSWORD_ACCOUNT_NAME is set).
 */

import { createScraper } from 'israeli-bank-scrapers';
import { writeScrapeResultToLedger, getLedgerDbPathSync } from './ledger-writer.js';
import { applyScraperSecretsToEnv } from './op-secrets.js';

// Resolve 1Password op:// refs into env before reading credentials
await applyScraperSecretsToEnv();

const COMPANY_ID = process.env.SCRAPER_COMPANY_ID || 'discount';
const startDate = process.env.SCRAPER_START_DATE
  ? new Date(process.env.SCRAPER_START_DATE)
  : new Date(Date.now() - 365 * 24 * 60 * 60 * 1000);

function getCredentials() {
  const c = COMPANY_ID.toLowerCase();
  if (c === 'discount') {
    return {
      id: process.env.SCRAPER_DISCOUNT_ID || process.env.SCRAPER_ID,
      password: process.env.SCRAPER_DISCOUNT_PASSWORD || process.env.SCRAPER_PASSWORD,
      num: process.env.SCRAPER_DISCOUNT_NUM || process.env.SCRAPER_NUM,
    };
  }
  if (c === 'leumi') {
    return {
      username: process.env.SCRAPER_LEUMI_USERNAME || process.env.SCRAPER_USERNAME,
      password: process.env.SCRAPER_LEUMI_PASSWORD || process.env.SCRAPER_PASSWORD,
    };
  }
  if (c === 'hapoalim') {
    return {
      userCode: process.env.SCRAPER_HAPOALIM_USER_CODE || process.env.SCRAPER_USERNAME,
      password: process.env.SCRAPER_HAPOALIM_PASSWORD || process.env.SCRAPER_PASSWORD,
    };
  }
  if (c === 'visacal') {
    return {
      username: process.env.SCRAPER_VISACAL_USERNAME || process.env.SCRAPER_USERNAME,
      password: process.env.SCRAPER_VISACAL_PASSWORD || process.env.SCRAPER_PASSWORD,
    };
  }
  if (c === 'isracard') {
    return {
      id: process.env.SCRAPER_ISRACARD_ID || process.env.SCRAPER_ID,
      card6Digits: process.env.SCRAPER_ISRACARD_CARD_6_DIGITS || process.env.SCRAPER_CARD_6_DIGITS,
      password: process.env.SCRAPER_ISRACARD_PASSWORD || process.env.SCRAPER_PASSWORD,
    };
  }
  if (c === 'max') {
    return {
      username: process.env.SCRAPER_MAX_USERNAME || process.env.SCRAPER_USERNAME,
      password: process.env.SCRAPER_MAX_PASSWORD || process.env.SCRAPER_PASSWORD,
    };
  }
  if (c === 'beinleumi') {
    return {
      username: process.env.SCRAPER_BEINLEUMI_USERNAME || process.env.SCRAPER_USERNAME,
      password: process.env.SCRAPER_BEINLEUMI_PASSWORD || process.env.SCRAPER_PASSWORD,
    };
  }
  // Generic fallback
  return {
    username: process.env.SCRAPER_USERNAME,
    password: process.env.SCRAPER_PASSWORD,
    id: process.env.SCRAPER_ID,
    num: process.env.SCRAPER_NUM,
    card6Digits: process.env.SCRAPER_CARD_6_DIGITS,
  };
}

async function main() {
  const credentials = getCredentials();
  if (!credentials.password && !credentials.id) {
    console.error('Set credentials via env (e.g. SCRAPER_PASSWORD, SCRAPER_USERNAME or company-specific vars).');
    process.exit(1);
  }

  const options = {
    companyId: COMPANY_ID,
    startDate,
    combineInstallments: false,
    showBrowser: process.env.SCRAPER_SHOW_BROWSER === '1',
  };

  console.error(`Scraping ${COMPANY_ID} from ${startDate.toISOString().slice(0, 10)}...`);
  const scraper = createScraper(options);
  const result = await scraper.scrape(credentials);

  if (!result.success) {
    console.error('Scrape failed:', result.errorType, result.errorMessage);
    process.exit(2);
  }

  const ledgerPath = process.env.LEDGER_DATABASE_PATH || getLedgerDbPathSync();
  const out = writeScrapeResultToLedger(result, COMPANY_ID, ledgerPath);
  if (out.error) {
    console.error('Ledger write error:', out.error);
    process.exit(3);
  }

  console.error(`Wrote ${out.written} transactions from ${out.accounts} account(s) to ${ledgerPath}`);
  console.log(JSON.stringify({ success: true, written: out.written, accounts: out.accounts }));
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
