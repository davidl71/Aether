use api::finance_rates::{CurveResponse, RatePointResponse};
use api::loans::{LoanStatus, LoanType as ApiLoanType};
use api::{Alert, AlertLevel, NatsTransportHealthState, OrderSnapshot, SystemSnapshot};
use chrono::{Duration, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use tokio::sync::{mpsc, watch};

use std::collections::{HashMap, VecDeque};

use crate::focus_context::FocusContext;
use crate::workspace::{SettingsHealthFocus, SettingsSection, SecondaryFocus, VisibleWorkspace};
use crate::{
    config::TuiConfig,
    events::{AppEvent, ConnectionState, ConnectionStatus, ConnectionTarget},
    models::SnapshotSource,
    models::TuiSnapshot,
    ui::{charts::render_charts, render, Candle},
};

use super::{App, InputMode, LoanEntryState, LoanType, Tab};
use crate::mode::AppMode;

use crate::input::Action;
use crate::input_loans::{apply_loan_action, loan_form_key_action};

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
        None, // yield_refresh_tx
        None, // loans_fetch_tx
        None, // loan_create_tx
        None, // loan_bulk_import_tx
        None, // fmp_fetch_tx
        None, // greeks_fetch_tx
        None, // discount_bank_fetch_tx
        None, // ledger_fetch_tx
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
fn focus_context_reflects_tab_and_secondary_focus() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Settings;
    app.settings_section = SettingsSection::Health;
    app.settings_health_focus = SettingsHealthFocus::Services;
    let fc: FocusContext = app.focus_context();
    assert_eq!(fc.active_tab, Tab::Settings);
    assert_eq!(fc.input_mode, InputMode::Normal);
    assert_eq!(
        fc.secondary_focus,
        SecondaryFocus::SettingsHealth(SettingsHealthFocus::Services)
    );
    assert_eq!(fc.visible_workspace, VisibleWorkspace::None);
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
fn input_mode_reports_command_palette_when_visible() {
    let (mut app, _, _) = make_app();
    app.command_palette.show(app.app_mode, app.active_tab);
    assert_eq!(app.input_mode(), InputMode::CommandPalette);
}

#[test]
fn app_mode_tracks_input_mode_chart_search_and_normal() {
    assert_eq!(
        App::app_mode_for_input_mode(InputMode::Normal),
        AppMode::Navigation
    );
    assert_eq!(
        App::app_mode_for_input_mode(InputMode::ChartSearch),
        AppMode::Edit
    );
    assert_eq!(
        App::app_mode_for_input_mode(InputMode::CommandPalette),
        AppMode::Edit
    );
    assert_eq!(App::app_mode_for_input_mode(InputMode::Help), AppMode::View);
}

#[test]
fn update_app_mode_pushes_toast_on_transition() {
    let (mut app, _, _) = make_app();
    app.app_mode = AppMode::Navigation;
    app.active_tab = Tab::Charts;
    app.chart_search_visible = true;
    let before = app.toast_manager.active_count();
    app.update_app_mode();
    assert_eq!(app.app_mode, AppMode::Edit);
    assert!(app.toast_manager.active_count() > before);
}

#[test]
fn charts_tab_k_prefers_pill_navigation_over_strategy_cancel() {
    let (app, _, _) = make_app();
    let press = |code: KeyCode| KeyEvent {
        kind: KeyEventKind::Press,
        ..KeyEvent::from(code)
    };
    assert_eq!(
        crate::input::key_to_action(&app, press(KeyCode::Char('k'))),
        Some(crate::input::Action::StrategyCancelAll)
    );

    let (mut app_charts, _, _) = make_app();
    app_charts.active_tab = Tab::Charts;
    assert_eq!(
        crate::input::key_to_action(&app_charts, press(KeyCode::Char('k'))),
        Some(crate::input::Action::ChartPillUp)
    );
}

#[test]
fn positions_tab_r_cycles_sort_mode() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Positions;
    let press = |code: KeyCode| KeyEvent {
        kind: KeyEventKind::Press,
        ..KeyEvent::from(code)
    };

    assert_eq!(
        crate::input::key_to_action(&app, press(KeyCode::Char('r'))),
        Some(crate::input::Action::PositionsCycleSort)
    );
}

