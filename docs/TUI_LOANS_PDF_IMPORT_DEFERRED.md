# TUI: loans bulk import from PDF (deferred)

**Task:** T-1774864120067507000  
**Status:** Deferred — design note only; no PDF pipeline in tree.

## Scope boundaries (concrete)

### In scope when/if implemented

- **Operator workflow:** Loans tab continues to use the same entry points as today (`b` / `i` for bulk path, or a clearly labeled future variant) so the mental model stays “bulk import → validate → `api.loans.import_bulk`”.
- **Output contract:** Any PDF path must produce rows that conform to existing bulk semantics: `LoanRecord` validation (`LoanRecord::validate`), CSV/JSON parsing rules in `api::loans` (`parse_loans_import_file`, `ParsedLoansImport`), and NATS `api.loans.import_bulk` — no parallel loan schema.
- **Trust model:** Extracted values are **untrusted input** until they pass the same validation as hand-prepared JSON/CSV; partial imports and per-row errors should follow existing `BulkImportRowError` / bulk-summary behavior where applicable.
- **Product stance:** Stays aligned with read-only exploration (`docs/DATA_EXPLORATION_MODE.md`): import is data ingest for the ledger/read model, not execution or trading.

### Explicitly out of scope (this task / v1 PDF)

- **Full generic PDF extraction:** No commitment to “any bank PDF worldwide”; bank-specific templates, table detection on arbitrary layouts, and production-grade layout engines are deferred unless scoped per bank/version.
- **OCR and scanned statements:** Image-based PDFs, `tesseract` or similar, GPU/CLI OCR pipelines, and quality guarantees for scans are **out of scope** unless product explicitly funds a separate “scanned statements” epic.
- **Legal / compliance / retention:** How long to retain source PDFs, PII handling policies, and regulatory sign-off are not defined here.
- **REST surface:** Loans remain NATS-first per `api` crate docs; PDF parsing does not imply a new REST API by itself.

### Boundary diagram (logical)

```text
PDF file  →  [optional: extract/normalize step]  →  UTF-8 text / CSV-shaped payload
                                                      ↓
                              parse_loans_import_file*  →  LoanRecord[] + errors
                                                      ↓
                              api.loans.import_bulk  →  SQLite `loans` + existing UX refresh
```

\*Or equivalent: build `Vec<LoanRecord>` in code only if every field is validated before send.

## Suggested Rust dependencies (evaluation list)

Use this as a **shortlist for spike work**, not as locked-in Cargo.toml entries. Before adding any crate: license, MSRV vs workspace, binary/native deps (PDFium, Poppler), cross-platform story, and maintenance cadence.

| Concern | Suggested crates / approaches | Notes |
|--------|-------------------------------|--------|
| Low-level PDF structure | `lopdf` | Good for inspection and some text extraction; may be insufficient for complex statement layouts. |
| Text extraction (Rust) | `pdf-extract` | Higher-level text pull; quality varies by PDF producer. |
| Layout-accurate rendering | `pdfium-render` (PDFium) | Heavy native dependency; often needed for “real” bank PDFs with tables. |
| External tooling (non-Rust) | Poppler (`pdftotext`), mutool | Sometimes the pragmatic path for a **batch pre-step** that emits CSV checked by existing parsers. |

**PDF extract / OCR note:** The **first shippable slice** should assume **text-based PDFs** (or a manual pre-conversion to CSV/JSON). Full in-process PDF layout parsing and **any OCR** are explicitly **stretch goals** and should not block definition of `LoanRecord`-compatible import.

Prefer a **small extraction step** (CLI tool, sidecar, or isolated crate) that outputs intermediate CSV/JSON checked by existing parsers over baking PDF logic into the TUI.

## Data model touchpoints (`loans` tab and backend)

### Canonical type: `LoanRecord` (`agents/backend/crates/api/src/loans.rs`)

Bulk import ultimately targets this struct (serde-stable wire shape). PDF mapping must populate these fields (or a subset that still passes validation — all required fields per `validate()`):

