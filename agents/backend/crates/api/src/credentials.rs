use std::env;
use std::fs;
use std::path::PathBuf;

const SERVICE: &str = "aether";

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aether")
}

fn cred_file(key: &str) -> PathBuf {
    config_dir().join(format!("{}.cred", key))
}

#[derive(Clone, Copy, Debug)]
pub enum CredentialKey {
    FredApiKey,
    FmpApiKey,
    PolygonApiKey,
    AlpacaPaperApiKeyId,
    AlpacaPaperSecretKey,
    AlpacaLiveApiKeyId,
    AlpacaLiveSecretKey,
    TastytradeApiKey,
    TastytradeAccount,
    TaseApiKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlpacaEnvironment {
    Paper,
    Live,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CredentialSource {
    Env,
    Keyring,
    File,
}

impl CredentialSource {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::Keyring => "keyring",
            Self::File => "file",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Resolved Alpaca source credentials and endpoints.
///
/// Keep paper and live identities separate so market-data and future broker
/// adapters do not collapse into a single generic Alpaca credential blob.
pub struct AlpacaCredentialSet {
    pub environment: AlpacaEnvironment,
    pub api_key_id: String,
    pub api_secret_key: String,
    pub trading_base_url: String,
    pub data_base_url: String,
}

impl CredentialKey {
    const fn user(&self) -> &'static str {
        match self {
            Self::FredApiKey => "fred_api_key",
            Self::FmpApiKey => "fmp_api_key",
            Self::PolygonApiKey => "polygon_api_key",
            Self::AlpacaPaperApiKeyId => "alpaca_paper_api_key_id",
            Self::AlpacaPaperSecretKey => "alpaca_paper_secret_key",
            Self::AlpacaLiveApiKeyId => "alpaca_live_api_key_id",
            Self::AlpacaLiveSecretKey => "alpaca_live_secret_key",
            Self::TastytradeApiKey => "tastytrade_api_key",
            Self::TastytradeAccount => "tastytrade_account",
            Self::TaseApiKey => "tase_api_key",
        }
    }

    const fn env_var(&self) -> &'static str {
        match self {
            Self::FredApiKey => "FRED_API_KEY",
            Self::FmpApiKey => "FMP_API_KEY",
            Self::PolygonApiKey => "POLYGON_API_KEY",
            Self::AlpacaPaperApiKeyId => "ALPACA_PAPER_API_KEY_ID",
            Self::AlpacaPaperSecretKey => "ALPACA_PAPER_API_SECRET_KEY",
            Self::AlpacaLiveApiKeyId => "ALPACA_LIVE_API_KEY_ID",
            Self::AlpacaLiveSecretKey => "ALPACA_LIVE_API_SECRET_KEY",
            Self::TastytradeApiKey => "TASTYTRADE_API_KEY",
            Self::TastytradeAccount => "TASTYTRADE_ACCOUNT",
            Self::TaseApiKey => "TASE_API_KEY",
        }
    }

    const fn legacy_env_vars(&self) -> &'static [&'static str] {
        match self {
            // Preserve historical generic Alpaca env vars as the paper/default identity.
            Self::AlpacaPaperApiKeyId => &["ALPACA_API_KEY_ID", "ALPACA_API_KEY"],
            Self::AlpacaPaperSecretKey => &["ALPACA_API_SECRET_KEY", "ALPACA_SECRET_KEY"],
            _ => &[],
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::FredApiKey => "fred",
            Self::FmpApiKey => "fmp",
            Self::PolygonApiKey => "polygon",
            Self::AlpacaPaperApiKeyId => "alpaca-paper-key",
            Self::AlpacaPaperSecretKey => "alpaca-paper-secret",
            Self::AlpacaLiveApiKeyId => "alpaca-live-key",
            Self::AlpacaLiveSecretKey => "alpaca-live-secret",
            Self::TastytradeApiKey => "tastytrade-key",
            Self::TastytradeAccount => "tastytrade-account",
            Self::TaseApiKey => "tase",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::FredApiKey => "FRED API Key",
            Self::FmpApiKey => "FMP API Key",
            Self::PolygonApiKey => "Polygon API Key",
            Self::AlpacaPaperApiKeyId => "Alpaca Paper API Key ID",
            Self::AlpacaPaperSecretKey => "Alpaca Paper API Secret Key",
            Self::AlpacaLiveApiKeyId => "Alpaca Live API Key ID",
            Self::AlpacaLiveSecretKey => "Alpaca Live API Secret Key",
            Self::TastytradeApiKey => "Tastytrade API Key",
            Self::TastytradeAccount => "Tastytrade Account",
            Self::TaseApiKey => "TASE API Key",
        }
    }

