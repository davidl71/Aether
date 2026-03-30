#!/usr/bin/env Rscript
# sanity_box_spread.R — batch sanity checks for box-spread export v1 (CSV or JSON).
# Usage: Rscript sanity_box_spread.R <path.csv|path.json>
# CSV path uses only base R. JSON requires jsonlite.

args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 1L) {
  stop("usage: Rscript sanity_box_spread.R <export.csv|export.json>")
}

path <- args[[1L]]
ext <- tolower(tools::file_ext(path))

read_export <- function(p) {
  if (ext == "csv") {
    df <- read.csv(p, stringsAsFactors = FALSE, check.names = FALSE)
    return(df)
  }
  if (ext == "json") {
    if (!requireNamespace("jsonlite", quietly = TRUE)) {
      stop("JSON input requires package jsonlite: install.packages('jsonlite')")
    }
    raw <- jsonlite::fromJSON(p, simplifyVector = TRUE)
    if (is.null(raw$rows)) {
      stop("JSON must contain top-level 'rows' array")
    }
    as.data.frame(raw$rows, stringsAsFactors = FALSE)
  } else {
    stop("unsupported extension: use .csv or .json")
  }
}

df <- read_export(path)

required <- c(
  "schema_version", "symbol", "k_long_call", "k_short_call",
  "k_long_put", "k_short_put", "premium_net", "t_years", "r"
)
miss <- setdiff(required, names(df))
if (length(miss)) {
  stop("missing columns: ", paste(miss, collapse = ", "))
}

if (!all(df$schema_version == 1)) {
  stop("only schema_version 1 is supported in this spike")
}

box_width <- with(df, abs(k_short_call - k_long_call))
if (!all(box_width == with(df, abs(k_long_put - k_short_put)))) {
  warning("call spread width != put spread width; verify box construction")
}

theoretical_pv <- box_width * exp(-df$r * df$t_years)

cat("=== box spread export sanity ===\n")
cat(sprintf("rows: %d\n", nrow(df)))
for (i in seq_len(nrow(df))) {
  row <- df[i, , drop = FALSE]
  w <- box_width[[i]]
  tpv <- theoretical_pv[[i]]
  prem <- row$premium_net[[1]]
  cat(sprintf(
    "\n[%s] width=%g discounted_width=%.6f premium_net=%.6f\n",
    row$symbol[[1]], w, tpv, prem
  ))
  if (prem > tpv + 1e-6) {
    message("  note: premium_net exceeds theoretical upper bound (checksign/units)")
  }
  if (prem < -1e-6) {
    message("  note: negative premium_net (credit box?) — verify semantics")
  }
}
invisible(TRUE)
