# Markdown Style Guide

## gomarklint Rules

Run: `gomarklint docs/`

### Required Rules

| Rule | Description | Auto-fix |
|------|-------------|----------|
| `first-heading-level` | First heading must be level 2 or higher | Manual: change `# Title` → `## Title` |
| `heading-level-jump` | No jumping from h2 to h4 (must have h3 in between) | Manual: insert `###` heading |
| `duplicate-heading` | No duplicate headings in the same section | Manual: rename or consolidate |

### Common False Positives (code blocks — do NOT "fix" these)

gomarklint parses headings inside fenced code blocks. These are **NOT real errors**:

```bash
#!/bin/bash        # shebangs — not headings
#include <vector>  # C++ includes — not headings
#include "file.h"
#pragma once
[derive(Debug)]
[tokio::test]
```

ASCII separators and file paths in tables/lists are also false positives.

### Creating New Docs

1. **First heading must be level 2+** — never start with `# Title`, use `## Title`
2. **Use unique headings** — if you have multiple "Overview" sections, rename them
3. **No heading level jumps** — `##` must be followed by `###`, not `####`
4. **Keep headings descriptive** — avoid generic names like "Overview", "Status", "Summary"

### Example

```markdown
## Box Spread Strategy          # ✅ Level 2 first heading

### Overview                     # ✅ Level 3 under level 2
### Implementation Details       # ✅ Level 3 under level 2
#### Leg Construction             # ✅ Level 4 under level 3 (no jump)
```

### Fixing Scripts

```bash
# Auto-fix first-heading-level only
gomarklint docs/ 2>/dev/null | grep "First heading should be level 2" | \
  sed 's/:1: First heading.*//' | xargs -I{} sed -i '' '1s/^# /## /' {}

# Note: Do NOT auto-fix duplicate headings — most are false positives from code blocks
```
