/**
 * HTTP server: POST /scrape triggers scrape and ledger write; GET /api/health for liveness.
 * Credentials from env or 1Password SDK (op:// refs in OP_SCRAPER_*_SECRET). Port from PORT or 8010.
 */

import express from 'express';
import { createScraper } from 'israeli-bank-scrapers';
import { writeScrapeResultToLedger, getLedgerDbPathSync } from './ledger-writer.js';
import { applyScraperSecretsToEnv } from './op-secrets.js';

const app = express();
app.use(express.json());

const PORT = parseInt(process.env.PORT || '8010', 10);

// Resolve 1Password op:// refs into env at startup (when SDK and OP_SERVICE_ACCOUNT_TOKEN or desktop app available)
await applyScraperSecretsToEnv();

function getCompanyCredentials(companyId) {
  const c = (companyId || 'discount').toLowerCase();
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
  return {
    username: process.env.SCRAPER_USERNAME,
    password: process.env.SCRAPER_PASSWORD,
    id: process.env.SCRAPER_ID,
    num: process.env.SCRAPER_NUM,
    card6Digits: process.env.SCRAPER_CARD_6_DIGITS,
  };
}

app.get('/api/health', (req, res) => {
  const ledgerPath = getLedgerDbPathSync();
  res.json({
    status: 'ok',
    service: 'israeli_bank_scrapers',
    ledger_path: ledgerPath,
    port: PORT,
  });
});

app.post('/scrape', async (req, res) => {
  const companyId = req.body?.companyId || process.env.SCRAPER_COMPANY_ID || 'discount';
  const startDateStr = req.body?.startDate || process.env.SCRAPER_START_DATE;
  const startDate = startDateStr ? new Date(startDateStr) : new Date(Date.now() - 365 * 24 * 60 * 60 * 1000);

  const credentials = getCompanyCredentials(companyId);
  if (!credentials.password && !credentials.id) {
    return res.status(400).json({
      success: false,
      error: 'Missing credentials. Set SCRAPER_* env vars (see README).',
    });
  }

  try {
    const scraper = createScraper({
      companyId,
      startDate,
      combineInstallments: false,
      showBrowser: false,
    });
    const result = await scraper.scrape(credentials);

    if (!result.success) {
      return res.status(502).json({
        success: false,
        errorType: result.errorType,
        errorMessage: result.errorMessage,
      });
    }

    const ledgerPath = process.env.LEDGER_DATABASE_PATH || getLedgerDbPathSync();
    const out = writeScrapeResultToLedger(result, companyId, ledgerPath);
    if (out.error) {
      return res.status(500).json({ success: false, error: out.error });
    }

    res.json({
      success: true,
      written: out.written,
      accounts: out.accounts,
      ledger_path: ledgerPath,
    });
  } catch (e) {
    res.status(500).json({ success: false, error: e.message });
  }
});

app.listen(PORT, () => {
  console.error(`israeli-bank-scrapers service listening on port ${PORT}`);
});
