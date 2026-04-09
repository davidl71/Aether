//! View-level actions that mutate scroll/selection or open overlays.
//!
//! This is the shared implementation for actions that apply across multiple tabs
//! (Orders, Alerts, Dashboard, Scenarios, etc.) once the input router has decided
//! an [`Action`].

use crate::app::{App, DetailPopupContent, Tab};
use crate::input::Action;

fn clamp_filtered_orders_selection(app: &mut App) {
    let len = app.filtered_orders_len();
    app.orders_table.clamp_to_len(len);
}

pub(crate) fn apply_view_action(app: &mut App, action: Action) -> bool {
    match action {
        Action::OrdersScrollUp => {
            app.orders_table.move_up();
        }
        Action::OrdersScrollDown => {
            let len = app.filtered_orders_len();
            app.orders_table.move_down(len);
        }
        Action::OrdersScrollPageUp => {
            let len = app.filtered_orders_len();
            app.orders_table.shift_selected(-10, len);
        }
        Action::OrdersScrollPageDown => {
            let len = app.filtered_orders_len();
            app.orders_table.shift_selected(10, len);
        }
        Action::OrdersDetail => {
            if let Some(ref snap) = app.snapshot() {
                let filtered = app.filtered_orders(snap);
                let idx = app
                    .orders_table
                    .selected()
                    .min(filtered.len().saturating_sub(1));
                if let Some(order) = filtered.get(idx) {
                    app.detail_popup = Some(DetailPopupContent::Order(order.clone()));
                }
            }
        }
        Action::OrdersFilterFocus => {
            if app.order_filter_active && app.order_filter.is_empty() {
                app.order_filter_active = false;
                #[cfg(feature = "tui-interact")]
                app.orders_filter_interact.on_close();
                app.command_success(
                    "orders_filter",
                    "Filter mode off (/ or i to type again). Saved text kept until Esc clears.",
                );
            } else {
                app.order_filter_active = true;
                #[cfg(feature = "tui-interact")]
                app.orders_filter_interact.on_open();
                app.command_success(
                    "orders_filter",
                    "Filter mode: type symbol, status, or side (/ or i to focus; Esc clears; / exits when empty).",
                );
            }
        }
        Action::OrdersFilterChar(c) => {
            app.order_filter_active = true;
            app.order_filter.push(c);
            clamp_filtered_orders_selection(app);
        }
        Action::OrdersFilterBackspace => {
            app.order_filter.pop();
            clamp_filtered_orders_selection(app);
        }
        Action::OrdersFilterClear => {
            app.order_filter.clear();
            app.order_filter_active = false;
            #[cfg(feature = "tui-interact")]
            app.orders_filter_interact.on_close();
            clamp_filtered_orders_selection(app);
            app.command_success("orders_filter", "Filter cleared.");
        }
        Action::OrdersFilterFocusNext => {
            #[cfg(feature = "tui-interact")]
            app.orders_filter_interact.tab_next();
        }
        Action::OrdersFilterFocusPrev => {
            #[cfg(feature = "tui-interact")]
            app.orders_filter_interact.tab_prev();
        }
        Action::OrdersCancel => {
            app.command_disabled("cancel_all");
        }
        Action::AlertsScrollUp => {
            let len = crate::ui::alerts::alert_lines(app).len();
            app.alerts_scroll.shift_scroll(-1, len.saturating_sub(1));
        }
        Action::AlertsScrollDown => {
            let len = crate::ui::alerts::alert_lines(app).len();
            app.alerts_scroll.shift_scroll(1, len.saturating_sub(1));
        }
        Action::AlertsScrollPageUp => {
            let len = crate::ui::alerts::alert_lines(app).len();
            app.alerts_scroll.shift_scroll(-10, len.saturating_sub(1));
        }
        Action::AlertsScrollPageDown => {
            let len = crate::ui::alerts::alert_lines(app).len();
            app.alerts_scroll.shift_scroll(10, len.saturating_sub(1));
        }
        Action::DashboardScrollUp => {
            app.dashboard_table.move_up();
        }
        Action::DashboardScrollDown => {
            let len = app
                .snapshot()
                .as_ref()
                .map(|s| s.inner.symbols.len())
                .unwrap_or(0);
            app.dashboard_table.move_down(len);
        }
        Action::DashboardNavigateToChart => {
            let symbol = app.snapshot().as_ref().and_then(|snap| {
                let idx = app
                    .dashboard_table
                    .selected()
                    .min(snap.inner.symbols.len().saturating_sub(1));
                snap.inner.symbols.get(idx).map(|s| s.symbol.clone())
            });
            if let Some(symbol) = symbol {
                app.active_tab = Tab::Charts;
                app.symbol_for_chart = symbol;
                app.chart_search_visible = false;
                app.chart_search_input.clear();
                #[cfg(feature = "tui-interact")]
                app.chart_search_interact.on_close();
            }
        }
        Action::ScenariosScrollUp => {
            app.scenarios_table.move_up();
        }
        Action::ScenariosScrollDown => {
            let filtered = crate::ui::filtered_scenarios(app);
            app.scenarios_table.move_down(filtered.len());
        }
        Action::ScenariosScrollPageUp => {
            let filtered = crate::ui::filtered_scenarios(app);
            app.scenarios_table.shift_selected(-10, filtered.len());
        }
        Action::ScenariosScrollPageDown => {
            let filtered = crate::ui::filtered_scenarios(app);
            app.scenarios_table.shift_selected(10, filtered.len());
        }
        Action::ScenariosDetail => {
            let filtered = crate::ui::filtered_scenarios(app);
            let idx = app
                .scenarios_table
                .selected()
                .min(filtered.len().saturating_sub(1));
            if let Some(scenario) = filtered.get(idx) {
                app.detail_popup = Some(DetailPopupContent::Scenario(scenario.clone()));
            }
        }
        Action::ScenariosDteContract => {
            app.scenarios_dte_half_width = (app.scenarios_dte_half_width - 1).max(0);
        }
        Action::ScenariosDteExpand => {
            app.scenarios_dte_half_width = (app.scenarios_dte_half_width + 1).min(60);
        }
        Action::ScenariosExecute => {
            app.command_disabled("scenario_run");
        }
        Action::ScenariosCycleStrikeWidth => {
            app.scenarios_strike_width_filter = match app.scenarios_strike_width_filter {
                None => Some(25),
                Some(25) => Some(50),
                Some(50) => Some(100),
                Some(_) => None,
            };
        }
        Action::LedgerScrollUp => {
            app.ledger_table.move_up();
        }
        Action::LedgerScrollDown => {
            let len = app
                .ledger_journal
                .as_ref()
                .and_then(|r| r.as_ref().ok())
                .map(|j| j.entries.len())
                .unwrap_or(0);
            app.ledger_table.move_down(len);
        }
        Action::LedgerScrollPageUp => {
            let len = app
                .ledger_journal
                .as_ref()
                .and_then(|r| r.as_ref().ok())
                .map(|j| j.entries.len())
                .unwrap_or(0);
            app.ledger_table.shift_selected(-10, len);
        }
        Action::LedgerScrollPageDown => {
            let len = app
                .ledger_journal
                .as_ref()
                .and_then(|r| r.as_ref().ok())
                .map(|j| j.entries.len())
                .unwrap_or(0);
            app.ledger_table.shift_selected(10, len);
        }
        Action::LedgerRefresh => {
            app.request_ledger_fetch();
        }
        Action::ChartSearchFocus => {
            app.chart_search_visible = true;
            app.chart_search_input.clear();
            app.chart_search_results.clear();
            app.chart_search_selected = 0;
            #[cfg(feature = "tui-interact")]
            app.chart_search_interact.on_open();
            crate::ui::charts::update_search_results(app);
        }
        Action::ChartSearchChar(c) => {
            app.chart_search_input.push(c);
            app.chart_search_selected = 0;
            crate::ui::charts::update_search_results(app);
        }
        Action::ChartSearchBackspace => {
            app.chart_search_input.pop();
            app.chart_search_selected = 0;
            crate::ui::charts::update_search_results(app);
        }
        Action::ChartSearchUp => {
            if !app.chart_search_results.is_empty() {
                app.chart_search_selected = app.chart_search_selected.saturating_sub(1);
            }
        }
        Action::ChartSearchDown => {
            app.chart_search_selected = (app.chart_search_selected + 1)
                .min(app.chart_search_results.len().saturating_sub(1));
        }
        Action::ChartSearchEnter => {
            if app.chart_search_input.is_empty() && !app.chart_search_results.is_empty() {
                if let Some(selected) = app.chart_search_results.get(app.chart_search_selected) {
                    app.symbol_for_chart = selected.clone();
                    if !app.chart_search_history.contains(selected) {
                        app.chart_search_history.push_front(selected.clone());
                        if app.chart_search_history.len()
                            > crate::chart_search_history::CHART_SEARCH_HISTORY_MAX
                        {
                            app.chart_search_history.pop_back();
                        }
                    }
                }
            } else if !app.chart_search_input.is_empty() {
                app.symbol_for_chart = app.chart_search_input.clone();
                if !app.chart_search_history.contains(&app.chart_search_input) {
                    app.chart_search_history
                        .push_front(app.chart_search_input.clone());
                    if app.chart_search_history.len()
                        > crate::chart_search_history::CHART_SEARCH_HISTORY_MAX
                    {
                        app.chart_search_history.pop_back();
                    }
                }
            }
            crate::chart_search_history::save_chart_search_history(&app.chart_search_history);
            app.chart_search_visible = false;
            app.chart_search_input.clear();
            #[cfg(feature = "tui-interact")]
            app.chart_search_interact.on_close();
        }
        Action::ChartSearchEscape => {
            app.chart_search_visible = false;
            app.chart_search_input.clear();
            app.chart_search_results.clear();
            #[cfg(feature = "tui-interact")]
            app.chart_search_interact.on_close();
        }
        Action::ChartSearchFirst => {
            if !app.chart_search_results.is_empty() {
                app.chart_search_selected = 0;
            }
        }
        Action::ChartSearchLast => {
            if !app.chart_search_results.is_empty() {
                app.chart_search_selected = app.chart_search_results.len() - 1;
            }
        }
        Action::ChartSearchFocusNext => {
            #[cfg(feature = "tui-interact")]
            app.chart_search_interact.tab_next();
        }
        Action::ChartSearchFocusPrev => {
            #[cfg(feature = "tui-interact")]
            app.chart_search_interact.tab_prev();
        }
        Action::ChartPillLeft => {
            if app.chart_pill_row == 0 {
                app.chart_expiry_index = app.chart_expiry_index.saturating_sub(1);
            }
        }
        Action::ChartPillRight => {
            if app.chart_pill_row == 0 {
                let max = crate::ui::charts::chart_expiry_max_index(app);
                if app.chart_expiry_index < max {
                    app.chart_expiry_index += 1;
                }
            }
        }
        Action::ChartPillUp => {
            if app.chart_pill_row > 0 {
                app.chart_pill_row -= 1;
            } else {
                app.chart_pill_row = 1;
            }
        }
        Action::ChartPillDown => {
            if app.chart_pill_row < 1 {
                app.chart_pill_row += 1;
            }
        }
        Action::ChartPillSelect => {
            if app.chart_pill_row == 1 {
                let n = crate::ui::charts::CHART_STRIKE_PILL_COUNT;
                app.chart_strike_index = (app.chart_strike_index + 1) % n;
            }
        }
        Action::LogScrollUp => {
            app.log_state.transition(tui_logger::TuiWidgetEvent::UpKey);
        }
        Action::LogScrollDown => {
            app.log_state
                .transition(tui_logger::TuiWidgetEvent::DownKey);
        }
        Action::LogPageUp => {
            app.log_state
                .transition(tui_logger::TuiWidgetEvent::PrevPageKey);
        }
        Action::LogPageDown => {
            app.log_state
                .transition(tui_logger::TuiWidgetEvent::NextPageKey);
        }
        Action::LogLevelUp => {
            app.log_state
                .transition(tui_logger::TuiWidgetEvent::PlusKey);
            app.log_display_level = match app.log_display_level {
                log::LevelFilter::Trace => log::LevelFilter::Debug,
                log::LevelFilter::Debug => log::LevelFilter::Info,
                log::LevelFilter::Info => log::LevelFilter::Warn,
                log::LevelFilter::Warn => log::LevelFilter::Error,
                log::LevelFilter::Error => log::LevelFilter::Error,
                _ => app.log_display_level,
            };
            tui_logger::set_default_level(app.log_display_level);
        }
        Action::LogLevelDown => {
            app.log_state
                .transition(tui_logger::TuiWidgetEvent::MinusKey);
            app.log_display_level = match app.log_display_level {
                log::LevelFilter::Error => log::LevelFilter::Warn,
                log::LevelFilter::Warn => log::LevelFilter::Info,
                log::LevelFilter::Info => log::LevelFilter::Debug,
                log::LevelFilter::Debug => log::LevelFilter::Trace,
                log::LevelFilter::Trace => log::LevelFilter::Trace,
                _ => app.log_display_level,
            };
            tui_logger::set_default_level(app.log_display_level);
        }
        Action::LogHide => {
            app.log_state
                .transition(tui_logger::TuiWidgetEvent::HideKey);
        }
        Action::LogEscape => {
            app.log_state
                .transition(tui_logger::TuiWidgetEvent::EscapeKey);
        }
        Action::LogLevelError => {
            app.log_display_level = log::LevelFilter::Error;
            tui_logger::set_default_level(log::LevelFilter::Error);
        }
        Action::LogLevelWarn => {
            app.log_display_level = log::LevelFilter::Warn;
            tui_logger::set_default_level(log::LevelFilter::Warn);
        }
        Action::LogLevelInfo => {
            app.log_display_level = log::LevelFilter::Info;
            tui_logger::set_default_level(log::LevelFilter::Info);
        }
        Action::LogLevelDebug => {
            app.log_display_level = log::LevelFilter::Debug;
            tui_logger::set_default_level(log::LevelFilter::Debug);
        }
        Action::TreeUp => {
            app.tree_state.borrow_mut().key_up();
        }
        Action::TreeDown => {
            app.tree_state.borrow_mut().key_down();
        }
        Action::TreeLeft => {
            app.tree_state.borrow_mut().key_left();
        }
        Action::TreeRight => {
            app.tree_state.borrow_mut().key_right();
        }
        Action::TreeToggle => {
            app.tree_state.borrow_mut().toggle_selected();
        }
        Action::TreeEscape => {
            app.show_tree_panel = false;
        }
        _ => return false,
    }
    true
}
