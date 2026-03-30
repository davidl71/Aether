# fit_term_structure.R — sparse zero curve → dense grid (base R + optional CRAN add-ons).
# Task T-1774201865476785000: deferred scaffold; no calls from Rust yet.

.is_pos_sorted <- function(x) {
  ok <- is.numeric(x) && length(x) >= 2L && all(x > 0) && !anyNA(x)
  if (!ok) return(FALSE)
  identical(x, sort(x))
}

# Monotone natural spline on (T, z). For duplicate maturities, averages zero_rates.
.aggregate_unique <- function(Tm, z) {
  ord <- order(Tm)
  Tm <- Tm[ord]
  z <- z[ord]
  u <- unique(Tm)
  if (length(u) == length(Tm)) return(list(T = Tm, z = z))
  z_agg <- vapply(u, function(t) mean(z[Tm == t]), numeric(1))
  list(T = u, z = z_agg)
}

# Simple discrete instantaneous forward approx from annualized continuously compounded zeros
# (rough): f ~ (z2*T2 - z1*T1) / (T2 - T1) on each segment; forward at grid via diff.
.zero_to_forward_piecewise <- function(Tgrid, zgrid) {
  n <- length(Tgrid)
  if (n < 2L) return(rep(NA_real_, n))
  fwd <- rep(NA_real_, n)
  for (i in seq_len(n - 1L)) {
    dt <- Tgrid[i + 1L] - Tgrid[i]
    if (dt <= 0) next
    fwd[i] <- (zgrid[i + 1L] * Tgrid[i + 1L] - zgrid[i] * Tgrid[i]) / dt
  }
  fwd[n] <- fwd[n - 1L]
  fwd
}

#' Estimate a smooth zero curve on a grid from sparse pillars.
#'
#' @param maturities_years Positive numeric, strictly increasing after aggregation.
#' @param zero_rates Annualized decimals (e.g. 0.052), same length as maturities.
#' @param output_grid Optional grid in years; default 50 points between min and max T.
#' @return list(method, grid_years, zero_rates, forward_rates, optional note)
estimate_term_structure <- function(
    maturities_years,
    zero_rates,
    output_grid = NULL) {
  if (length(maturities_years) != length(zero_rates)) {
    stop("maturities_years and zero_rates must have the same length")
  }
  ag <- .aggregate_unique(as.numeric(maturities_years), as.numeric(zero_rates))
  Tm <- ag$T
  z <- ag$z
  if (length(Tm) < 2L) {
    stop("need at least two distinct maturities after aggregating duplicates")
  }
  if (!.is_pos_sorted(Tm)) {
    stop("maturities_years must be positive, finite, and sortable to increasing order")
  }
  if (anyNA(z) || !is.numeric(z)) stop("zero_rates must be numeric and finite")

  if (is.null(output_grid)) {
    output_grid <- seq(min(Tm), max(Tm), length.out = 50L)
  } else {
    output_grid <- sort(unique(as.numeric(output_grid)))
    output_grid <- output_grid[output_grid >= min(Tm) & output_grid <= max(Tm)]
    if (length(output_grid) < 2L) stop("output_grid must span at least two points in-range")
  }

  # Natural spline extrapolates; clip to pillar range for stability in sidecar use.
  sp <- stats::spline(Tm, z, xout = output_grid, method = "natural")
  zg <- pmax(as.numeric(sp$y), 0)
  method <- "natural_spline"

  list(
    method = method,
    grid_years = as.numeric(sp$x),
    zero_rates = zg,
    forward_rates = .zero_to_forward_piecewise(as.numeric(sp$x), zg),
    note = paste(
      "Optional CRAN packages termstrc / YieldCurve: install for NS/Svensson;",
      "see README; enable explicit fitting in a follow-up task."
    )
  )
}
