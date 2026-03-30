# plumber.R — HTTP sidecar for term-structure smoothing (T-1774201865476785000).
# Run from this directory: Rscript run_plumber.R

library(plumber)

source("fit_term_structure.R")

#* Health check
#* @get /health
function() {
  list(
    ok = TRUE,
    termstrc_installed = requireNamespace("termstrc", quietly = TRUE),
    yieldcurve_installed = requireNamespace("YieldCurve", quietly = TRUE)
  )
}

#* Fit zero curve from sparse pillars (JSON body)
#* @post /estimate
function(req, res) {
  if (is.null(req$postBody) || !nzchar(req$postBody)) {
    res$status <- 400L
    return(list(error = "expected JSON body"))
  }
  b <- tryCatch(
    jsonlite::fromJSON(req$postBody, simplifyVector = TRUE),
    error = function(e) NULL
  )
  if (is.null(b) || is.null(b$maturities_years) || is.null(b$zero_rates)) {
    res$status <- 400L
    return(list(error = "body must include maturities_years and zero_rates arrays"))
  }
  og <- b$output_grid
  out <- tryCatch(
    estimate_term_structure(b$maturities_years, b$zero_rates, output_grid = og),
    error = function(e) {
      res$status <- 400L
      list(error = conditionMessage(e))
    }
  )
  if (!is.null(out$error)) return(out)
  out$data_source <- "R_sidecar"
  out
}
