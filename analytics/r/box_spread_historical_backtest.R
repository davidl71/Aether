#!/usr/bin/env Rscript
# Historical backtest MVP for one long box position marked to CSV MTM series (T-1774201866539327000).
# Input: exported daily marks (QuestDB or Rust → CSV). Columns: date, box_mtm (USD).
# Output: cumulative P&L, total return, annualized Sharpe (log returns), max drawdown.
# Optional: ggplot2 PNG, plotly HTML, quantstrat path (set AETHER_QUANTSTRAT=1).

argv <- commandArgs(trailingOnly = FALSE)
file_flags <- grep("^--file=", argv, value = TRUE)
script_dir <- if (length(file_flags)) {
  dirname(normalizePath(sub("^--file=", "", file_flags[[1]]), winslash = "/"))
} else {
  getwd()
}
args <- commandArgs(trailingOnly = TRUE)

plot_path <- NULL
plotly_path <- NULL
csv_arg <- NULL
for (a in args) {
  if (startsWith(a, "--plot=")) {
    plot_path <- sub("^--plot=", "", a)
  } else if (startsWith(a, "--plotly=")) {
    plotly_path <- sub("^--plotly=", "", a)
  } else if (a == "--no-plot") {
    plot_path <- NA_character_
  } else if (!startsWith(a, "--")) {
    csv_arg <- a
  }
}

default_csv <- file.path(script_dir, "fixtures", "sample_box_mtm_v1.csv")
path <- if (!is.null(csv_arg)) csv_arg else default_csv

if (!file.exists(path)) {
  stop("CSV not found: ", path)
}

raw <- read.csv(path, stringsAsFactors = FALSE)
need <- c("date", "box_mtm")
miss <- setdiff(need, names(raw))
if (length(miss) > 0) {
  stop("CSV must include columns: ", paste(need, collapse = ", "))
}

raw$date <- as.Date(raw$date)
raw <- raw[order(raw$date), ]
mtm <- raw$box_mtm
n <- length(mtm)
if (n < 3) {
  stop("Need at least 3 rows for return stats")
}

initial <- mtm[[1]]
equity <- mtm / initial
r <- diff(log(equity))
mu <- mean(r)
sdv <- sd(r)
sharpe <- if (is.finite(sdv) && sdv > 0) sqrt(252) * mu / sdv else NA_real_

peak <- cummax(equity)
dd <- (equity - peak) / peak
max_dd <- min(dd)
pnl_cum <- mtm[[n]] - initial

cat("=== box_spread_historical_backtest ===\n")
cat("task: T-1774201866539327000\n")
cat("file:", path, "\n")
cat("rows:", n, "\n")
cat("cumulative_pnl_usd:", sprintf("%.6f", pnl_cum), "\n")
cat("total_return_pct:", sprintf("%.4f", 100 * (equity[[n]] - 1)), "\n")
cat("sharpe_annualized_log:", sprintf("%.4f", sharpe), "\n")
cat("max_drawdown_pct:", sprintf("%.4f", 100 * max_dd), "\n")

# --- Optional ggplot2 ---
if (is.null(plot_path)) {
  plot_path <- file.path(script_dir, "out", "box_mtm_backtest.png")
}
if (!is.na(plot_path) && nzchar(plot_path)) {
  dir.create(dirname(plot_path), showWarnings = FALSE, recursive = TRUE)
  if (requireNamespace("ggplot2", quietly = TRUE)) {
    df <- data.frame(date = raw$date, equity = as.numeric(equity))
    p <- ggplot2::ggplot(df, ggplot2::aes(x = date, y = equity)) +
      ggplot2::geom_line(linewidth = 0.6) +
      ggplot2::labs(
        title = "Box spread — normalized equity (MTM / MTM[1])",
        x = NULL,
        y = "Equity"
      ) +
      ggplot2::theme_minimal()
    ggplot2::ggsave(plot_path, p, width = 8, height = 4, dpi = 120)
    cat("ggplot2_png:", plot_path, "\n")
  } else {
    cat("note: install ggplot2 for default PNG under analytics/r/out/\n")
  }
}