    fn file_key(&self) -> String {
        self.user().to_string()
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "fred" => Some(Self::FredApiKey),
            "fmp" => Some(Self::FmpApiKey),
            "polygon" => Some(Self::PolygonApiKey),
            "alpaca-paper-key" | "alpaca_paper_api_key_id" | "alpaca-key" | "alpaca_api_key" => {
                Some(Self::AlpacaPaperApiKeyId)
            }
            "alpaca-paper-secret"
            | "alpaca_paper_secret_key"
            | "alpaca-secret"
            | "alpaca_secret" => Some(Self::AlpacaPaperSecretKey),
            "alpaca-live-key" | "alpaca_live_api_key_id" => Some(Self::AlpacaLiveApiKeyId),
            "alpaca-live-secret" | "alpaca_live_secret_key" => Some(Self::AlpacaLiveSecretKey),
            "tastytrade-key" | "tastytrade_api_key" => Some(Self::TastytradeApiKey),
            "tastytrade-account" | "tastytrade_account" => Some(Self::TastytradeAccount),
            "tase" => Some(Self::TaseApiKey),
            _ => None,
        }
    }
}

pub fn get_credential(key: CredentialKey) -> Option<String> {
    credential_value_and_source(key).map(|(value, _)| value)
}

pub fn credential_source(key: CredentialKey) -> Option<CredentialSource> {
    credential_value_and_source(key).map(|(_, source)| source)
}

fn credential_value_and_source(key: CredentialKey) -> Option<(String, CredentialSource)> {
    // Check environment variables first (highest priority)
    if let Ok(val) = env::var(key.env_var()) {
        if !val.trim().is_empty() {
            return Some((val, CredentialSource::Env));
        }
    }
    for legacy_var in key.legacy_env_vars() {
        if let Ok(val) = env::var(legacy_var) {
            if !val.trim().is_empty() {
                return Some((val, CredentialSource::Env));
            }
        }
    }

    // Keyring is primary storage when feature is enabled
    #[cfg(feature = "keyring")]
    {
        match keyring::Entry::new(SERVICE, key.user()) {
            Ok(entry) => match entry.get_password() {
                Ok(val) if !val.trim().is_empty() => {
                    return Some((val, CredentialSource::Keyring));
                }
                Ok(_) => {} // Empty password, continue to check file for migration
                Err(e) => {
                    tracing::debug!("Keyring get failed for {}: {}", key.user(), e);
                }
            },
            Err(e) => {
                tracing::debug!("Keyring entry creation failed for {}: {}", key.user(), e);
            }
        }
    }

    // File fallback (for non-keyring builds or migration)
    let file = cred_file(&key.file_key());
    if file.exists() {
        if let Ok(content) = fs::read_to_string(&file) {
            let val = content.trim();
            if !val.is_empty() {
                return Some((val.to_string(), CredentialSource::File));
            }
        }
    }

    None
}

pub fn set_credential(key: CredentialKey, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err("Credential value cannot be empty".to_string());
    }

    #[cfg(feature = "keyring")]
    {
        let entry = keyring::Entry::new(SERVICE, key.user())
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

        entry
            .set_password(value)
            .map_err(|e| format!("Failed to store credential in keyring: {}", e))?;

        tracing::debug!("Credential stored in keyring: {}", key.user());
    }

    #[cfg(not(feature = "keyring"))]
    {
        let dir = config_dir();
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {}", e))?;

        let file = cred_file(&key.file_key());
        fs::write(&file, value).map_err(|e| format!("Failed to write credential: {}", e))?;

        tracing::debug!("Credential stored in file: {}", file.display());
    }

    Ok(())
}

pub fn delete_credential(key: CredentialKey) -> Result<(), String> {
    #[cfg(feature = "keyring")]
    {
        match keyring::Entry::new(SERVICE, key.user()) {
            Ok(entry) => {
                if entry.get_password().is_ok() {
                    entry
                        .delete_credential()
                        .map_err(|e| format!("Failed to delete credential from keyring: {}", e))?;
                    tracing::debug!("Credential deleted from keyring: {}", key.user());
                }
            }
            Err(e) => tracing::debug!("Keyring entry not found for {}: {}", key.user(), e),
        }
    }

    #[cfg(not(feature = "keyring"))]
    {
        let file = cred_file(&key.file_key());
        if file.exists() {
            fs::remove_file(&file)
                .map_err(|e| format!("Failed to delete credential file: {}", e))?;
            tracing::debug!("Credential file deleted: {}", file.display());
        }
    }

    Ok(())
}

pub fn fred_api_key() -> Option<String> {
    get_credential(CredentialKey::FredApiKey)
}

pub fn fmp_api_key() -> Option<String> {
    get_credential(CredentialKey::FmpApiKey)
}

pub fn set_fred_api_key(key: &str) -> Result<(), String> {
    set_credential(CredentialKey::FredApiKey, key)
}

