# Development Tools

## Code Navigation

- **ctags** - Generate tags for code navigation

  ```bash
  ctags -R .
  ```

- **tagref** - Check cross-references in code comments

  ```bash
  brew install tagref
  # Run check
  tagref --upstream-refs .
  ```

  Format: `# [tag:tagname]` and `# [ref:tagname]`

- **xrefcheck** - Check cross-references in documentation (markdown)

  ```bash
  cargo install xrefcheck
  # Run check
  xrefcheck -i docs/
  ```

## Installation

```bash
# macOS
brew install universal-ctags tagref

# Rust tools
cargo install xrefcheck
```