#[test]
fn chart_symbol_search_home_end_jumps_list_ends() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Charts;
    app.chart_search_visible = true;
    app.chart_search_results = vec!["A".into(), "B".into(), "C".into()];
    app.chart_search_selected = 0;

    app.handle_key(KeyEvent {
        kind: KeyEventKind::Press,
        ..KeyEvent::from(KeyCode::End)
    });
    assert_eq!(app.chart_search_selected, 2);

    app.handle_key(KeyEvent {
        kind: KeyEventKind::Press,
        ..KeyEvent::from(KeyCode::Home)
    });
    assert_eq!(app.chart_search_selected, 0);
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
fn app_updates_transport_health_status() {
    let (mut app, _, event_tx) = make_app();

    event_tx
        .send(AppEvent::TransportHealth(
            NatsTransportHealthState::connected(Some("nats://localhost:4222".into()), Utc::now())
                .with_subject("system.health")
                .with_role("health-subscriber"),
        ))
        .expect("send transport health");

    app.tick();

    assert_eq!(app.nats_transport.status, "ok");
    assert_eq!(app.nats_transport.role(), Some("health-subscriber"));
    assert_eq!(app.nats_transport.subject(), Some("system.health"));
}

#[test]
fn transport_health_updates_do_not_spam_toasts() {
    let (mut app, _, event_tx) = make_app();

    let before = app.toast_manager.active_count();

    for _ in 0..10 {
        event_tx
            .send(AppEvent::TransportHealth(
                NatsTransportHealthState::connected(
                    Some("nats://localhost:4222".into()),
                    Utc::now() - Duration::seconds(2),
                )
                .with_subject("system.health")
                .with_role("health-subscriber"),
            ))
            .expect("send transport health");
    }

    app.tick();

    assert_eq!(app.toast_manager.active_count(), before);
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

#[test]
fn tab_cycle_skips_logs_as_primary_shell_tab() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Alerts;

    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(app.active_tab, Tab::Yield);
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
fn charts_tab_renders_ohlcv_when_history_present() {
    let (mut app, _, _) = make_app();
    app.symbol_for_chart = "SPX".to_string();
    app.active_tab = Tab::Charts;

    let mut history = VecDeque::new();
    history.push_back(Candle {
        open: 100.0,
        high: 105.0,
        low: 99.0,
        close: 104.0,
        volume: Some(1_000_000.0),
    });
    history.push_back(Candle {
        open: 104.0,
        high: 104.5,
        low: 102.0,
        close: 102.5,
        volume: Some(800_000.0),
    });
    app.chart_history.insert("SPX".to_string(), history);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render_charts(f, &app, f.area())).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(
        content.contains(" SPX "),
        "candlestick block title should include symbol; got:\n{}",
        content
    );
    assert!(
        content.contains('│') || content.contains('█'),
        "expected OHLCV wick/body glyphs; got:\n{}",
        content
    );
    assert!(
        content.contains("Volume"),
        "volume subchart title; got:\n{}",
        content
    );
    assert!(
        !content.contains("Waiting for live candle data for SPX."),
        "with history, should not show empty-state copy; got:\n{}",
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

    let backend = TestBackend::new(80, 18);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &mut app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("No alerts"));
    assert!(content.contains("Logs ["));
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

    let backend = TestBackend::new(80, 18);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &mut app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("provider switched to polygon"));
    assert!(content.contains("SPX quote is stale"));
}

#[test]
fn macos_cmd_comma_opens_settings() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Dashboard;
    app.handle_key(KeyEvent {
        code: KeyCode::Char(','),
        modifiers: KeyModifiers::SUPER,
        kind: KeyEventKind::Press,
        ..KeyEvent::from(KeyCode::Char('x'))
    });
    assert_eq!(app.active_tab, Tab::Settings);
}

