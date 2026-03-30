use crate::app::{App, CommandStatusView, DetailPopupContent, Tab};
use crate::input::Action;

pub(crate) fn apply_view_action(app: &mut App, action: Action) -> bool {
    match action {
        Action::OrdersScrollUp => {
            app.orders_scroll = app.orders_scroll.saturating_sub(1);
        }
        Action::OrdersScrollDown => {
            let len = app.filtered_orders_len();
            if len > 0 {
                app.orders_scroll = (app.orders_scroll + 1).min(len - 1);
            }
        }
        Action::OrdersScrollPageUp => {
            app.orders_scroll = app.orders_scroll.saturating_sub(10);
        }
        Action::OrdersScrollPageDown => {
            let len = app.filtered_orders_len();
            if len > 0 {
                app.orders_scroll = (app.orders_scroll + 10).min(len - 1);
            }
        }
        Action::OrdersDetail => {
            if let Some(ref snap) = app.snapshot() {
                let filtered = app.filtered_orders(snap);
                let idx = app.orders_scroll.min(filtered.len().saturating_sub(1));
                if let Some(order) = filtered.get(idx) {
                    app.detail_popup = Some(DetailPopupContent::Order(order.clone()));
                }
            }
        }
        Action::OrdersFilterFocus => {
            app.order_filter_active = true;
            app.set_command_status(CommandStatusView::success(
                "orders_filter",
                "Filter mode active: type symbol, status, or side; Esc to exit.",
            ));
        }
        Action::OrdersFilterChar(c) => {
            app.order_filter_active = true;
            app.order_filter.push(c);
        }
        Action::OrdersFilterBackspace => {
            app.order_filter.pop();
        }
        Action::OrdersFilterClear => {
            app.order_filter.clear();
            app.order_filter_active = false;
            app.set_command_status(CommandStatusView::success(
                "orders_filter",
                "Filter cleared.",
            ));
        }
        Action::OrdersCancel => {
            app.set_command_status(CommandStatusView::disabled("cancel_all"));
        }
        Action::AlertsScrollUp => {
            app.alerts_scroll = app.alerts_scroll.saturating_sub(1);
        }
        Action::AlertsScrollDown => {
            let len = app
                .snapshot()
                .as_ref()
                .map(|s| s.dto().alerts.len())
                .unwrap_or(0);
            if len > 0 {
                app.alerts_scroll = (app.alerts_scroll + 1).min(len - 1);
            }
        }
        Action::AlertsScrollPageUp => {
            app.alerts_scroll = app.alerts_scroll.saturating_sub(10);
        }
        Action::AlertsScrollPageDown => {
            let len = app
                .snapshot()
                .as_ref()
                .map(|s| s.dto().alerts.len())
                .unwrap_or(0);
            if len > 0 {
                app.alerts_scroll = (app.alerts_scroll + 10).min(len - 1);
            }
        }
        Action::DashboardScrollUp => {
            app.dashboard_scroll = app.dashboard_scroll.saturating_sub(1);
        }
        Action::DashboardScrollDown => {
            let len = app
                .snapshot()
                .as_ref()
                .map(|s| s.inner.symbols.len())
                .unwrap_or(0);
            if len > 0 {
                app.dashboard_scroll = (app.dashboard_scroll + 1).min(len - 1);
            }
        }
        Action::DashboardNavigateToChart => {
            let symbol = app.snapshot().as_ref().and_then(|snap| {
                let idx = app
                    .dashboard_scroll
                    .min(snap.inner.symbols.len().saturating_sub(1));
                snap.inner.symbols.get(idx).map(|s| s.symbol.clone())
            });
            if let Some(symbol) = symbol {
                app.active_tab = Tab::Charts;
                app.symbol_for_chart = symbol;
                app.chart_search_visible = false;
                app.chart_search_input.clear();
            }
        }
        Action::ScenariosScrollUp => {
            app.scenarios_scroll = app.scenarios_scroll.saturating_sub(1);
        }
        Action::ScenariosScrollDown => {
            let filtered = crate::ui::filtered_scenarios(app);
            if !filtered.is_empty() {
                app.scenarios_scroll =
                    (app.scenarios_scroll + 1).min(filtered.len().saturating_sub(1));
            }
        }
        Action::ScenariosScrollPageUp => {
            app.scenarios_scroll = app.scenarios_scroll.saturating_sub(10);
        }
        Action::ScenariosScrollPageDown => {
            let filtered = crate::ui::filtered_scenarios(app);
            if !filtered.is_empty() {
                app.scenarios_scroll =
                    (app.scenarios_scroll + 10).min(filtered.len().saturating_sub(1));
            }
        }
        Action::ScenariosDetail => {
            let filtered = crate::ui::filtered_scenarios(app);
            let idx = app.scenarios_scroll.min(filtered.len().saturating_sub(1));
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
            app.set_command_status(CommandStatusView::disabled("scenario_run"));
        }
        Action::ScenariosCycleStrikeWidth => {
            app.scenarios_strike_width_filter = match app.scenarios_strike_width_filter {
                None => Some(25),
                Some(25) => Some(50),
                Some(50) => Some(100),
                Some(_) => None,
            };
        }
        Action::ChartSearchFocus => {
            app.chart_search_visible = true;
            app.chart_search_input.clear();
            app.chart_search_results.clear();
            app.chart_search_selected = 0;
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
                        if app.chart_search_history.len() > 10 {
                            app.chart_search_history.pop_back();
                        }
                    }
                }
            } else if !app.chart_search_input.is_empty() {
                app.symbol_for_chart = app.chart_search_input.clone();
                if !app.chart_search_history.contains(&app.chart_search_input) {
                    app.chart_search_history
                        .push_front(app.chart_search_input.clone());
                    if app.chart_search_history.len() > 10 {
                        app.chart_search_history.pop_back();
                    }
                }
            }
            app.chart_search_visible = false;
            app.chart_search_input.clear();
        }
        Action::ChartSearchEscape => {
            app.chart_search_visible = false;
            app.chart_search_input.clear();
            app.chart_search_results.clear();
        }
        Action::ChartPillLeft => {
            if app.chart_pill_row == 0 {
                app.chart_expiry_index = app.chart_expiry_index.saturating_sub(1);
            }
        }
        Action::ChartPillRight => {
            if app.chart_pill_row == 0 {
                app.chart_expiry_index += 1;
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
                app.chart_strike_index = (app.chart_strike_index + 1) % 5;
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
