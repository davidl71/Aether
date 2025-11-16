#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
KIT_DIR="$REPO_ROOT/global_kit"

usage() {
  echo "Usage: $0 [--dest <path>] [--mode copy|link]"
  echo "  --dest: destination directory (default: \$HOME)"
  echo "  --mode: copy or link (default: link)"
}

DEST="$HOME"
MODE="link"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dest) DEST="$2"; shift 2 ;;
    --mode) MODE="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1"; usage; exit 2 ;;
  esac
done

echo "Installing Global Kit from: $KIT_DIR"
echo "Destination: $DEST"
echo "Mode: $MODE"
echo

mkdir -p "$DEST"

install_one() {
  local src="$1"
  local base
  base="$(basename "$src")"
  local dest_path="$DEST/$base"
  if [[ -e "$dest_path" || -L "$dest_path" ]]; then
    echo "Backing up existing $dest_path -> $dest_path.bak"
    mv -f "$dest_path" "$dest_path.bak"
  fi
  case "$MODE" in
    copy) cp -R "$src" "$dest_path" ;;
    link) ln -s "$src" "$dest_path" ;;
    *) echo "Invalid mode: $MODE"; exit 2 ;;
  esac
  echo "Installed $base"
}

for file in ".editorconfig" ".gitignore_global" ".markdownlint.json" "cspell.json" ".clang-format" ".clang-tidy" ".pre-commit-config.yaml" ".shellcheckrc"; do
  install_one "$KIT_DIR/$file"
done

echo
echo "Done."
echo "Tip: set your global gitignore:"
echo "  git config --global core.excludesfile \"$DEST/.gitignore_global\""
