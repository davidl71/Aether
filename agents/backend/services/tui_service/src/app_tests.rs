use api::finance_rates::{CurveResponse, RatePointResponse};
use api::{Alert, AlertLevel, OrderSnapshot, SystemSnapshot};
use chrono::{Duration, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use tokio::sync::{mpsc, watch};

use std::collections::HashMap;

use crate::workspace::SettingsSection;
use crate::{
    config::TuiConfig,
    events::{AppEvent, ConnectionState, ConnectionStatus, ConnectionTarget},
    models::SnapshotSource,
    models::TuiSnapshot,
    ui::{charts::render_charts, render},
};

use super::{App, InputMode, Tab};

fn make_app() -> (
    App,
    watch::Sender<Option<TuiSnapshot>>,
    mpsc::UnboundedSender<AppEvent>,
) {
    let (snap_tx, snap_rx) = watch::channel(None);
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (_config_tx, config_rx) = watch::channel(TuiConfig::default());
    let (_health_tx, health_rx) = watch::channel(HashMap::new());
    let app = App::new(
        TuiConfig::default(),
        snap_rx,
        event_rx,
        config_rx,
        health_rx,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    (app, snap_tx, event_tx)
}

fn make_snapshot() -> TuiSnapshot {
    let mut snap = TuiSnapshot::new(SystemSnapshot::default(), SnapshotSource::Nats);
    snap.inner.alerts.clear();
    snap.refresh_display_dto();
    snap
}

#[test]
fn input_mode_prefers_settings_edit_over_base_flags() {
    let (mut app, _, _) = make_app();
    app.settings_edit_config_key = Some("NATS_URL".into());
    app.settings_add_symbol_input = Some("nats://demo".into());

    assert_eq!(app.input_mode(), InputMode::SettingsEditConfig);
}

#[test]
fn input_mode_reports_chart_search_when_active() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Charts;
    app.chart_search_visible = true;

    assert_eq!(app.input_mode(), InputMode::ChartSearch);
}

#[test]
fn app_updates_connection_status() {
    let (mut app, _, event_tx) = make_app();

    event_tx
        .send(AppEvent::Connection {
            target: ConnectionTarget::Nats,
            status: ConnectionStatus::new(ConnectionState::Retrying, "Connection refused"),
        })
        .expect("send connection status");

    app.tick();

    assert_eq!(app.nats_status.state, ConnectionState::Retrying);
    assert_eq!(app.nats_status.detail, "Connection refused");
}

#[test]
fn config_hot_reload_updates_app_config() {
    let (snap_tx, snap_rx) = watch::channel(None);
    let (_event_tx, event_rx) = mpsc::unbounded_channel();
    let (config_tx, config_rx) = watch::channel(TuiConfig::default());
    let (_health_tx, health_rx) = watch::channel(HashMap::new());
    let mut app = App::new(
        TuiConfig::default(),
        snap_rx,
        event_rx,
        config_rx,
        health_rx,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    drop(snap_tx);

    let new_config = TuiConfig {
        watchlist: vec!["TSLA".into()],
        ..Default::default()
    };
    config_tx.send(new_config).expect("send new config");

    app.tick();

    assert_eq!(app.config.watchlist, vec!["TSLA"]);
}

#[test]
fn log_tab_keys_do_not_panic() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Logs;

    for key in [
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::PageUp,
        KeyCode::PageDown,
        KeyCode::Char('+'),
        KeyCode::Char('-'),
        KeyCode::Esc,
    ] {
        app.handle_key(KeyEvent::from(key));
    }
}

#[test]
fn positions_and_alerts_scroll_keys_do_not_panic() {
    let (mut app, _, _) = make_app();

    app.active_tab = Tab::Positions;
    for key in [
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::PageUp,
        KeyCode::PageDown,
    ] {
        app.handle_key(KeyEvent::from(key));
    }

    app.active_tab = Tab::Alerts;
    for key in [
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::PageUp,
        KeyCode::PageDown,
    ] {
        app.handle_key(KeyEvent::from(key));
    }
}

fn buffer_to_string(area: &ratatui::layout::Rect, buffer: &ratatui::buffer::Buffer) -> String {
    let mut s = String::new();
    for y in 0..area.height {
        for x in 0..area.width {
            s.push_str(buffer[(x, y)].symbol());
        }
        s.push('\n');
    }
    s
}

#[test]
fn yield_curve_tab_renders_with_data() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Yield;
    let curve = CurveResponse {
        symbol: "SPX".to_string(),
        points: vec![RatePointResponse {
            symbol: "SPX".to_string(),
            expiry: "2026-03-20".to_string(),
            days_to_expiry: 30,
            strike_width: 5.0,
            strike_low: None,
            strike_high: None,
            buy_implied_rate: 0.04,
            sell_implied_rate: 0.05,
            mid_rate: 0.045,
            net_debit: 4.5,
            net_credit: 4.4,
            liquidity_score: 70.0,
            timestamp: String::new(),
            spread_id: None,
            convenience_yield: None,
            data_source: None,
        }],
        timestamp: String::new(),
        strike_width: None,
        point_count: 1,
        underlying_price: None,
    };
    app.yield_curves_all
        .insert("SPX".to_string(), curve.clone());
    app.yield_curve = Some(curve);
    app.config.watchlist = vec!["SPX".to_string()];
    app.yield_benchmarks = None;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal
        .draw(|f| crate::ui::render_yield_curve_tab(f, &app, f.area()))
        .unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(
        content.contains("Yield"),
        "Yield tab should show 'Yield' title; got:\n{}",
        content
    );
    assert!(
        content.contains("SPX %") || content.contains("SPX"),
        "Yield tab should show SPX column header; got:\n{}",
        content
    );
    assert!(
        content.contains("SPX"),
        "Yield tab should show symbol SPX; got:\n{}",
        content
    );
    assert!(
        content.contains("4.50") || content.contains("4.5"),
        "Yield tab should show mid rate 4.50% (or truncated 4.5) for one point; got:\n{}",
        content
    );
}