#[test]
fn help_overlay_documents_mode_aware_bindings() {
    let (mut app, _, _) = make_app();
    app.show_help = true;

    let backend = TestBackend::new(100, 32);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &mut app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Key bindings"));
    assert!(content.contains("⌘⇧P"));
    assert!(content.contains("command palette"));
    assert!(content.contains("8 Disc"));
    assert!(content.contains(":"));
    assert!(content.contains("NAV"));
    assert!(content.contains("Toasts"));
}

#[test]
fn macos_cmd_shift_p_toggles_command_palette() {
    let (mut app, _, _) = make_app();
    assert!(!app.command_palette.visible);
    app.handle_key(KeyEvent {
        code: KeyCode::Char('p'),
        modifiers: KeyModifiers::SUPER | KeyModifiers::SHIFT,
        kind: KeyEventKind::Press,
        ..KeyEvent::from(KeyCode::Char('p'))
    });
    assert!(app.command_palette.visible);
    app.handle_key(KeyEvent {
        code: KeyCode::Char('p'),
        modifiers: KeyModifiers::SUPER | KeyModifiers::SHIFT,
        kind: KeyEventKind::Press,
        ..KeyEvent::from(KeyCode::Char('p'))
    });
    assert!(!app.command_palette.visible);
}

#[test]
fn command_palette_closes_after_executing_non_palette_action() {
    let (mut app, _, _) = make_app();
    app.command_palette.visible = true;
    crate::input::apply_action(&mut app, crate::input::Action::JumpToTab(2));
    assert!(
        !app.command_palette.visible,
        "palette should close after a command runs (e.g. Enter on a palette entry)"
    );
}

#[test]
fn split_pane_renders_visible_mode_label() {
    let (mut app, _, _) = make_app();
    app.split_pane = true;

    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &mut app)).unwrap();

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
    let _ = terminal.draw(|f| render(f, &mut app)).unwrap();

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
fn settings_tab_tab_cycles_settings_section_when_no_workspace_focus_cycle() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Settings;
    app.set_last_main_area_size(120, 24); // not wide enough for Operations workspace

    app.settings_section = SettingsSection::Health;
    app.settings_health_focus = SettingsHealthFocus::Transport;

    app.handle_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(
        app.active_tab,
        Tab::Settings,
        "Tab should not leave Settings"
    );
    assert_eq!(
        app.settings_section,
        SettingsSection::Config,
        "Tab should advance Settings focus"
    );

    app.handle_key(KeyEvent::from(KeyCode::BackTab));
    assert_eq!(
        app.active_tab,
        Tab::Settings,
        "BackTab should not leave Settings"
    );
    assert_eq!(
        app.settings_section,
        SettingsSection::Health,
        "BackTab should reverse Settings focus"
    );
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
    let frame = terminal.draw(|f| render(f, &mut app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Market Workspace"));
    assert!(content.contains("Dash + Pos + Orders + Yield visible"));
    assert!(content.contains("Scroll: pane under cursor"));
}

#[test]
fn wide_terminal_renders_operations_workspace() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Alerts;

    let backend = TestBackend::new(190, 32);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &mut app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Operations Workspace"));
    assert!(content.contains("Alerts + Logs + Settings visible"));
    assert!(
        content.contains("Alpaca credentials"),
        "composed ops workspace should include Settings → Alpaca like full Settings tab"
    );
}

#[test]
fn standalone_alerts_tab_renders_logs_below_alerts() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Alerts;

    let backend = TestBackend::new(120, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &mut app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Alerts"));
    assert!(content.contains("Logs ["));
}

#[test]
fn wide_operations_workspace_tab_cycles_focus_between_panes() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Alerts;

    let backend = TestBackend::new(190, 32);
    let mut terminal = Terminal::new(backend).unwrap();
    let _ = terminal.draw(|f| render(f, &mut app)).unwrap();

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
    app.settings_section = SettingsSection::Health;
    app.settings_health_focus = SettingsHealthFocus::Services;

    let backend = TestBackend::new(190, 32);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &mut app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Focus: Settings / Health / Services"));
}

