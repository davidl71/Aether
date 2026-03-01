## Ansible Dev Environment Setup

This repository includes playbooks for a reproducible developer setup split into:

- Global roles (reusable across projects): `common`, `editor`, `langs`
- Project role (this repo): `ib_box_spread`

### Prerequisites

- macOS (Darwin) or Linux (Debian/Ubuntu)
- Ansible **9.x or 10.x** (ansible-core 2.15+ or 2.16+)

### Install Ansible

**macOS (Homebrew):**

```bash
brew install ansible
ansible-galaxy collection install community.general
```

For a minimal install (ansible-core only, install collections as needed):

```bash
brew install ansible-core
ansible-galaxy collection install community.general
```

**Linux (Debian/Ubuntu):**

```bash
# Option A: system package (may be older)
sudo apt update && sudo apt install ansible
ansible-galaxy collection install community.general

# Option B: pip/uv (newer versions)
uv pip install ansible
ansible-galaxy collection install community.general
```

**Optional (any OS):** Pin versions with pip:

```bash
uv pip install "ansible>=9,<11" "ansible-core>=2.15"
ansible-galaxy collection install community.general
```

### Structure

- `playbooks/global.yml` — global setup (brew packages/casks, editor, global kit)
- `playbooks/project.yml` — project setup (IB API paths, CMake configure)
- `playbooks/site.yml` — runs both in sequence
- `roles/common`, `roles/editor`, `roles/langs` — global roles
- `roles/ib_box_spread` — project-specific role

### Usage

Run global setup (safe across all projects):

```bash
ansible-playbook playbooks/global.yml -K -t "global"
```

Run project setup (requires IB API path):

```bash
export IBJTS_DIR=~/IBJts/source/cppclient
ansible-playbook playbooks/project.yml -K -e ibjts_dir="$IBJTS_DIR" -t "project"
```

Run everything:

```bash
export IBJTS_DIR=~/IBJts/source/cppclient
ansible-playbook playbooks/site.yml -K
```

### Tags

- Global: `global`, `common`, `editor`, `langs`
- Project: `project`, `ib`, `cmake`, `lint`

Examples:

```bash
ansible-playbook playbooks/global.yml -K -t "editor"
ansible-playbook playbooks/project.yml -K -t "cmake"
```

### Notes

- Global kit is installed via `scripts/install_global_kit.sh --mode link`
- No secrets are handled by playbooks; IB credentials must remain out-of-band
- Tasks are idempotent and safe to re-run

### Upgrading Ansible

After upgrading Ansible or collections, run a playbook with `deprecation_warnings = True` in `ansible.cfg` (default in this repo) to see any deprecations. Fix playbook or role references as needed, then you can set `deprecation_warnings = False` again if desired. Check version with:

```bash
ansible --version
ansible-galaxy collection list
```