#[test]
fn yield_curve_tab_renders_empty_state() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Yield;
    app.yield_curve = Some(CurveResponse {
        symbol: "SPX".to_string(),
        points: vec![],
        timestamp: String::new(),
        strike_width: None,
        point_count: 0,
        underlying_price: None,
    });
    app.yield_benchmarks = None;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal
        .draw(|f| crate::ui::render_yield_curve_tab(f, &app, f.area()))
        .unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Yield"), "Yield title; got:\n{}", content);
    assert!(
        content.contains("0 points") || content.contains("no data") || content.contains("waiting"),
        "Empty curve should show empty/waiting state; got:\n{}",
        content
    );
}

#[test]
fn charts_tab_shows_waiting_state_without_history() {
    let (mut app, _, _) = make_app();
    app.symbol_for_chart = "SPX".to_string();
    app.active_tab = Tab::Charts;

    let backend = TestBackend::new(60, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render_charts(f, &app, f.area())).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Waiting for live candle data for SPX."));
    assert!(content.contains("Synthetic candles are disabled."));
    assert!(content.contains("Waiting for the first backend snapshot"));
}

#[test]
fn charts_tab_shows_stale_snapshot_warning_without_history() {
    let (mut app, _, _) = make_app();
    let mut snap = make_snapshot();
    snap.received_at = Utc::now() - Duration::seconds(45);
    app.set_snapshot(Some(snap));
    app.symbol_for_chart = "SPX".to_string();
    app.active_tab = Tab::Charts;

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render_charts(f, &app, f.area())).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Waiting for live candle data for SPX."));
    assert!(content.contains("Synthetic candles are disabled."));
    assert!(content.contains("Latest snapshot is stale"));
}

#[test]
fn alerts_tab_displays_placeholder_when_no_snapshot() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Alerts;

    let backend = TestBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("No alerts"));
}

#[test]
fn alerts_tab_renders_live_alert_messages() {
    let (mut app, _, _) = make_app();
    let mut snap = make_snapshot();
    snap.inner.alerts = vec![
        Alert {
            level: AlertLevel::Info,
            message: "provider switched to polygon".into(),
            timestamp: Utc::now() - Duration::seconds(5),
        },
        Alert {
            level: AlertLevel::Warning,
            message: "SPX quote is stale".into(),
            timestamp: Utc::now(),
        },
    ];
    snap.refresh_display_dto();
    app.set_snapshot(Some(snap));
    app.active_tab = Tab::Alerts;

    let backend = TestBackend::new(60, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("provider switched to polygon"));
    assert!(content.contains("SPX quote is stale"));
}

#[test]
fn help_overlay_documents_mode_aware_bindings() {
    let (mut app, _, _) = make_app();
    app.show_help = true;

    let backend = TestBackend::new(100, 32);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Key bindings"));
}

#[test]
fn split_pane_renders_visible_mode_label() {
    let (mut app, _, _) = make_app();
    app.split_pane = true;

    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Split pane"));
    assert!(content.contains("Dashboard + Positions"));
    assert!(content.contains("PANE:DASH+POS"));
}

#[test]
fn split_pane_tab_cycles_focus_between_dashboard_and_positions() {
    let (mut app, _, _) = make_app();
    app.split_pane = true;
    app.active_tab = Tab::Dashboard;
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let _ = terminal.draw(|f| render(f, &app)).unwrap();

    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(app.active_tab, Tab::Positions);

    app.handle_key(KeyEvent::from(KeyCode::BackTab));
    assert_eq!(app.active_tab, Tab::Dashboard);
}

