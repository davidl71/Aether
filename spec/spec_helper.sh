# shellcheck shell=sh

# Defining variables and functions here will affect all specfiles.
# Change shell options inside a function may cause different behavior,
# so it is better to set them here.
# set -eu

# AI CONTEXT: Set project root for sourcing shared functions
# This is used by all test files to locate scripts/include/ files
# spec_helper.sh is in spec/, so go up one level to get project root
if [ -z "${PROJECT_ROOT:-}" ]; then
  # Calculate from spec directory location (spec_helper.sh is in spec/)
  # Use a more reliable method that works in ShellSpec context
  _SPEC_HELPER_DIR="${SHELLSPEC_SPECDIR:-spec}"
  if [ -d "${_SPEC_HELPER_DIR}" ]; then
    PROJECT_ROOT="$(cd "${_SPEC_HELPER_DIR}/.." && pwd)"
  else
    # Fallback: assume we're in spec/ directory
    PROJECT_ROOT="$(cd "$(dirname "$0")/.." 2>/dev/null && pwd || pwd)"
  fi
  export PROJECT_ROOT
fi

# This callback function will be invoked only once before loading specfiles.
spec_helper_precheck() {
  # Available functions: info, warn, error, abort, setenv, unsetenv
  # Available variables: VERSION, SHELL_TYPE, SHELL_VERSION
  : minimum_version "0.28.1"
}

# This callback function will be invoked after a specfile has been loaded.
spec_helper_loaded() {
  :
}

# This callback function will be invoked after core modules has been loaded.
spec_helper_configure() {
  # Available functions: import, before_each, after_each, before_all, after_all
  : import 'support/custom_matcher'
}
