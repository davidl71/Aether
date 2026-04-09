# TUI: loans bulk import from PDF (deferred)

**Task:** T-1774864120067507000  
**Status:** Deferred — design note only; no PDF pipeline in tree.

## Scope

- **Goal (future):** Let operators point the Loans tab bulk-import flow at a **bank-statement PDF** and derive rows that match `LoanRecord` / existing bulk import semantics, then submit via the same backend path as JSON/CSV.
- **Out of scope now:** Full PDF parsing, OCR, bank-specific layouts, or legal/compliance review of statement handling.

## Product constraints

- Aether’s default direction is **read-only exploration** (`docs/DATA_EXPLORATION_MODE.md`). Any PDF work must not reintroduce execution or live-trading paths.
- Today’s bulk import is **structured files only** (JSON/CSV). PDF → structured loans is **extract + normalize + validate**; treat extracted data as untrusted until validated against the same rules as CSV/JSON (`api::loans::parse_loans_import_file` and NATS `api.loans.import_bulk`).

## Crates to evaluate later (Rust ecosystem)

| Area | Candidates (evaluate license, MSRV, and maintenance) |
|------|------------------------------------------------------|
| PDF text/layout | `lopdf`, `pdf-extract`, `pdfium-render` (PDFium bindings) |
| Tables / layout-heavy PDFs | Often needs **PDFium** or **Poppler**-class tooling; pure-Rust may be insufficient for messy statements |
| OCR (scanned statements) | `tesseract` / system OCR — heavy dependency; only if product explicitly needs scans |

Prefer a **small extraction step** that outputs an intermediate CSV/JSON checked by existing parsers over baking PDF logic into the TUI.

## Integration points (`tui_service`)

| Location | Role |
|----------|------|
| `src/input_loans.rs` | `Action::LoansBulkImportFocus`, path buffer, `LoansImportPathEnter` → sends `PathBuf` on `loan_bulk_import_tx` |
| `src/main.rs` | `run_loan_bulk_importer`: reads file as UTF-8 text → `api::loans::parse_loans_import_file` → NATS `topics::api::loans::IMPORT_BULK` |
| `src/ui/loans.rs` | Loans tab UI, bulk-import path overlay, hints (`b` / `i`) |
| `src/app.rs` | `loan_import_path`, `loan_bulk_import_tx`, `loans_bulk_import_inflight` |
| `crates/api/src/loans.rs` | `parse_loans_import_file` / CSV+JSON validation — **reuse** after PDF extraction |

A future PDF path would likely add a **pre-step** (binary PDF → text or CSV) before the existing importer, or a parallel worker that emits the same `ParsedLoansImport` shape.

## References

- `docs/DATA_EXPLORATION_MODE.md` — read-only product stance  
- `docs/TUI_RATATUI_INTERACT.md` — optional `tui-interact` sub-focus on import path