#[test]
fn left_right_no_longer_switch_tabs_globally() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Dashboard;

    app.handle_key(KeyEvent::from(KeyCode::Right));
    assert_eq!(app.active_tab, Tab::Dashboard);

    app.handle_key(KeyEvent::from(KeyCode::Left));
    assert_eq!(app.active_tab, Tab::Dashboard);
}

#[test]
fn settings_left_right_escapes_nested_sections() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Settings;
    app.settings_section = SettingsSection::Config;
    app.settings_config_key_index = 3;

    app.handle_key(KeyEvent::from(KeyCode::Left));
    assert_eq!(app.settings_section, SettingsSection::Health);

    app.handle_key(KeyEvent::from(KeyCode::Right));
    assert_eq!(app.settings_section, SettingsSection::Config);

    app.handle_key(KeyEvent::from(KeyCode::Right));
    assert_eq!(app.settings_section, SettingsSection::Symbols);

    app.handle_key(KeyEvent::from(KeyCode::Left));
    assert_eq!(app.settings_section, SettingsSection::Config);
}

#[test]
fn settings_up_down_escape_at_list_boundaries() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Settings;
    app.settings_section = SettingsSection::Config;
    app.settings_config_key_index = 0;

    app.handle_key(KeyEvent::from(KeyCode::Up));
    assert_eq!(app.settings_section, SettingsSection::Health);

    app.settings_section = SettingsSection::Config;
    app.settings_config_key_index = app.config_key_count().saturating_sub(1);
    app.handle_key(KeyEvent::from(KeyCode::Down));
    assert_eq!(app.settings_section, SettingsSection::Symbols);

    app.settings_section = SettingsSection::Symbols;
    app.settings_symbol_index = 0;
    app.handle_key(KeyEvent::from(KeyCode::Up));
    assert_eq!(app.settings_section, SettingsSection::Config);
}

#[test]
fn wide_terminal_renders_market_workspace() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Dashboard;

    let backend = TestBackend::new(190, 32);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Market Workspace"));
    assert!(content.contains("Dash + Pos + Orders + Yield visible"));
}

#[test]
fn wide_terminal_renders_operations_workspace() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Alerts;

    let backend = TestBackend::new(190, 32);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Operations Workspace"));
    assert!(content.contains("Alerts + Logs + Settings visible"));
}

#[test]
fn wide_operations_workspace_tab_cycles_focus_between_panes() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Alerts;

    let backend = TestBackend::new(190, 32);
    let mut terminal = Terminal::new(backend).unwrap();
    let _ = terminal.draw(|f| render(f, &app)).unwrap();

    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(app.active_tab, Tab::Logs);

    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(app.active_tab, Tab::Settings);

    app.handle_key(KeyEvent::from(KeyCode::BackTab));
    assert_eq!(app.active_tab, Tab::Logs);
}

#[test]
fn operations_workspace_banner_shows_nested_settings_focus() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Settings;
    app.settings_section = SettingsSection::Sources;

    let backend = TestBackend::new(190, 32);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Focus: Settings / Sources"));
}

#[test]
fn settings_hint_bar_shows_secondary_focus_label() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Settings;
    app.settings_section = SettingsSection::Config;

    let backend = TestBackend::new(180, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("focus:Config"));
}

#[test]
fn orders_tab_renders_filter_mode_cues() {
    let (mut app, _, _) = make_app();
    let mut snap = make_snapshot();
    snap.inner.orders = vec![OrderSnapshot {
        id: "ord-1".into(),
        symbol: "SPY".into(),
        side: "BUY".into(),
        quantity: 3,
        status: "Submitted".into(),
        submitted_at: Utc::now(),
    }];
    snap.refresh_display_dto();
    app.set_snapshot(Some(snap));
    app.active_tab = Tab::Orders;
    app.order_filter_active = true;
    app.order_filter = "SPY".into();

    let backend = TestBackend::new(100, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Orders [FILTER]"));
    assert!(content.contains("symbol/status/side"));
    assert!(content.contains("SPY"));
}

#[test]
fn settings_tab_renders_config_edit_label_and_prompt() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Settings;
    app.settings_section = SettingsSection::Config;
    app.settings_edit_config_key = Some("NATS_URL".into());
    app.settings_add_symbol_input = Some("nats://demo".into());

    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Config overrides (editing NATS_URL [editable])"));
    assert!(content.contains("NATS_URL"));
    assert!(content.contains("Active section: Config"));
}

#[test]
fn hint_bar_renders_async_status_cues() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Yield;
    app.yield_refresh_pending = true;
    app.loans_fetch_pending = true;

    let backend = TestBackend::new(240, 12);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Yield:loading"));
    assert!(content.contains("Loans:loading"));
}
