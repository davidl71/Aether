## Ansible Dev Environment Setup (macOS)

This repository includes playbooks for a reproducible developer setup split into:

- Global roles (reusable across projects): `common`, `editor`, `langs`
- Project role (this repo): `ib_box_spread`

### Prerequisites

- macOS (Darwin)
- Homebrew installed: <https://brew.sh>
- Ansible:

```bash
brew install ansible
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
