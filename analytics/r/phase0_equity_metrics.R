#!/usr/bin/env Rscript
# Phase 0: load equity OHLCV-like bars from CSV, compute simple buy-and-hold metrics.
# Full box-spread / quantstrat replay is deferred (T-1774201866539327000); this proves the R sidecar path.

argv <- commandArgs(trailingOnly = FALSE)
file_flags <- grep("^--file=", argv, value = TRUE)
script_dir <- if (length(file_flags)) {
  dirname(normalizePath(sub("^--file=", "", file_flags[[1]]), winslash = "/"))
} else {
  getwd()
}
args <- commandArgs(trailingOnly = TRUE)
default_csv <- file.path(script_dir, "fixtures", "sample_equity_bars_v1.csv")
path <- if (length(args) >= 1) args[[1]] else default_csv

if (!file.exists(path)) {
  stop("CSV not found: ", path)
}

raw <- read.csv(path, stringsAsFactors = FALSE)
need <- c("date", "close")
miss <- setdiff(need, names(raw))
if (length(miss) > 0) {
  stop("CSV must include columns: ", paste(need, collapse = ", "), " (missing: ", paste(miss, collapse = ", "), ")")
}

raw$date <- as.Date(raw$date)
raw <- raw[order(raw$date), ]
px <- raw$close
n <- length(px)
if (n < 3) {
  stop("Need at least 3 rows for return stats")
}

r <- diff(log(px))
mu <- mean(r)
sdv <- sd(r)
sharpe <- if (is.finite(sdv) && sdv > 0) sqrt(252) * mu / sdv else NA_real_

equity <- cumprod(c(1, exp(r)))
peak <- cummax(equity)
dd <- (equity - peak) / peak
max_dd <- min(dd)

cat("=== phase0_equity_metrics ===\n")
cat("file:", path, "\n")
cat("rows:", n, " trading_days_estimate:", length(r), "\n")
cat("total_return_pct:", sprintf("%.4f", 100 * (equity[length(equity)] - 1)), "\n")
cat("ann_vol_log_returns_pct:", sprintf("%.4f", 100 * sdv * sqrt(252)), "\n")
cat("sharpe_annualized_log:", sprintf("%.4f", sharpe), "\n")
cat("max_drawdown_pct:", sprintf("%.4f", 100 * max_dd), "\n")
invisible(list(equity = equity, sharpe = sharpe, max_drawdown = max_dd))