# --- Optional plotly ---
if (!is.null(plotly_path) && nzchar(plotly_path)) {
  if (requireNamespace("plotly", quietly = TRUE) && requireNamespace("htmlwidgets", quietly = TRUE)) {
    df <- data.frame(date = raw$date, box_mtm = as.numeric(mtm), equity = as.numeric(equity))
    fig <- plotly::plot_ly(df, x = ~date, y = ~equity, type = "scatter", mode = "lines", name = "Equity")
    fig <- plotly::layout(fig, title = "Box spread normalized equity")
    htmlwidgets::saveWidget(plotly::as_widget(fig), plotly_path, selfcontained = TRUE)
    cat("plotly_html:", plotly_path, "\n")
  } else {
    cat("note: install plotly + htmlwidgets for --plotly=\n")
  }
}

# --- Optional quantstrat (heavy deps: quantstrat, FinancialInstrument, blotter, xts, TTR) ---
run_qs <- Sys.getenv("AETHER_QUANTSTRAT", "") == "1"
if (run_qs) {
  pkgs <- c("quantstrat", "FinancialInstrument", "blotter", "xts")
  miss_p <- pkgs[!vapply(pkgs, requireNamespace, logical(1), quietly = TRUE)]
  if (length(miss_p)) {
    cat("quantstrat_skip: missing packages — install.packages(c('quantstrat','FinancialInstrument','blotter','xts'))\n")
  } else {
    ok <- tryCatch(
      {
        suppressPackageStartupMessages({
          library(quantstrat)
          library(FinancialInstrument)
          library(blotter)
          library(xts)
        })
        sym <- "BOX_MTM"
        currency("USD")
        stock(sym, currency = "USD", multiplier = 1)
        tt <- raw$date
        px <- raw$box_mtm
        m <- xts::xts(
          cbind(Open = px, High = px, Low = px, Close = px, Volume = 0),
          order.by = tt
        )
        colnames(m) <- paste(sym, c("Open", "High", "Low", "Close", "Volume"), sep = ".")
        assign(sym, m, envir = .GlobalEnv)

        init_date <- as.character(min(tt) - 1)
        strategy.st <- "strat.box.mtm"
        portfolio.st <- "port.box.mtm"
        account.st <- "acct.box.mtm"

        suppressWarnings(try(rm.strat(strategy.st), silent = TRUE))

        initPortf(portfolio.st, symbols = sym, initDate = init_date, currency = "USD")
        initAcct(account.st, portfolios = portfolio.st, initDate = init_date, currency = "USD", initEq = as.numeric(initial))
        initOrders(portfolio.st, initDate = init_date)
        strategy(strategy.st, store = TRUE)

        close_col <- paste0(sym, ".Close")
        add.signal(
          strategy.st,
          name = "sigThreshold",
          arguments = list(
            data = quote(mktdata),
            column = close_col,
            threshold = 0,
            relationship = "gt"
          ),
          label = "longEntry"
        )
        add.rule(
          strategy.st,
          name = "ruleSignal",
          arguments = list(
            sigcol = "longEntry",
            sigval = TRUE,
            orderqty = 1,
            ordertype = "market",
            orderside = "long",
            replace = FALSE,
            prefer = "Close"
          ),
          type = "enter",
          path.dep = TRUE
        )

        applyStrategy(strategy.st, portfolios = portfolio.st)
        updatePortf(Portfolio = portfolio.st)
        updateAcct(Account = account.st)
        updateEndEq(Account = account.st)
        cat("quantstrat: applyStrategy + updatePortf/updateAcct/updateEndEq OK\n")
        TRUE
      },
      error = function(e) {
        cat("quantstrat_error:", conditionMessage(e), "\n")
        FALSE
      }
    )
    if (!ok) {
      invisible(NULL)
    }
  }
} else {
  cat("quantstrat: set AETHER_QUANTSTRAT=1 to run quantstrat path (see analytics/r/README.md)\n")
}

invisible(list(
  equity = equity,
  sharpe = sharpe,
  max_drawdown = max_dd,
  cumulative_pnl = pnl_cum
))
