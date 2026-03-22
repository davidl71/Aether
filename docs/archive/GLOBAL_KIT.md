## Global Kit (shared across projects)

This kit provides consistent editor/formatter/linter configs and a one-step installer you can reuse across projects.

### What’s included

- `.editorconfig` — whitespace, newlines, line length
- `.gitignore_global` — base ignores for dev artifacts
- `.markdownlint.json` — consistent Markdown rules
- `cspell.json` — shared dictionary and settings
- `.clang-format` — C/C++ formatting (LLVM-ish, 2 spaces)
- `.clang-tidy` — core checks; safe defaults
- `.pre-commit-config.yaml` — formatters/linters/secrets scan
- `.shellcheckrc` — tuned shellcheck warnings

Templates live in `global_kit/`.

### Install

Option A: quick install to home

```bash
./scripts/install_global_kit.sh
```

Option B: copy into your dotfiles repo

```bash
cp -R global_kit/* /path/to/your/dotfiles/
```

### Recommended global settings (user-level)

- Editor: format on save, trim trailing whitespace, insert final newline
- Telemetry: disabled
- Spell checker: personal dictionaries in `cspell.json`
- Markdown/YAML validation: enabled

### Using with pre-commit

```bash
brew install pre-commit
pre-commit install
```

### Notes

- Keep project-specific overrides in each repo (e.g., stricter markdownlint per docs-heavy repos).
- Language servers/toolchains should remain project/workspace-specific.
