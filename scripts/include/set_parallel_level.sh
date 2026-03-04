# Set CMAKE_BUILD_PARALLEL_LEVEL to CPU count when unset.
# Source this before cmake --build so Ninja uses all cores by default.
# Callers can still override with export CMAKE_BUILD_PARALLEL_LEVEL=N.
if [[ -z "${CMAKE_BUILD_PARALLEL_LEVEL:-}" ]]; then
  if [[ "$(uname -s)" == "Darwin" ]]; then
    export CMAKE_BUILD_PARALLEL_LEVEL=$(sysctl -n hw.ncpu 2>/dev/null || echo 4)
  else
    export CMAKE_BUILD_PARALLEL_LEVEL=$(nproc 2>/dev/null || echo 4)
  fi
fi