#[test]
fn workspace_focus_target_returns_none_when_no_workspace_is_visible() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Alerts;
    app.set_last_main_area_size(120, 24);

    assert_eq!(
        app.visible_workspace(),
        crate::workspace::VisibleWorkspace::None
    );
    assert_eq!(app.workspace_focus_target(true), None);
    assert_eq!(app.workspace_focus_target(false), None);
}

#[test]
fn workspace_focus_target_cycles_market_workspace_tabs() {
    let (mut app, _, _) = make_app();
    app.set_last_main_area_size(190, 32);
    app.active_tab = Tab::Dashboard;

    assert_eq!(
        app.visible_workspace(),
        crate::workspace::VisibleWorkspace::Market
    );
    assert_eq!(app.workspace_focus_target(true), Some(Tab::Positions));
    assert_eq!(app.workspace_focus_target(false), Some(Tab::Yield));

    app.active_tab = Tab::Yield;
    assert_eq!(app.workspace_focus_target(true), Some(Tab::Dashboard));
    assert_eq!(app.workspace_focus_target(false), Some(Tab::Orders));
}

#[test]
fn market_workspace_mouse_scroll_targets_pane_under_cursor() {
    use api::{CandleSnapshot, SymbolSnapshot};
    use crossterm::event::{MouseEvent, MouseEventKind};

    let (mut app, _, _) = make_app();
    app.set_last_main_area_size(190, 32);

    // Focus Yield, but scroll the Dashboard pane via mouse wheel location.
    app.active_tab = Tab::Yield;

    // Seed a snapshot with multiple dashboard rows so selection can advance.
    let mut sys = SystemSnapshot::default();
    let now = Utc::now();
    sys.symbols = vec![
        SymbolSnapshot {
            symbol: "AAA".into(),
            last: 1.0,
            bid: 1.0,
            ask: 1.0,
            spread: 0.0,
            roi: 0.0,
            maker_count: 0,
            taker_count: 0,
            volume: 0,
            candle: CandleSnapshot {
                open: 1.0,
                high: 1.0,
                low: 1.0,
                close: 1.0,
                volume: 0,
                entry: 1.0,
                updated: now,
            },
        },
        SymbolSnapshot {
            symbol: "BBB".into(),
            last: 2.0,
            bid: 2.0,
            ask: 2.0,
            spread: 0.0,
            roi: 0.0,
            maker_count: 0,
            taker_count: 0,
            volume: 0,
            candle: CandleSnapshot {
                open: 2.0,
                high: 2.0,
                low: 2.0,
                close: 2.0,
                volume: 0,
                entry: 2.0,
                updated: now,
            },
        },
        SymbolSnapshot {
            symbol: "CCC".into(),
            last: 3.0,
            bid: 3.0,
            ask: 3.0,
            spread: 0.0,
            roi: 0.0,
            maker_count: 0,
            taker_count: 0,
            volume: 0,
            candle: CandleSnapshot {
                open: 3.0,
                high: 3.0,
                low: 3.0,
                close: 3.0,
                volume: 0,
                entry: 3.0,
                updated: now,
            },
        },
    ];
    let mut snap = TuiSnapshot::new(sys, SnapshotSource::Nats);
    snap.refresh_display_dto();
    app.set_snapshot(Some(snap));

    let size = ratatui::layout::Rect::new(0, 0, 190, 32);
    let mouse = MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 5,
        row: 6,
        modifiers: crossterm::event::KeyModifiers::empty(),
    };

    let action = crate::mouse::handle_mouse_event(&app, mouse, size).expect("expected action");
    assert!(
        matches!(
            action,
            crate::input::Action::MouseScrollDownIn(Tab::Dashboard)
        ),
        "expected mouse scroll to route to Dashboard pane, got {action:?}"
    );

    let before = app.dashboard_table.selected();
    app.handle_action(action);
    let after = app.dashboard_table.selected();
    assert!(
        after > before,
        "expected dashboard selection to advance (before={before}, after={after})"
    );
    assert_eq!(
        app.active_tab,
        Tab::Yield,
        "mouse wheel should not steal focus from the active tab"
    );
}

