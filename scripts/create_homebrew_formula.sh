#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/include/logging.sh"

usage() {
  cat <<'EOF'
Usage: scripts/create_homebrew_formula.sh [OPTIONS]

Generate or update a tap-level Homebrew formula for ib-box-spread.

Options:
  --tap <user/repo>        Target tap (default: davidl71/homebrew-tap)
  --formula <name>         Formula name (default: ib-box-spread)
  --class-name <Class>     Ruby class name (default: IbBoxSpread)
  --version-tag <tag>      Git tag to package (default: v1.2.0)
  --url <tarball_url>      Override source URL
  --sha256 <hash>          Override release SHA256
  --dry-run                Print the formula without writing files
  -h, --help               Show this message

Examples:
  scripts/create_homebrew_formula.sh
  scripts/create_homebrew_formula.sh --tap myuser/homebrew-tools --dry-run
EOF
}

TAP="davidl71/homebrew-tap"
FORMULA_NAME="ib-box-spread"
CLASS_NAME="IbBoxSpread"
VERSION_TAG="v1.2.0"
TARBALL_URL="https://github.com/davidl71/ib_box_spread_full_universal/archive/refs/tags/${VERSION_TAG}.tar.gz"
TARBALL_SHA256="0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
DRY_RUN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tap)
      TAP="$2"
      shift 2
      ;;
    --formula)
      FORMULA_NAME="$2"
      shift 2
      ;;
    --class-name)
      CLASS_NAME="$2"
      shift 2
      ;;
    --version-tag)
      VERSION_TAG="$2"
      TARBALL_URL="https://github.com/davidl71/ib_box_spread_full_universal/archive/refs/tags/${VERSION_TAG}.tar.gz"
      shift 2
      ;;
    --url)
      TARBALL_URL="$2"
      shift 2
      ;;
    --sha256)
      TARBALL_SHA256="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      log_error "Unknown option: $1"
      usage
      exit 1
      ;;
  esac
done

if ! command -v brew >/dev/null 2>&1; then
  log_error "Homebrew is required but was not found on PATH."
  exit 1
fi

log_note "Preparing Homebrew formula '${FORMULA_NAME}' for tap '${TAP}'."

TAP_PATH="$(brew --repository)/Library/Taps/${TAP/\//\/}"
FORMULA_DIR="${TAP_PATH}/Formula"
FORMULA_PATH="${FORMULA_DIR}/${FORMULA_NAME}.rb"

formula_contents() {
  cat <<EOF
class ${CLASS_NAME} < Formula
  desc "Native + TUI toolkit for IBKR box spread research"
  homepage "https://github.com/davidl71/ib_box_spread_full_universal"
  url "${TARBALL_URL}"
  sha256 "${TARBALL_SHA256}"
  license "MIT"

  depends_on "cmake" => :build
  depends_on "ninja" => :build
  depends_on "pkg-config" => :build
  depends_on "go" => :build
  depends_on "rust" => :build
  depends_on "python@3.11" => :build

  def install
    system "cmake", "-S", "native", "-B", "build", *std_cmake_args
    system "cmake", "--build", "build"
    bin.install "build/ib_box_spread"

    cd "tui" do
      system "go", "build", "-o", bin/"ib-box-tui", "./cmd/tui"
    end
  end

  test do
    system "#{bin}/ib_box_spread", "--help"
    system "#{bin}/ib-box-tui", "--help"
  end
end
EOF
}

if [[ "${DRY_RUN}" -eq 1 ]]; then
  log_info "Dry run enabled; printing formula:"
  echo
  formula_contents
  exit 0
fi

log_info "Ensuring tap '${TAP}' is available locally."
brew tap --force "${TAP}" >/dev/null

mkdir -p "${FORMULA_DIR}"
log_info "Writing formula to ${FORMULA_PATH}."
formula_contents > "${FORMULA_PATH}"

log_info "Formula updated. Next steps:"
cat <<EOF
- brew install --build-from-source ${TAP}/${FORMULA_NAME}
- brew audit --new --formula ${TAP}/${FORMULA_NAME}
EOF

