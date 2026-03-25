use super::*;

impl App {
    /// Replaces the current snapshot (used by tick processing).
    #[inline]
    pub fn set_snapshot(&mut self, snap: Option<TuiSnapshot>) {
        unsafe {
            *self.snapshot.get() = snap;
        }
        self.hydrate_chart_history_from_snapshot();
    }

    /// Applies a tick market data event to the current snapshot.
    /// Creates a new symbol entry if this is the first tick for the symbol.
    /// Does nothing if no snapshot is loaded yet.
    #[inline]
    pub fn apply_tick(&mut self, symbol: &str, bid: f64, ask: f64, last: f64) {
        let snap_ptr = self.snapshot.get();
        if snap_ptr.is_null() {
            return;
        }
        let snap = unsafe { &mut *snap_ptr };
        if let Some(ref mut s) = snap {
            let mid = if last != 0.0 { last } else { (bid + ask) * 0.5 };
            if let Some(entry) = s.inner.symbols.iter_mut().find(|e| e.symbol == symbol) {
                entry.last = mid;
                entry.bid = bid;
                entry.ask = ask;
                entry.spread = (ask - bid).max(0.0);
            } else if bid != 0.0 || ask != 0.0 {
                s.inner.symbols.push(api::SymbolSnapshot {
                    symbol: symbol.to_string(),
                    last: mid,
                    bid,
                    ask,
                    spread: (ask - bid).max(0.0),
                    roi: 0.0,
                    maker_count: 1,
                    taker_count: 0,
                    volume: 0,
                    candle: api::CandleSnapshot {
                        open: mid,
                        high: mid,
                        low: mid,
                        close: mid,
                        volume: 0,
                        entry: mid,
                        updated: chrono::Utc::now(),
                    },
                });
            }

            for position in &mut s.inner.positions {
                if position.symbol == symbol {
                    position.mark = mid;
                    position.unrealized_pnl =
                        (mid - position.cost_basis) * position.quantity as f64;
                }
            }

            s.refresh_display_dto();
            self.mark_dirty();
        }
    }

    pub fn apply_candle(
        &mut self,
        symbol: &str,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: u64,
    ) {
        self.push_chart_candle(
            symbol,
            Candle {
                open,
                high,
                low,
                close,
                volume: Some(volume as f64),
            },
        );
        self.mark_dirty();
    }

    pub fn apply_alert(
        &mut self,
        level: api::AlertLevel,
        message: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) {
        let snap_ptr = self.snapshot.get();
        if snap_ptr.is_null() {
            return;
        }
        let snap = unsafe { &mut *snap_ptr };
        if let Some(ref mut s) = snap {
            s.inner.alerts.push(Alert {
                level,
                message,
                timestamp,
            });
            while s.inner.alerts.len() > 32 {
                s.inner.alerts.remove(0);
            }
            s.refresh_display_dto();
            self.mark_dirty();
        }
    }

    fn hydrate_chart_history_from_snapshot(&mut self) {
        let symbols = self
            .snapshot()
            .as_ref()
            .map(|snap| snap.inner.symbols.clone())
            .unwrap_or_default();

        for symbol in symbols {
            self.push_chart_candle(
                &symbol.symbol,
                Candle {
                    open: symbol.candle.open,
                    high: symbol.candle.high,
                    low: symbol.candle.low,
                    close: symbol.candle.close,
                    volume: Some(symbol.candle.volume as f64),
                },
            );
        }
    }

    fn push_chart_candle(&mut self, symbol: &str, candle: Candle) {
        let history = self.chart_history.entry(symbol.to_string()).or_default();

        if let Some(last) = history.back_mut() {
            if last.open == candle.open
                && last.high == candle.high
                && last.low == candle.low
                && last.close == candle.close
                && last.volume == candle.volume
            {
                return;
            }
        }

        history.push_back(candle);
        while history.len() > CHART_HISTORY_SIZE {
            history.pop_front();
        }
    }

    /// Pull latest snapshot and config updates, process queued events.
    /// Returns true if the UI state changed and needs redraw.
    pub fn tick(&mut self) {
        let mut changed = false;

        let ttl = std::time::Duration::from_secs(TOAST_TTL_SECS);
        let before = self.toast_queue.len();
        self.toast_queue.retain(|(_, _, ts)| ts.elapsed() < ttl);
        if self.toast_queue.len() != before {
            changed = true;
        }

        if self.config_rx.has_changed().unwrap_or(false) {
            let base = self.config_rx.borrow_and_update().clone();
            self.config = merge_config_overrides(base, &self.config_overrides);
            self.config_warning = validate_config_hint(&self.config);
            self.split_pane = self.config.split_pane;
            tracing::info!("Config reloaded from disk");
            changed = true;
        }

        if self.health_rx.has_changed().unwrap_or(false) {
            self.backend_health = self.health_rx.borrow_and_update().clone();
        }

        while let Ok(event) = self.event_rx.try_recv() {
            self.apply_event(event);
        }

        if self.snapshot_rx.has_changed().unwrap_or(false) {
            let next_snapshot = {
                let borrowed = self.snapshot_rx.borrow_and_update();
                borrowed.clone()
            };

            if let Some(snap) = next_snapshot {
                if self.should_accept_snapshot(&snap) {
                    self.set_snapshot(Some(snap.clone()));
                    self.update_roi_history(&snap);
                    changed = true;
                }
            }
        }

        if let Some(ref s) = self.snapshot() {
            let (display_len, _, _) = crate::ui::positions_display_info(
                &s.dto().positions,
                self.positions_combo_view,
                &self.positions_expanded_combos,
            );
            if display_len > 0 {
                self.positions_scroll = self.positions_scroll.min(display_len - 1);
            }
        }

        const MARKET_CHECK_INTERVAL_TICKS: u32 = 240;
        self.market_open_tick = self.market_open_tick.saturating_add(1);
        if self.market_open_tick == 1 || self.market_open_tick >= MARKET_CHECK_INTERVAL_TICKS {
            self.market_open_tick = 0;
            self.market_open = nyse_is_open();
        }

        if self.active_tab == Tab::Loans && self.loans_list.is_none() && !self.loans_fetch_pending {
            self.request_loans_fetch();
        }

        let needs_cred_refresh = self
            .credential_status_refreshed_at
            .map(|t| t.elapsed().as_secs() >= 30)
            .unwrap_or(true);
        if needs_cred_refresh {
            self.refresh_credential_status();
        }

        if changed {
            self.needs_redraw = true;
        }
    }