#[test]
fn mouse_click_on_tab_bar_routes_via_recorded_regions() {
    use crossterm::event::{MouseEvent, MouseEventKind};

    let (mut app, _, _) = make_app();

    // Render once to populate `tab_bar_regions` via `ui::render_tab_bar`.
    let backend = TestBackend::new(120, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let _ = terminal.draw(|f| render(f, &mut app)).unwrap();

    let (tab, rect) = app
        .tab_bar_regions
        .borrow()
        .iter()
        .find(|(t, _)| *t == Tab::Orders)
        .copied()
        .expect("expected Orders tab region");

    // Click within the Orders tab region.
    let mouse = MouseEvent {
        kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: rect.x.saturating_add(1),
        row: rect.y,
        modifiers: crossterm::event::KeyModifiers::empty(),
    };

    let size = ratatui::layout::Rect::new(0, 0, 120, 24);
    let action = crate::mouse::handle_mouse_event(&app, mouse, size).expect("expected action");

    assert_eq!(
        action,
        crate::input::Action::JumpToTab(
            (Tab::ALL.iter().position(|t| t == &tab).unwrap() + 1) as u8
        ),
        "expected mouse click to map to JumpToTab for the clicked tab"
    );
}

#[test]
fn workspace_focus_target_cycles_credit_workspace_tabs() {
    let (mut app, _, _) = make_app();
    app.set_last_main_area_size(190, 32);
    app.active_tab = Tab::Loans;

    assert_eq!(
        app.visible_workspace(),
        crate::workspace::VisibleWorkspace::Credit
    );
    assert_eq!(app.workspace_focus_target(true), Some(Tab::DiscountBank));
    assert_eq!(app.workspace_focus_target(false), Some(Tab::DiscountBank));
}

#[test]
fn settings_hint_bar_shows_secondary_focus_label() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Settings;
    app.settings_section = SettingsSection::Health;
    app.settings_health_focus = SettingsHealthFocus::Transport;

    let backend = TestBackend::new(180, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &mut app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("focus:Transport"));
}

#[test]
fn settings_tab_uses_wide_layout_in_medium_terminal() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Settings;

    // Medium terminal: wide enough for the 2-column Settings layout even when
    // not "ultra wide".
    let backend = TestBackend::new(110, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &mut app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(
        content.contains("Config overrides")
            && content.contains("Data sources")
            && content.contains("Alpaca credentials"),
        "expected Settings sections to render in medium terminal"
    );
}

