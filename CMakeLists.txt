cmake_minimum_required(VERSION 3.21)
project(SyntheticFinancingPlatformSuperbuild LANGUAGES NONE)

# Convenience targets: lint and exarp-go (run scripts from repo root)
# (fetch_third_party removed: native build removed)
set(_root "${CMAKE_SOURCE_DIR}")
add_custom_target(lint
  COMMAND "${_root}/scripts/run_linters.sh"
  WORKING_DIRECTORY "${_root}"
  COMMENT "Run all linters (run_linters.sh)"
)
add_custom_target(exarp-lint
  COMMAND "${_root}/scripts/run_exarp_go.sh" -tool lint -quiet
  WORKING_DIRECTORY "${_root}"
  COMMENT "Run exarp-go lint"
)
add_custom_target(exarp-list
  COMMAND "${_root}/scripts/run_exarp_go.sh" -list -quiet
  WORKING_DIRECTORY "${_root}"
  COMMENT "List exarp-go tools"
)

# DEPRECATED: C++ native build removed - use Rust backend_service + tui_service
# add_subdirectory(native)
