# Git LFS Installation via Ansible

**Status:** ✅ Integrated

**Location:** `ansible/roles/devtools/tasks/main.yml`

**Purpose:** Automatically install and configure Git LFS on both macOS and Ubuntu/Debian systems.

---

## Installation

Git LFS is automatically installed when running the devtools Ansible role:

### macOS


- Installed via Homebrew: `git-lfs`
- Automatically initialized after installation


### Ubuntu/Debian

- Installed via apt: `git-lfs`
- Automatically initialized after installation

---

## Usage

### Run Devtools Setup

```bash

# Via setup script

./setup_global_tools.sh

# Or directly via Ansible

ansible-playbook -i localhost, --connection=local ansible/playbooks/setup_devtools.yml
```

### Verify Installation

After running the playbook, Git LFS should be installed and initialized:

```bash
git lfs version

# Should show: git-lfs/3.7.1 (or similar)

git lfs ls-files

# Should show any LFS-tracked files
```

---

## What Gets Installed

1. **Git LFS Package**
   - macOS: `git-lfs` via Homebrew
   - Ubuntu/Debian: `git-lfs` via apt

2. **Git LFS Initialization**
   - Automatically runs `git lfs install`
   - Sets up Git hooks for LFS
   - Configures Git to use LFS for tracked files

3. **Verification**
   - Checks that Git LFS is installed

   - Displays version information

---

## Integration

Git LFS is part of the `devtools` role, which is used by:


- `ansible/playbooks/setup_devtools.yml` - Main devtools setup
- `setup_global_tools.sh` - Global tools setup script


---

## Files Tracked with LFS

Currently tracked:

- `.todo2/state.todo2.json` (62.54 MB)

To add more files to LFS:

```bash
git lfs track "path/to/large/file"
git add .gitattributes
git commit -m "Track file with Git LFS"
```

---

## See Also

- `docs/ANSIBLE_SETUP.md` - Complete Ansible setup guide
- `.gitattributes` - Git LFS tracking configuration
- `ansible/roles/devtools/tasks/main.yml` - Devtools role tasks
