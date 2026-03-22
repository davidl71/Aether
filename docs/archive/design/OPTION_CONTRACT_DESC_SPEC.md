# Option Contract String Parsing Spec

**Purpose:** Single documented spec for option contract description format so C++ and Python (and any other consumers) follow the same parsing rules. Aligns with Logic Unify C3 and `docs/design/LOGIC_WE_COULD_UNIFY.md` §5.

## IB contractDesc (Client Portal / TWS)

Used by IBKR API and returned in position/snapshot payloads. Format:

- **Pattern:** `SYMBOL EXPIRY STRIKE RIGHT [OCC_SUFFIX]`
- **SYMBOL:** Underlying symbol (e.g. `SPX`, `XSP`). Non-whitespace.
- **EXPIRY:** `MMMYYYY` (e.g. `MAR2027`, `DEC2025`). Word chars + 4-digit year.
- **STRIKE:** Numeric strike (integer or decimal, e.g. `6825`, `3950.5`).
- **RIGHT:** `C` (call) or `P` (put). Case-insensitive; normalize to uppercase.
- **OCC suffix:** Optional `[SYMBOL YYMMDDC/P...]` in square brackets; may be ignored for symbol/expiry/strike/right extraction.

**Regex (canonical):**  
`^\s*(\S+)\s+(\w+\d{4})\s+(\d+(?:\.\d+)?)\s+([CP])\s+`  
Groups: 1=symbol, 2=expiry, 3=strike, 4=right.

**Examples:**

| contractDesc | symbol | expiry | strike | right |
|--------------|--------|--------|--------|-------|
| `SPX    MAR2027 6825 C [SPX   270319C06825000 100]` | SPX | MAR2027 | 6825 | C |
| `SPX    MAR2027 6950 P [SPX   270319P06950000 100]` | SPX | MAR2027 | 6950 | P |
| `XSP  DEC2025 4000 C` | XSP | DEC2025 | 4000 | C |

**Parsing rules:**

1. Strip leading/trailing whitespace.
2. Match the regex; if no match, return null/None/false.
3. Strike: parse as float; reject if not a valid number.
4. Right: accept only `C` or `P` (case-insensitive); normalize to `C`/`P`.
5. Symbol and expiry: use as-is (trimmed). No embedded spaces in symbol.

**Box spread synthetic descriptor:**  
For display or logging, a box can be represented as `{symbol} {expiry} {k1}/{k2} box` (e.g. `SPX MAR2027 6825/6950 box`). Parsing this is optional; the canonical parse target is single-leg contractDesc.

## OCC option symbol (reference)

OCC standard format (e.g. for Alpaca, some APIs):  
`SYMBOL + YYMMDD + C/P + 8-digit strike (padded) + optional multiplier`  
Example: `SPX   270319C06825000 100`.  
This spec does not require parsing OCC format in the same code path; when contractDesc contains an OCC suffix in brackets, it is optional to parse it. Primary contract identity for box-spread matching is (symbol, expiry, strike, right) from the main part of contractDesc.

## Implementation notes

- **Python:** Historical docs referenced `combo_detector.parse_opt_contract_desc(contract_desc)` returning `(symbol, expiry, strike, right)` or `None`.
- **C++:** The preferred direction is a canonical native parser exposed via pybind11 where Python/helper consumers need it.
- **Proto:** `proto/messages.proto` defines `OptionContract` with symbol, expiry, strike, option_type (C/P). Serialization of parsed result should align with that message.

## References

- `docs/design/LOGIC_WE_COULD_UNIFY.md` §5 (option/contract string parsing)
- legacy Python parser notes in older docs
- `proto/messages.proto` – `OptionContract`, `BoxSpreadLeg`
