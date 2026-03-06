/**
 * Resolve scraper credentials from 1Password using the formal API (SDK).
 * When OP_SCRAPER_*_SECRET env vars contain op:// refs, resolves them via @1password/sdk;
 * otherwise credentials come from process.env (or shell wrapper that used CLI).
 *
 * Auth: OP_SERVICE_ACCOUNT_TOKEN (automation) or 1Password desktop app (OP_1PASSWORD_ACCOUNT_NAME).
 * See https://developer.1password.com/docs/sdks/
 */

const OP_SECRET_VARS = [
  ['OP_SCRAPER_DISCOUNT_ID_SECRET', 'SCRAPER_DISCOUNT_ID'],
  ['OP_SCRAPER_DISCOUNT_PASSWORD_SECRET', 'SCRAPER_DISCOUNT_PASSWORD'],
  ['OP_SCRAPER_DISCOUNT_NUM_SECRET', 'SCRAPER_DISCOUNT_NUM'],
  ['OP_SCRAPER_LEUMI_USERNAME_SECRET', 'SCRAPER_LEUMI_USERNAME'],
  ['OP_SCRAPER_LEUMI_PASSWORD_SECRET', 'SCRAPER_LEUMI_PASSWORD'],
  ['OP_SCRAPER_HAPOALIM_USER_CODE_SECRET', 'SCRAPER_HAPOALIM_USER_CODE'],
  ['OP_SCRAPER_HAPOALIM_PASSWORD_SECRET', 'SCRAPER_HAPOALIM_PASSWORD'],
  ['OP_SCRAPER_VISACAL_USERNAME_SECRET', 'SCRAPER_VISACAL_USERNAME'],
  ['OP_SCRAPER_VISACAL_PASSWORD_SECRET', 'SCRAPER_VISACAL_PASSWORD'],
  ['OP_SCRAPER_ISRACARD_ID_SECRET', 'SCRAPER_ISRACARD_ID'],
  ['OP_SCRAPER_ISRACARD_CARD_6_DIGITS_SECRET', 'SCRAPER_ISRACARD_CARD_6_DIGITS'],
  ['OP_SCRAPER_ISRACARD_PASSWORD_SECRET', 'SCRAPER_ISRACARD_PASSWORD'],
  ['OP_SCRAPER_MAX_USERNAME_SECRET', 'SCRAPER_MAX_USERNAME'],
  ['OP_SCRAPER_MAX_PASSWORD_SECRET', 'SCRAPER_MAX_PASSWORD'],
  ['OP_SCRAPER_BEINLEUMI_USERNAME_SECRET', 'SCRAPER_BEINLEUMI_USERNAME'],
  ['OP_SCRAPER_BEINLEUMI_PASSWORD_SECRET', 'SCRAPER_BEINLEUMI_PASSWORD'],
  ['OP_SCRAPER_USERNAME_SECRET', 'SCRAPER_USERNAME'],
  ['OP_SCRAPER_PASSWORD_SECRET', 'SCRAPER_PASSWORD'],
  ['OP_SCRAPER_ID_SECRET', 'SCRAPER_ID'],
  ['OP_SCRAPER_NUM_SECRET', 'SCRAPER_NUM'],
  ['OP_SCRAPER_CARD_6_DIGITS_SECRET', 'SCRAPER_CARD_6_DIGITS'],
];

let cachedClient = null;
let cachedSecrets = null;

function isOpRef(value) {
  return typeof value === 'string' && value.startsWith('op://');
}

/**
 * Create 1Password client (service account or desktop app). Returns null if SDK unavailable or auth missing.
 */
async function getClient() {
  if (cachedClient !== null) return cachedClient;
  try {
    const sdk = await import('@1password/sdk');
    const api = sdk.default ?? sdk;
    const token = process.env.OP_SERVICE_ACCOUNT_TOKEN;
    const accountName = process.env.OP_1PASSWORD_ACCOUNT_NAME;

    if (token) {
      cachedClient = await api.createClient({
        auth: token,
        integrationName: 'ib_box_spread_israeli_bank_scrapers',
        integrationVersion: '1.0.0',
      });
    } else if (accountName) {
      const DesktopAuth = api.DesktopAuth ?? sdk.DesktopAuth;
      cachedClient = await api.createClient({
        auth: new DesktopAuth(accountName),
        integrationName: 'ib_box_spread_israeli_bank_scrapers',
        integrationVersion: '1.0.0',
      });
    } else {
      return null;
    }
    return cachedClient;
  } catch {
    return null;
  }
}

/**
 * Resolve all OP_SCRAPER_*_SECRET refs that are op:// and return a plain object of SCRAPER_* keys to values.
 * Prefers existing process.env[outKey] when already set (avoids redundant resolution when started via shell script that used CLI).
 */
async function resolveScraperSecrets() {
  if (cachedSecrets !== null) return cachedSecrets;

  const client = await getClient();
  const result = {};

  for (const [refVar, outKey] of OP_SECRET_VARS) {
    const ref = process.env[refVar];
    const existing = process.env[outKey];
    // Prefer env when already set (e.g. shell script already resolved via CLI) to avoid double resolution
    if (existing != null && String(existing).trim() !== '') {
      result[outKey] = existing;
      continue;
    }
    if (!ref || !isOpRef(ref)) continue;
    if (!client) {
      result[outKey] = existing || '';
      continue;
    }
    try {
      const value = await client.secrets.resolve(ref);
      result[outKey] = typeof value === 'string' ? value : String(value ?? '');
    } catch {
      result[outKey] = existing || '';
    }
  }

  cachedSecrets = result;
  return result;
}

/**
 * Apply resolved secrets into process.env so getCompanyCredentials() sees them.
 * Call once at startup (CLI or server) when using SDK path.
 */
async function applyScraperSecretsToEnv() {
  const secrets = await resolveScraperSecrets();
  for (const [key, value] of Object.entries(secrets)) {
    if (value !== undefined && value !== '') process.env[key] = value;
  }
}

export { resolveScraperSecrets, applyScraperSecretsToEnv, getClient, isOpRef };