    fn refresh_credential_status(&mut self) {
        use api::credentials::{credential_source, CredentialKey};
        for (provider, key) in [
            ("fmp", CredentialKey::FmpApiKey),
            ("polygon", CredentialKey::PolygonApiKey),
            ("fred", CredentialKey::FredApiKey),
            ("tase", CredentialKey::TaseApiKey),
        ] {
            let source = credential_source(key);
            self.credential_status.insert(provider, source.is_some());
            if let Some(source) = source {
                self.credential_source.insert(provider, source.label());
            } else {
                self.credential_source.remove(provider);
            }
        }
        self.credential_status_refreshed_at = Some(Instant::now());
    }

    fn apply_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Connection { target, status } => match target {
                ConnectionTarget::Nats => self.nats_status = status,
            },
            AppEvent::CommandStatus(reply) => {
                self.set_command_status(CommandStatusView::from_reply(&reply));
            }
            AppEvent::MarketTick {
                symbol,
                bid,
                ask,
                last,
                source,
                source_priority,
            } => {
                self.apply_tick(&symbol, bid, ask, last);
                self.market_data_sources.insert(
                    source.clone(),
                    MarketDataSourceMeta::new(source, source_priority),
                );
                self.recompute_live_market_data_source();
            }
            AppEvent::MarketCandle {
                symbol,
                open,
                high,
                low,
                close,
                volume,
            } => {
                self.apply_candle(&symbol, open, high, low, close, volume);
            }
            AppEvent::AlertReceived {
                level,
                message,
                timestamp,
            } => {
                self.apply_alert(level, message, timestamp);
            }
            AppEvent::YieldCurveKvUpdate {
                symbol,
                curve,
                fetched_at,
            } => {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&fetched_at) {
                    self.yield_last_refreshed
                        .insert(symbol.clone(), dt.with_timezone(&chrono::Utc));
                }
                self.yield_curves_all.insert(symbol.clone(), curve);
                self.sync_yield_curve_from_cache();
                if let Some(task) = self
                    .yield_tasks
                    .iter_mut()
                    .find(|t| t.status == RefreshTaskStatus::Pending)
                {
                    task.status = RefreshTaskStatus::Done;
                    task.completed_at = Some(chrono::Utc::now());
                }
                self.yield_refresh_pending = false;
                self.yield_error = None;
                self.mark_dirty();
            }
            AppEvent::BenchmarksUpdate(benchmarks) => {
                self.yield_benchmarks = Some(benchmarks);
                self.mark_dirty();
            }
            AppEvent::YieldRefreshAck { ok } => {
                if !ok {
                    if let Some(task) = self
                        .yield_tasks
                        .iter_mut()
                        .find(|t| t.status == RefreshTaskStatus::Pending)
                    {
                        task.status = RefreshTaskStatus::Error("backend rejected refresh".into());
                        task.completed_at = Some(chrono::Utc::now());
                    }
                    self.yield_refresh_pending = false;
                }
                self.mark_dirty();
            }
        }
    }

    fn recompute_live_market_data_source(&mut self) {
        const LIVE_SOURCE_TTL_SECS: u64 = 5;

        self.market_data_sources
            .retain(|_, meta| meta.is_fresh(LIVE_SOURCE_TTL_SECS));

        let configured_provider = self
            .snapshot()
            .as_ref()
            .and_then(|snap| snap.inner.market_data_source.as_deref())
            .map(str::to_lowercase);
        let real_provider_configured = configured_provider
            .as_deref()
            .is_some_and(|provider| provider != "mock")
            || self
                .market_data_sources
                .keys()
                .any(|source| source.as_str() != "mock");

        let mut candidates: Vec<&MarketDataSourceMeta> = self
            .market_data_sources
            .values()
            .filter(|meta| !(real_provider_configured && meta.source == "mock"))
            .collect();
        if candidates.is_empty() {
            candidates = self.market_data_sources.values().collect();
        }

        candidates.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.age_secs().cmp(&b.age_secs()))
                .then_with(|| a.source.cmp(&b.source))
        });

        self.live_market_data_source = candidates.first().cloned().cloned();
    }

    fn should_accept_snapshot(&self, _incoming: &TuiSnapshot) -> bool {
        true
    }
}
