#!/usr/bin/env bash
# Set CMAKE_BUILD_PARALLEL_LEVEL to CPU count when unset.
# Source this before cmake --build so Ninja uses all cores by default.
# Callers can still override with export CMAKE_BUILD_PARALLEL_LEVEL=N.
if [[ -z "${CMAKE_BUILD_PARALLEL_LEVEL:-}" ]]; then
  _nproc=
  if [[ "$(uname -s)" == "Darwin" ]]; then
    _nproc=$(sysctl -n hw.ncpu 2>/dev/null || echo 4)
  else
    _nproc=$(nproc 2>/dev/null || echo 4)
  fi
  export CMAKE_BUILD_PARALLEL_LEVEL="${_nproc}"
fi
