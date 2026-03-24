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
    AlpacaApiKey,
    AlpacaSecretKey,
    TastytradeApiKey,
    TastytradeAccount,
    TaseApiKey,
}

impl CredentialKey {
    const fn user(&self) -> &'static str {
        match self {
            Self::FredApiKey => "fred_api_key",
            Self::FmpApiKey => "fmp_api_key",
            Self::PolygonApiKey => "polygon_api_key",
            Self::AlpacaApiKey => "alpaca_api_key",
            Self::AlpacaSecretKey => "alpaca_secret_key",
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
            Self::AlpacaApiKey => "ALPACA_API_KEY",
            Self::AlpacaSecretKey => "ALPACA_SECRET_KEY",
            Self::TastytradeApiKey => "TASTYTRADE_API_KEY",
            Self::TastytradeAccount => "TASTYTRADE_ACCOUNT",
            Self::TaseApiKey => "TASE_API_KEY",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::FredApiKey => "fred",
            Self::FmpApiKey => "fmp",
            Self::PolygonApiKey => "polygon",
            Self::AlpacaApiKey => "alpaca-key",
            Self::AlpacaSecretKey => "alpaca-secret",
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
            Self::AlpacaApiKey => "Alpaca API Key",
            Self::AlpacaSecretKey => "Alpaca Secret Key",
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
            "alpaca-key" | "alpaca_api_key" => Some(Self::AlpacaApiKey),
            "alpaca-secret" | "alpaca_secret" => Some(Self::AlpacaSecretKey),
            "tastytrade-key" | "tastytrade_api_key" => Some(Self::TastytradeApiKey),
            "tastytrade-account" | "tastytrade_account" => Some(Self::TastytradeAccount),
            "tase" => Some(Self::TaseApiKey),
            _ => None,
        }
    }
}

pub fn get_credential(key: CredentialKey) -> Option<String> {
    if let Ok(val) = env::var(key.env_var()) {
        if !val.trim().is_empty() {
            return Some(val);
        }
    }

    #[cfg(feature = "keyring")]
    {
        if let Ok(entry) = keyring::Entry::new(SERVICE, key.user()) {
            if let Ok(val) = entry.get_password() {
                if !val.trim().is_empty() {
                    return Some(val);
                }
            }
        }
    }

    let file = cred_file(&key.file_key());
    if file.exists() {
        if let Ok(content) = fs::read_to_string(&file) {
            let val = content.trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }

    None
}

pub fn set_credential(key: CredentialKey, value: &str) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {}", e))?;

    let file = cred_file(&key.file_key());
    fs::write(&file, value).map_err(|e| format!("Failed to write credential: {}", e))?;

    #[cfg(feature = "keyring")]
    {
        if let Ok(entry) = keyring::Entry::new(SERVICE, key.user()) {
            let _ = entry.set_password(value);
        }
    }

    Ok(())
}

pub fn delete_credential(key: CredentialKey) -> Result<(), String> {
    let file = cred_file(&key.file_key());
    if file.exists() {
        fs::remove_file(&file).map_err(|e| format!("Failed to delete credential: {}", e))?;
    }

    #[cfg(feature = "keyring")]
    {
        if let Ok(entry) = keyring::Entry::new(SERVICE, key.user()) {
            let _ = entry.delete_credential();
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

pub fn alpaca_api_key() -> Option<String> {
    get_credential(CredentialKey::AlpacaApiKey)
}

pub fn alpaca_secret_key() -> Option<String> {
    get_credential(CredentialKey::AlpacaSecretKey)
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
        ("alpaca-key", "Alpaca API Key"),
        ("alpaca-secret", "Alpaca Secret Key"),
        ("tastytrade-key", "Tastytrade API Key"),
        ("tastytrade-account", "Tastytrade Account Number"),
        ("tase", "TASE (Tel Aviv Stock Exchange)"),
    ]
}