- Identifiers and parties: `loan_id`, `bank_name`, `account_number`
- Classification: `loan_type` (`SHIR_BASED` / `CPI_LINKED`), `status` (`ACTIVE` / `PAID_OFF` / `DEFAULTED`)
- Amounts: `principal`, `original_principal`, `monthly_payment`, `interest_rate`, `spread`, `base_cpi`, `current_cpi` (CPI-linked rules apply)
- Dates (RFC3339 or `YYYY-MM-DD` per parser): `origination_date`, `maturity_date`, `next_payment_date`, `last_update`
- Other: `payment_frequency_months`, `currency` (default ILS)

### Persistence and API

- **SQLite:** `LoanRepository` / `loans` table schema in the same module — imported rows land here like CSV/JSON imports.
- **NATS:** Subjects under `api.loans.*` (see `docs/platform/NATS_API.md`); bulk path uses `import_bulk` with `LoansBulkImportRequest { loans: Vec<LoanRecord> }`.
- **Parsing helpers:** `parse_loans_import_file`, `parse_loans_import_file_json`, `ParsedLoansImport` — reuse after extraction rather than duplicating validation rules.

### TUI (`tui_service`) — where the operator sees loans

| Location | Role |
|----------|------|
| `src/ui/loans.rs` | Renders Loans tab (`api.loans.list`), bulk-import path overlay (“Bulk import loans (`api.loans.import_bulk`)”), hints `n=new`, `b/i=bulk JSON` (CSV also supported by backend). |
| `src/input_loans.rs` | Actions for scroll, new loan form, bulk import path focus, Enter/Esc for path submission and cancel. |
| `src/app.rs` | `loans_list`, `loans_fetch_pending`, `loan_import_path`, `loan_bulk_import_tx`, `loans_bulk_import_inflight`, `loan_entry`, table selection state. |
| `src/main.rs` | `run_loan_bulk_importer`: reads file as UTF-8 text → `parse_loans_import_file` → NATS import. A PDF flow would insert **before** this or feed the same pipeline with transformed text. |

Optional: `docs/TUI_RATATUI_INTERACT.md` — `tui-interact` sub-focus on the import path field (`loan_import_interact`).

### Aggregation (read path)

- `LoanRecord::to_aggregation_input` → `LoanAggregationInput` for portfolio-style views; PDF import does not change this if `LoanRecord` stays canonical.

## Acceptance sketch (when picking up the work)

Use as a checklist for a future implementation ticket; wording is intentionally testable.

1. **Input:** Operator can select a PDF path from the Loans tab (or documented alternative entry) without breaking existing JSON/CSV path behavior.
2. **Extraction:** For agreed sample statement PDFs (text-based), the pipeline produces at least one valid `LoanRecord` per expected loan row, or surfaces a clear, actionable error (no silent drop of required fields).
3. **Validation:** All persisted rows pass `LoanRecord::validate`; behavior matches CSV/JSON bulk import for duplicates, partial failure, and NATS error propagation.
4. **UX:** After successful import, loans list refresh matches current bulk import (`loans_list` / fetch pending / inflight flags) and status bar or toasts stay consistent with existing patterns.
5. **Scope honesty:** Documentation states which bank/layout versions are supported; OCR and scanned PDFs remain explicitly unsupported unless a follow-up task adds them.
6. **Safety:** No new live-trading or order paths; no storage of secrets or credentials in import artifacts.

## Product constraints

- Aether’s default direction is **read-only exploration** (`docs/DATA_EXPLORATION_MODE.md`). Any PDF work must not reintroduce execution or live-trading paths.
- Today’s bulk import is **structured files only** (JSON/CSV). PDF → structured loans is **extract + normalize + validate**; treat extracted data as untrusted until validated against the same rules as CSV/JSON (`api::loans::parse_loans_import_file` and NATS `api.loans.import_bulk`).

## Integration points (`tui_service`) — summary

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
- `docs/platform/NATS_API.md` — `api.loans.*` subjects  
- `AGENTS.md` — canonical repository guidelines  
