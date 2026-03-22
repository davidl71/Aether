use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::{SinkExt, StreamExt};
use tokio::{sync::mpsc, time::timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, warn};
use url::Url;

use crate::{MarketDataEvent, MarketDataEventBuilder, MarketDataSource};

const DEFAULT_WS_URL: &str = "wss://socket.polygon.io";

pub struct PolygonWsMarketDataSource {
    api_key: String,
    base_url: Url,
    symbols: Arc<Vec<String>>,
    receiver: Arc<tokio::sync::Mutex<Option<mpsc::Receiver<MarketDataEvent>>>>,
}

impl PolygonWsMarketDataSource {
    pub fn new<I, S>(
        symbols: I,
        api_key: impl Into<String>,
        base_url: Option<&str>,
    ) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let symbols_vec: Vec<String> = symbols.into_iter().map(Into::into).collect();
        anyhow::ensure!(
            !symbols_vec.is_empty(),
            "at least one symbol must be configured"
        );

        let base = base_url.unwrap_or(DEFAULT_WS_URL);
        let base_url = Url::parse(base)
            .map_err(|err| anyhow::anyhow!("invalid polygon ws base url {base}: {err}"))?;

        Ok(Self {
            api_key: api_key.into(),
            base_url,
            symbols: Arc::new(symbols_vec),
            receiver: Arc::new(tokio::sync::Mutex::new(None)),
        })
    }

    pub async fn run(self) {
        let (ws_stream, _) = match connect_async(self.base_url.as_str()).await {
            Ok(s) => s,
            Err(e) => {
                warn!(error = %e, "failed to connect to polygon ws");
                return;
            }
        };

        let (mut write, mut read) = ws_stream.split();

        let auth_msg = serde_json::json!({
            "action": "auth",
            "params": &self.api_key
        });
        if let Err(e) = write.send(Message::Text(auth_msg.to_string().into())).await {
            warn!(error = %e, "failed to send auth");
            return;
        }

        let first_msg = match timeout(std::time::Duration::from_secs(5), read.next()).await {
            Ok(Some(Ok(msg))) => msg,
            Ok(Some(Err(e))) => {
                warn!(error = %e, "auth read error");
                return;
            }
            _ => {
                warn!("auth timeout or closed");
                return;
            }
        };

        let auth_resp: AuthResponse = match serde_json::from_slice(&first_msg.into_data()) {
            Ok(r) => r,
            Err(e) => {
                warn!(error = %e, "failed to parse auth response");
                return;
            }
        };

        if auth_resp.status != "authenticated" {
            warn!(status = %auth_resp.status, "polygon ws auth failed");
            return;
        }

        let subscribe_params: String = self
            .symbols
            .iter()
            .map(|s| format!("Q.{}", s))
            .collect::<Vec<_>>()
            .join(",");

        let subscribe_msg = serde_json::json!({
            "action": "subscribe",
            "params": subscribe_params
        });
        if let Err(e) = write
            .send(Message::Text(subscribe_msg.to_string().into()))
            .await
        {
            warn!(error = %e, "failed to subscribe");
            return;
        }

        debug!(symbols = ?*self.symbols, "polygon ws subscribed");

        let (tx, rx) = mpsc::channel::<MarketDataEvent>(100);
        {
            let mut guard = self.receiver.lock().await;
            *guard = Some(rx);
        }

        let symbols = self.symbols.clone();
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = process_ws_messages(&text, &symbols, &tx).await {
                        warn!(error = %e, "ws message error");
                    }
                }
                Ok(Message::Ping(data)) => {
                    if let Err(e) = write.send(Message::Pong(data)).await {
                        warn!(error = %e, "pong failed");
                        break;
                    }
                }
                Ok(Message::Close(_)) | Err(_) => {
                    break;
                }
                _ => {}
            }
        }

        debug!("polygon ws loop ended");
    }
}

async fn process_ws_messages(
    data: &str,
    symbols: &[String],
    tx: &mpsc::Sender<MarketDataEvent>,
) -> anyhow::Result<()> {
    let msgs: Vec<WsMessage> =
        serde_json::from_str(data).map_err(|e| anyhow::anyhow!("parse error: {e}"))?;

    for msg in msgs {
        if msg.event != "Q" {
            continue;
        }
        if !symbols.iter().any(|s| s == &msg.symbol) {
            continue;
        }

        let bid = msg.b.map(|b| b[0]).unwrap_or(0.0);
        let ask = msg.a.map(|a| a[0]).unwrap_or(0.0);

        if bid <= 0.0 || ask <= 0.0 {
            debug!(
                "skip non-positive quote for {}: bid={bid}, ask={ask}",
                msg.symbol
            );
            continue;
        }

        let timestamp: DateTime<Utc> = msg
            .timestamp
            .map(|ts| {
                if ts > 1_000_000_000_000_000 {
                    let secs = ts / 1_000_000_000;
                    let nanos = (ts % 1_000_000_000) as u32;
                    DateTime::from_timestamp(secs, nanos).unwrap_or_else(Utc::now)
                } else {
                    DateTime::from_timestamp(ts / 1_000, 0).unwrap_or_else(Utc::now)
                }
            })
            .unwrap_or_else(Utc::now);

        let event = MarketDataEventBuilder::default()
            .symbol(msg.symbol.clone())
            .bid(bid)
            .ask(ask)
            .timestamp(timestamp)
            .build()?;

        if tx.send(event).await.is_err() {
            break;
        }
    }
    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct AuthResponse {
    #[serde(rename = "status")]
    status: String,
}

#[derive(Debug, serde::Deserialize)]
struct WsMessage {
    #[serde(rename = "ev")]
    event: String,
    #[serde(rename = "sym")]
    symbol: String,
    #[serde(rename = "b")]
    b: Option<[f64; 4]>,
    #[serde(rename = "a")]
    a: Option<[f64; 4]>,
    #[serde(rename = "t")]
    timestamp: Option<i64>,
}

#[async_trait]
impl MarketDataSource for PolygonWsMarketDataSource {
    async fn next(&self) -> anyhow::Result<MarketDataEvent> {
        let mut guard = self.receiver.lock().await;
        let rx = guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("websocket not connected"))?;
        rx.recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("websocket channel closed"))
    }
}

impl Clone for PolygonWsMarketDataSource {
    fn clone(&self) -> Self {
        Self {
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            symbols: self.symbols.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_message_parse() {
        let data = r#"[
            {"ev":"Q","sym":"SPY","b":[100.5,10,12,0],"a":[100.7,10,12,0],"t":1700000000000},
            {"ev":"T","sym":"SPY","p":100.6,"s":100,"t":1700000000001}
        ]"#;
        let msgs: Vec<WsMessage> = serde_json::from_str(data).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].symbol, "SPY");
        assert_eq!(msgs[0].b.unwrap()[0], 100.5);
        assert_eq!(msgs[0].a.unwrap()[0], 100.7);
        assert_eq!(msgs[1].event, "T");
    }
}