pub fn alpaca_paper_api_key_id() -> Option<String> {
    get_credential(CredentialKey::AlpacaPaperApiKeyId)
}

pub fn alpaca_paper_secret_key() -> Option<String> {
    get_credential(CredentialKey::AlpacaPaperSecretKey)
}

pub fn alpaca_live_api_key_id() -> Option<String> {
    get_credential(CredentialKey::AlpacaLiveApiKeyId)
}

pub fn alpaca_live_secret_key() -> Option<String> {
    get_credential(CredentialKey::AlpacaLiveSecretKey)
}

pub fn alpaca_trading_base_url(environment: AlpacaEnvironment) -> String {
    let env_name = match environment {
        AlpacaEnvironment::Paper => "ALPACA_PAPER_BASE_URL",
        AlpacaEnvironment::Live => "ALPACA_LIVE_BASE_URL",
    };
    let default = match environment {
        AlpacaEnvironment::Paper => "https://paper-api.alpaca.markets",
        AlpacaEnvironment::Live => "https://api.alpaca.markets",
    };
    env::var(env_name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
}

pub fn alpaca_data_base_url(environment: AlpacaEnvironment) -> String {
    let env_name = match environment {
        AlpacaEnvironment::Paper => "ALPACA_PAPER_DATA_BASE_URL",
        AlpacaEnvironment::Live => "ALPACA_LIVE_DATA_BASE_URL",
    };
    env::var(env_name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "https://data.alpaca.markets".to_string())
}

pub fn alpaca_credentials(environment: AlpacaEnvironment) -> Option<AlpacaCredentialSet> {
    let (api_key_id, api_secret_key) = match environment {
        AlpacaEnvironment::Paper => (alpaca_paper_api_key_id()?, alpaca_paper_secret_key()?),
        AlpacaEnvironment::Live => (alpaca_live_api_key_id()?, alpaca_live_secret_key()?),
    };

    Some(AlpacaCredentialSet {
        environment,
        api_key_id,
        api_secret_key,
        trading_base_url: alpaca_trading_base_url(environment),
        data_base_url: alpaca_data_base_url(environment),
    })
}

// Backward-compatible paper-default accessors for older callers.
pub fn alpaca_api_key() -> Option<String> {
    alpaca_paper_api_key_id()
}

pub fn alpaca_secret_key() -> Option<String> {
    alpaca_paper_secret_key()
}

pub fn polygon_api_key() -> Option<String> {
    get_credential(CredentialKey::PolygonApiKey)
}

pub fn tastytrade_api_key() -> Option<String> {
    get_credential(CredentialKey::TastytradeApiKey)
}

pub fn tastytrade_account() -> Option<String> {
    get_credential(CredentialKey::TastytradeAccount)
}

pub fn tase_api_key() -> Option<String> {
    get_credential(CredentialKey::TaseApiKey)
}

pub fn set_tase_api_key(key: &str) -> Result<(), String> {
    set_credential(CredentialKey::TaseApiKey, key)
}

pub fn list_credentials() -> Vec<(&'static str, &'static str)> {
    vec![
        ("fred", "FRED (Federal Reserve Economic Data)"),
        ("fmp", "Financial Modeling Prep"),
        ("polygon", "Polygon.io"),
        ("alpaca-paper-key", "Alpaca Paper API Key ID"),
        ("alpaca-paper-secret", "Alpaca Paper API Secret Key"),
        ("alpaca-live-key", "Alpaca Live API Key ID"),
        ("alpaca-live-secret", "Alpaca Live API Secret Key"),
        ("tastytrade-key", "Tastytrade API Key"),
        ("tastytrade-account", "Tastytrade Account Number"),
        ("tase", "TASE (Tel Aviv Stock Exchange)"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paper_aliases_map_to_paper_credentials() {
        assert!(matches!(
            CredentialKey::from_name("alpaca-key"),
            Some(CredentialKey::AlpacaPaperApiKeyId)
        ));
        assert!(matches!(
            CredentialKey::from_name("alpaca-secret"),
            Some(CredentialKey::AlpacaPaperSecretKey)
        ));
    }

    #[test]
    fn alpaca_defaults_match_expected_endpoints() {
        assert_eq!(
            alpaca_trading_base_url(AlpacaEnvironment::Paper),
            "https://paper-api.alpaca.markets"
        );
        assert_eq!(
            alpaca_trading_base_url(AlpacaEnvironment::Live),
            "https://api.alpaca.markets"
        );
        assert_eq!(
            alpaca_data_base_url(AlpacaEnvironment::Paper),
            "https://data.alpaca.markets"
        );
        assert_eq!(
            alpaca_data_base_url(AlpacaEnvironment::Live),
            "https://data.alpaca.markets"
        );
    }
}
