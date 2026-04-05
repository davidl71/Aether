//! Loans tab input: list navigation plus loan entry/import overlays.
//!
//! Keeps the loans form + import-path state transitions isolated from the rest of the
//! input router.

use std::path::PathBuf;

use crossterm::event::KeyCode;

use crate::app::{App, LoanEntryState, Tab};
use crate::input::Action;
use crate::ui::ToastLevel;

pub(crate) fn loan_form_key_action(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Esc => Some(Action::LoansInputEscape),
        KeyCode::Enter => Some(Action::LoansInputEnter),
        KeyCode::Tab => Some(Action::LoansInputNavDown),
        KeyCode::BackTab => Some(Action::LoansInputNavUp),
        KeyCode::Up => Some(Action::LoansInputNavUp),
        KeyCode::Down => Some(Action::LoansInputNavDown),
        KeyCode::Backspace => Some(Action::LoansInputBackspace),
        KeyCode::Char(c) if c.is_ascii_digit() || c == '-' || c == '.' => {
            Some(Action::LoansInputChar(c))
        }
        KeyCode::Char(c) if c.is_alphabetic() => Some(Action::LoansInputChar(c)),
        _ => Some(Action::NoOp),
    }
}

pub(crate) fn apply_loan_action(app: &mut App, action: Action) -> bool {
    match action {
        Action::LoansBulkImportFocus => {
            app.loan_entry = None;
            app.active_tab = Tab::Loans;
            app.loan_import_path = Some(String::new());
            #[cfg(feature = "tui-interact")]
            app.loan_import_interact.on_open();
            app.request_loans_fetch_if_uncached();
        }
        Action::LoansImportPathEscape => {
            #[cfg(feature = "tui-interact")]
            app.loan_import_interact.on_close();
            app.loan_import_path = None;
        }
        Action::LoansImportPathChar(c) => {
            if let Some(ref mut buf) = app.loan_import_path {
                const MAX: usize = 4096;
                if buf.len() < MAX {
                    buf.push(c);
                }
            }
        }
        Action::LoansImportPathBackspace => {
            if let Some(ref mut buf) = app.loan_import_path {
                buf.pop();
            }
        }
        Action::LoansImportPathEnter => {
            let Some(buf) = app.loan_import_path.take() else {
                return true;
            };
            let trimmed = buf.trim();
            if trimmed.is_empty() {
                app.push_toast(
                    "Enter a path to a loans JSON or CSV file.",
                    ToastLevel::Warning,
                );
                app.loan_import_path = Some(buf);
                return true;
            }

            let path = PathBuf::from(trimmed);
            if !path.exists() {
                app.push_toast("File does not exist.", ToastLevel::Warning);
                app.loan_import_path = Some(buf);
                return true;
            }
            if !path.is_file() {
                app.push_toast("Path is not a file.", ToastLevel::Warning);
                app.loan_import_path = Some(buf);
                return true;
            }

            if let Some(ref tx) = app.loan_bulk_import_tx {
                if tx.send(path).is_ok() {
                    #[cfg(feature = "tui-interact")]
                    app.loan_import_interact.on_close();
                    app.loans_bulk_import_inflight = true;
                    app.loans_fetch_pending = true;
                    app.push_toast(
                        "Bulk import started (read JSON/CSV → api.loans.import_bulk)…",
                        ToastLevel::Info,
                    );
                } else {
                    app.push_toast("Bulk import channel closed.", ToastLevel::Error);
                }
            } else {
                app.push_toast("Loans bulk import is not wired.", ToastLevel::Error);
            }
        }
        Action::LoansScrollUp => {
            app.loans_table.move_up();
        }
        Action::LoansScrollDown => {
            let len = app
                .loans_list
                .as_ref()
                .and_then(|r| r.as_ref().ok())
                .map(|l| l.len())
                .unwrap_or(0);
            app.loans_table.move_down(len);
        }
        Action::LoansScrollPageUp => {
            let len = app
                .loans_list
                .as_ref()
                .and_then(|r| r.as_ref().ok())
                .map(|l| l.len())
                .unwrap_or(0);
            app.loans_table.shift_selected(-10, len);
        }
        Action::LoansScrollPageDown => {
            let len = app
                .loans_list
                .as_ref()
                .and_then(|r| r.as_ref().ok())
                .map(|l| l.len())
                .unwrap_or(0);
            app.loans_table.shift_selected(10, len);
        }
        Action::LoansNewLoan => {
            #[cfg(feature = "tui-interact")]
            if app.loan_import_path.is_some() {
                app.loan_import_interact.on_close();
            }
            app.loan_import_path = None;
            app.loan_entry = Some(LoanEntryState::new());
        }
        Action::LoanImportFocusNext => {
            #[cfg(feature = "tui-interact")]
            app.loan_import_interact.tab_next();
        }
        Action::LoanImportFocusPrev => {
            #[cfg(feature = "tui-interact")]
            app.loan_import_interact.tab_prev();
        }
        Action::LoansInputEscape => {
            app.loan_entry = None;
        }
        Action::LoansInputEnter => {
            if let Some(ref mut entry) = app.loan_entry {
                if entry.current_field == 2 {
                    entry.toggle_loan_type();
                    entry.validation_error = None;
                } else {
                    entry.calculate_maturity();
                    entry.calculate_monthly_payment();
                    if entry.is_complete() {
                        if let Some(loan_record) = entry.to_loan_record() {
                            if let Some(ref tx) = app.loan_create_tx {
                                let _ = tx.send(loan_record);
                            }
                            app.loan_entry = None;
                        }
                    } else {
                        entry.validation_error =
                            Some("Missing or invalid required fields".to_string());
                    }
                }
            }
        }
        Action::LoansInputNavUp => {
            if let Some(ref mut entry) = app.loan_entry {
                entry.validation_error = None;
                loop {
                    if entry.current_field > 0 {
                        entry.current_field -= 1;
                    } else {
                        entry.current_field = 9;
                    }
                    if entry.current_field < 10 {
                        break;
                    }
                }
            }
        }
        Action::LoansInputNavDown => {
            if let Some(ref mut entry) = app.loan_entry {
                entry.validation_error = None;
                loop {
                    if entry.current_field < 9 {
                        entry.current_field += 1;
                    } else {
                        entry.current_field = 0;
                    }
                    if entry.current_field < 10 {
                        break;
                    }
                }
            }
        }
        Action::LoansInputChar(c) => {
            if let Some(ref mut entry) = app.loan_entry {
                let field = entry.current_field;
                let max_len = match field {
                    0 => 50,
                    1 => 20,
                    2 => 3,
                    3 => 15,
                    4 => 6,
                    5 => 6,
                    6 => 10,
                    7 => 10,
                    8 => 5,
                    9 => 5,
                    _ => 20,
                };
                let target = match field {
                    0 => &mut entry.bank_name,
                    1 => &mut entry.account_number,
                    2 => return true,
                    3 => &mut entry.principal,
                    4 => &mut entry.interest_rate,
                    5 => &mut entry.spread,
                    6 => &mut entry.origination_date,
                    7 => &mut entry.first_payment_date,
                    8 => &mut entry.num_payments,
                    9 => &mut entry.currency,
                    _ => return true,
                };
                if target.len() < max_len {
                    target.push(c);
                }
                entry.calculate_maturity();
                entry.calculate_monthly_payment();
            }
        }
        Action::LoansInputBackspace => {
            if let Some(ref mut entry) = app.loan_entry {
                let field = entry.current_field;
                let target = match field {
                    0 => &mut entry.bank_name,
                    1 => &mut entry.account_number,
                    2 => return true,
                    3 => &mut entry.principal,
                    4 => &mut entry.interest_rate,
                    5 => &mut entry.spread,
                    6 => &mut entry.origination_date,
                    7 => &mut entry.first_payment_date,
                    8 => &mut entry.num_payments,
                    9 => &mut entry.currency,
                    _ => return true,
                };
                target.pop();
                entry.calculate_maturity();
                entry.calculate_monthly_payment();
            }
        }
        Action::DiscountBankScrollUp => {
            app.discount_bank_table.move_up();
        }
        Action::DiscountBankScrollDown => {
            let len = app
                .discount_bank_transactions
                .as_ref()
                .and_then(|r| r.as_ref().ok())
                .map(|t| t.transactions.len())
                .unwrap_or(0);
            app.discount_bank_table.move_down(len);
        }
        Action::DiscountBankScrollPageUp => {
            let len = app
                .discount_bank_transactions
                .as_ref()
                .and_then(|r| r.as_ref().ok())
                .map(|t| t.transactions.len())
                .unwrap_or(0);
            app.discount_bank_table.shift_selected(-10, len);
        }
        Action::DiscountBankScrollPageDown => {
            let len = app
                .discount_bank_transactions
                .as_ref()
                .and_then(|r| r.as_ref().ok())
                .map(|t| t.transactions.len())
                .unwrap_or(0);
            app.discount_bank_table.shift_selected(10, len);
        }
        Action::DiscountBankRefresh => {
            app.request_discount_bank_fetch();
        }
        Action::SettingsDelete => {
            if app.settings_section != crate::workspace::SettingsSection::Symbols {
                return false;
            }
            let wl = app.watchlist();
            if !wl.is_empty() && app.settings_symbol_index < wl.len() {
                let mut list = app
                    .watchlist_override
                    .clone()
                    .unwrap_or_else(|| app.config.watchlist.clone());
                list.remove(app.settings_symbol_index);
                let new_len = list.len();
                app.watchlist_override = Some(list);
                app.settings_symbol_index =
                    app.settings_symbol_index.min(new_len.saturating_sub(1));
                app.push_toast("Symbol removed from watchlist.", ToastLevel::Info);
            }
        }
        Action::SettingsReset => {
            app.watchlist_override = None;
            app.push_toast("Watchlist reset to config.", ToastLevel::Info);
        }
        _ => return false,
    }
    true
}