#[test]
fn settings_health_scroll_down_advances_nested_focus_before_next_section() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Settings;
    app.settings_section = SettingsSection::Health;
    app.settings_health_focus = SettingsHealthFocus::Transport;

    app.handle_key(KeyEvent::from(KeyCode::Down));
    assert_eq!(app.settings_section, SettingsSection::Health);
    assert_eq!(app.settings_health_focus, SettingsHealthFocus::Services);

    app.handle_key(KeyEvent::from(KeyCode::Down));
    assert_eq!(app.settings_section, SettingsSection::Config);
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
    let frame = terminal.draw(|f| render(f, &mut app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Orders [FILTER]"));
    assert!(content.contains("symbol/status/side"));
    assert!(content.contains("SPY"));
}

#[test]
fn orders_filter_typing_mode_deactivates_when_navigating_away() {
    let (mut app, _, _) = make_app();
    app.active_tab = Tab::Orders;
    app.order_filter_active = true;
    app.order_filter = "SPY".into();

    // Navigate away while still in OrdersFilter input mode. Plain '1' is treated as filter input,
    // so use the macOS Cmd+1 shell binding to jump tabs.
    app.handle_key(KeyEvent::new(
        KeyCode::Char('1'),
        crossterm::event::KeyModifiers::SUPER,
    ));

    assert_eq!(app.active_tab, Tab::Dashboard);
    assert!(!app.order_filter_active);
    assert_eq!(app.order_filter, "SPY");
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
    let frame = terminal.draw(|f| render(f, &mut app)).unwrap();

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
    app.fmp_fetch_pending = true;

    let backend = TestBackend::new(240, 12);
    let mut terminal = Terminal::new(backend).unwrap();
    let frame = terminal.draw(|f| render(f, &mut app)).unwrap();

    let content = buffer_to_string(&frame.area, &frame.buffer);
    assert!(content.contains("Yield:loading"));
    assert!(content.contains("Loans:loading"));
    assert!(content.contains("FMP:loading"));
}

#[test]
fn request_loans_fetch_if_uncached_skips_when_list_already_ok() {
    let (loans_tx, mut loans_rx) = mpsc::unbounded_channel();
    let (_snap_tx, snap_rx) = watch::channel(None);
    let (_event_tx, event_rx) = mpsc::unbounded_channel();
    let (_config_tx, config_rx) = watch::channel(TuiConfig::default());
    let (_health_tx, health_rx) = watch::channel(HashMap::new());
    let mut app = App::new(
        TuiConfig::default(),
        snap_rx,
        event_rx,
        config_rx,
        health_rx,
        None,
        Some(loans_tx),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    app.loans_list = Some(Ok(vec![]));
    app.request_loans_fetch_if_uncached();
    assert!(!app.loans_fetch_pending);
    assert!(loans_rx.try_recv().is_err());

    app.loans_list = None;
    app.request_loans_fetch_if_uncached();
    assert!(app.loans_fetch_pending);
    assert!(loans_rx.try_recv().is_ok());
}

#[test]
fn loan_form_key_action_maps_navigation_and_input_keys() {
    assert_eq!(
        loan_form_key_action(KeyCode::Esc),
        Some(Action::LoansInputEscape)
    );
    assert_eq!(
        loan_form_key_action(KeyCode::Enter),
        Some(Action::LoansInputEnter)
    );
    assert_eq!(
        loan_form_key_action(KeyCode::Tab),
        Some(Action::LoansInputNavDown)
    );
    assert_eq!(
        loan_form_key_action(KeyCode::BackTab),
        Some(Action::LoansInputNavUp)
    );
    assert_eq!(
        loan_form_key_action(KeyCode::Up),
        Some(Action::LoansInputNavUp)
    );
    assert_eq!(
        loan_form_key_action(KeyCode::Down),
        Some(Action::LoansInputNavDown)
    );
    assert_eq!(
        loan_form_key_action(KeyCode::Backspace),
        Some(Action::LoansInputBackspace)
    );
    assert_eq!(
        loan_form_key_action(KeyCode::Char('7')),
        Some(Action::LoansInputChar('7'))
    );
    assert_eq!(
        loan_form_key_action(KeyCode::Char('-')),
        Some(Action::LoansInputChar('-'))
    );
    assert_eq!(
        loan_form_key_action(KeyCode::Char('.')),
        Some(Action::LoansInputChar('.'))
    );
    assert_eq!(
        loan_form_key_action(KeyCode::Char('a')),
        Some(Action::LoansInputChar('a'))
    );
    assert_eq!(loan_form_key_action(KeyCode::F(1)), Some(Action::NoOp));
}

#[test]
fn loan_entry_toggle_loan_type_alternates_variant() {
    let mut entry = LoanEntryState::new();
    assert_eq!(entry.loan_type, LoanType::ShirBased);
    entry.toggle_loan_type();
    assert_eq!(entry.loan_type, LoanType::CpiLinked);
    entry.toggle_loan_type();
    assert_eq!(entry.loan_type, LoanType::ShirBased);
}

#[test]
fn loan_entry_to_loan_record_none_when_incomplete_or_invalid() {
    let entry = LoanEntryState::new();
    assert!(!entry.is_complete());
    assert!(entry.to_loan_record().is_none());

    let mut bad = LoanEntryState::new();
    bad.bank_name = "B".into();
    bad.account_number = "A".into();
    bad.principal = "x".into();
    bad.interest_rate = "5".into();
    bad.origination_date = "2025-01-01".into();
    bad.first_payment_date = "2025-02-01".into();
    bad.num_payments = "12".into();
    assert!(!bad.is_complete());
    assert!(bad.to_loan_record().is_none());
}

fn sample_complete_loan_entry() -> LoanEntryState {
    let mut e = LoanEntryState::new();
    e.bank_name = "TestBank".into();
    e.account_number = "ACC-9".into();
    e.principal = "100000".into();
    e.interest_rate = "5.5".into();
    e.spread = "0.25".into();
    e.origination_date = "2024-01-01".into();
    e.first_payment_date = "2024-02-01".into();
    e.num_payments = "360".into();
    e.currency = "ILS".into();
    e.calculate_maturity();
    e.calculate_monthly_payment();
    e
}

#[test]
fn loan_entry_to_loan_record_builds_api_loan_when_valid() {
    let e = sample_complete_loan_entry();
    let expected_maturity = e.maturity_date.clone();
    assert!(e.is_complete());
    let rec = e.to_loan_record().expect("loan record");
    assert_eq!(rec.bank_name, "TestBank");
    assert_eq!(rec.account_number, "ACC-9");
    assert_eq!(rec.loan_type, ApiLoanType::ShirBased);
    assert!((rec.principal - 100_000.0).abs() < 1e-6);
    assert_eq!(rec.original_principal, rec.principal);
    assert!((rec.interest_rate - 5.5).abs() < 1e-9);
    assert!((rec.spread - 0.25).abs() < 1e-9);
    assert_eq!(rec.origination_date, "2024-01-01");
    assert_eq!(rec.next_payment_date, "2024-02-01");
    assert_eq!(rec.maturity_date, expected_maturity);
    assert_eq!(rec.status, LoanStatus::Active);
    assert_eq!(rec.currency, "ILS");
    assert_eq!(rec.payment_frequency_months, 1);
    assert!(rec.loan_id.starts_with("loan-TestBank-"));
    assert!(rec.monthly_payment > 0.0);
}

#[test]
fn loan_entry_to_loan_record_respects_cpi_linked_type() {
    let mut e = sample_complete_loan_entry();
    e.loan_type = LoanType::CpiLinked;
    let rec = e.to_loan_record().expect("record");
    assert_eq!(rec.loan_type, ApiLoanType::CpiLinked);
}

#[test]
fn loan_entry_empty_spread_defaults_to_zero_in_record() {
    let mut e = sample_complete_loan_entry();
    e.spread.clear();
    let rec = e.to_loan_record().expect("record");
    assert_eq!(rec.spread, 0.0);
}

#[test]
fn apply_loan_enter_sends_record_and_clears_form_when_channel_wired() {
    let (loan_tx, mut loan_rx) = mpsc::unbounded_channel();
    let (_snap_tx, snap_rx) = watch::channel(None);
    let (_event_tx, event_rx) = mpsc::unbounded_channel();
    let (_config_tx, config_rx) = watch::channel(TuiConfig::default());
    let (_health_tx, health_rx) = watch::channel(HashMap::new());
    let mut app = App::new(
        TuiConfig::default(),
        snap_rx,
        event_rx,
        config_rx,
        health_rx,
        None,
        None,
        Some(loan_tx),
        None,
        None,
        None,
        None,
        None,
    );
    app.loan_entry = Some(sample_complete_loan_entry());
    assert!(apply_loan_action(&mut app, Action::LoansInputEnter));
    assert!(app.loan_entry.is_none());
    let rec = loan_rx.try_recv().expect("loan sent");
    assert_eq!(rec.bank_name, "TestBank");
}
