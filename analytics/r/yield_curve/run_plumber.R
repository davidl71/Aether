# Launch plumber API (default http://127.0.0.1:8765).
# deps: install.packages(c("plumber", "jsonlite"))

args <- commandArgs(trailingOnly = FALSE)
file_arg <- sub("^--file=", "", args[grep("^--file=", args)])
root <- if (length(file_arg)) dirname(normalizePath(file_arg)) else getwd()
setwd(root)

port <- 8765L
ta <- commandArgs(trailingOnly = TRUE)
if (length(ta) >= 1L) port <- as.integer(ta[[1]])

plumber::plumb("plumber.R")$run(host = "127.0.0.1", port = port)
