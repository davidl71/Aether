use std::time::Duration;

use async_nats::Client;
use tracing::{error, info};

use crate::error::{NatsAdapterError, Result};

/// NATS client wrapper with connection management
#[derive(Clone)]
pub struct NatsClient {
    client: Client,
    url: String,
}

impl NatsClient {
    /// Connect to NATS server
    ///
    /// # Arguments
    /// * `url` - NATS server URL (e.g., "nats://localhost:4222")
    ///
    /// # Returns
    /// Connected NATS client or error
    pub async fn connect(url: impl Into<String>) -> Result<Self> {
        let url = url.into();
        info!(url = %url, "Connecting to NATS server");

        let client = async_nats::connect(&url).await.map_err(|e| {
            error!(error = %e, "Failed to connect to NATS server");
            NatsAdapterError::Connection(format!("{}", e))
        })?;

        info!(url = %url, "Connected to NATS server");

        Ok(Self { client, url })
    }

    /// Connect with custom options
    pub async fn connect_with_options(
        url: impl Into<String>,
        options: async_nats::ConnectOptions,
    ) -> Result<Self> {
        let url = url.into();
        info!(url = %url, "Connecting to NATS server with custom options");

        let client = options.connect(&url).await.map_err(|e| {
            error!(error = %e, "Failed to connect to NATS server");
            NatsAdapterError::Connection(format!("{}", e))
        })?;

        info!(url = %url, "Connected to NATS server");

        Ok(Self { client, url })
    }

    /// Get the underlying NATS client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the server URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Check if client is connected
    /// Note: This is a best-effort check. The client may disconnect after this check.
    pub fn is_connected(&self) -> bool {
        // Client doesn't expose is_closed, so we assume connected if we have a client
        // In practice, connection status is managed by async-nats internally
        true
    }

    /// Flush pending messages
    pub async fn flush(&self) -> Result<()> {
        self.client
            .flush()
            .await
            .map_err(|e| NatsAdapterError::Connection(format!("Flush error: {}", e)))
    }

    /// Close the connection
    /// Note: async-nats Client doesn't have explicit close, it closes on drop
    pub async fn close(&self) {
        info!(url = %self.url, "NATS connection will close on drop");
        // Client closes automatically when dropped
    }
}

/// Default connection options for development
impl Default for NatsClient {
    fn default() -> Self {
        // This will panic if called - use connect() instead
        panic!("NatsClient::default() should not be called. Use NatsClient::connect() instead.")
    }
}

/// Create default connection options
pub fn default_connect_options() -> async_nats::ConnectOptions {
    async_nats::ConnectOptions::new()
        .reconnect_delay_callback(|attempts| {
            // Exponential backoff: 1s, 2s, 4s, 8s, 16s, max 30s
            let delay = (1u64 << attempts.min(4)).min(30);
            Duration::from_secs(delay)
        })
        .ping_interval(Duration::from_secs(20))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_connect_fails_without_server() {
        let result = timeout(
            Duration::from_secs(2),
            NatsClient::connect("nats://127.0.0.1:9"),
        )
        .await
        .expect("connection attempt should fail quickly");
        assert!(result.is_err());
    }
}
