# Reusable Ansible patterns from exarp-go (sibling repo)

Patterns from exarp-go’s `ansible/` that we can reuse or have adopted in this repo. exarp-go is Go-focused; this repo is C++/multi-language, so some roles differ but the **structure and patterns** transfer.

---

## 1. Role layout: OS-specific vars

**exarp-go:** Each role has `vars/{{ ansible_os_family }}.yml` (e.g. `Darwin.yml`, `Debian.yml`, `RedHat.yml`) and loads them at the top of `tasks/main.yml`:

```yaml
# tasks/main.yml
- name: Load OS-specific variables
  ansible.builtin.include_vars: "{{ ansible_os_family }}.yml"
  tags: [common]
```

**Vars files** define OS-specific package names and paths (e.g. `_base_packages`, `_node_pkg`, `_ca_bundle_path`). That keeps one task list and switches behavior by OS.

**Reuse here:** For any role that installs different packages on macOS vs Linux, add `roles/<role>/vars/Darwin.yml` and `roles/<role>/vars/Debian.yml`, and at the start of `tasks/main.yml` add the `include_vars` task. Then reference `_packages` (or similar) in a single `package`/`homebrew` task. We refactored `devtools` this way and also ported the **CA/SSL fix** from exarp-go’s common role: Debian gets `ca-certificates` in the package list, `_ca_bundle_path` in vars, and tasks to run `update-ca-certificates` and verify the bundle; macOS uses system keychain (no update task).

---

## 2. Inventory + group_vars for project and optional features

**exarp-go:** `inventories/development/group_vars/all.yml` defines:

- **Project:** `project_name`, `project_path`, `project_user` (with `lookup('env', 'HOME')`, `lookup('env', 'USER')`).
- **Optional install flags:** `install_linters`, `install_ollama`, `install_redis`, `install_security_scanners`, `install_gh`, `install_file_watchers`.
- **Tool config:** e.g. `go_version`, `linters` list, `ollama_models`.

Playbooks use `hosts: development` and roles are gated with `when: install_foo | default(false)`.

**Reuse here:** We added `ansible/inventories/development/` with `group_vars/all.yml` defining `project_name`, `project_path`, `project_user`, and optional flags (`install_*`) for future optional roles. Playbooks can target `hosts: development` and use the same `when:` + tags pattern.

---

## 3. Single “development” playbook with optional roles and tags

**exarp-go:** `playbooks/development.yml`:

- `hosts: development`, `gather_facts: true`, `become: true`.
- **Roles in order:** common (always) → golang (always) → linters, ollama, redis, security_scanners with `when: install_* | default(false)`.
- **Tags:** `always`, `optional`, and per-role tags (`common`, `linters`, `security_scanners`, etc.).
- **Summary task:** `ansible.builtin.debug` at the end listing what was installed.

**Reuse here:** We added `playbooks/development.yml` that runs the `devtools` role (and can be extended with more roles). It uses the same tag pattern and a short summary. Optional roles can be added with `when: install_* | default(false)` and matching tags.

---

## 4. Optional role with defaults and group_vars override

**exarp-go:** e.g. `security_scanners`:

- **defaults/main.yml:** `install_security_scanners: false`, `security_scanner_tools: [govulncheck, pip_audit, cargo_audit, npm]`.
- **Playbook:** `when: install_security_scanners | default(false)`.
- **group_vars:** `install_security_scanners: false` (or `true` for devs who want them).

**Reuse here:** For any new optional role (e.g. linters, security scanners, ollama), add `defaults/main.yml` with `install_<name>: false` and a list of sub-tools or options. In the playbook include the role with `when: install_<name> | default(false)`. Override in `inventories/development/group_vars/all.yml` per environment.

---

## 5. Role meta (platforms, dependencies)

**exarp-go:** `roles/common/meta/main.yml`:

```yaml
galaxy_info:
  role_name: common
  description: ...
  min_ansible_version: "2.14"
  platforms:
    - name: Debian
      versions: [bookworm, jammy, noble]
    - name: EL
      versions: [8, 9]
    - name: MacOSX
      versions: [all]
dependencies: []
```

**Reuse here:** Add `meta/main.yml` to roles that should declare supported OSes and Ansible version. Helps CI and future Galaxy sharing.

---

## 6. Async installs for optional tools (linters)

**exarp-go:** The linters role uses `async` and `poll: 0` to start several installs (golangci-lint, Go-based linters, npm linters, etc.), then `async_status` to wait. Failures are tolerated (`failed_when: false`) so one missing linter doesn’t fail the playbook.

**Reuse here:** If we add a dedicated “linters” role with many optional tools, we can use the same pattern: fire independent installs with `async`/`poll: 0`, then wait with `async_status`, and treat failures as non-fatal for optional linters.

---

## 7. Summary task at end of playbook

**exarp-go:** A final `ansible.builtin.debug` task that prints a short summary (Go version, project path, which optional tools were installed).

**Reuse here:** `playbooks/development.yml` ends with a short summary task. Expand it as we add more optional roles so a single run gives a clear “what’s installed” view.

---

## What we did not copy

- **exarp-go–specific roles:** golang, redis, ollama (we have devtools and third_party; we can add optional ollama/linters/security_scanners later if needed).
- **Full common role:** exarp-go’s common role does base packages, **CA certs** (Debian: `ca-certificates` + `update-ca-certificates`; verify bundle at `_ca_bundle_path`), Xcode CLT, git config, gh, SQLite. We already have devtools that cover build/lint and platform tooling; we didn’t duplicate common but **we did port the CA/SSL fix** into devtools (see below).
- **production playbook:** exarp-go has `production.yml`; we have no production Ansible yet. When we do, we can mirror the same inventory + group_vars + tags pattern.

---

## Files added or to add

| Item | Status |
|------|--------|
| `docs/EXARP_GO_ANSIBLE_PATTERNS.md` | Done (this file) |
| `ansible/inventories/development/hosts` | Done – `localhost` for local dev |
| `ansible/inventories/development/group_vars/all.yml` | Done – project vars + optional flags |
| `ansible/playbooks/development.yml` | Done – runs devtools with tags; extend with more roles and `when:` as needed |
| Existing `playbooks/setup_devtools.yml` | Unchanged – still works with `hosts: localhost` and no inventory |
| OS-specific vars in `devtools` (vars/Darwin.yml, vars/Debian.yml) | Optional refactor |
| `meta/main.yml` for existing roles | Optional |
| Optional `security_scanners` or `linters` role | Future – use exarp-go’s role as reference |

---

## References

- exarp-go: `ansible/playbooks/development.yml`, `ansible/inventories/development/group_vars/all.yml`, `ansible/roles/common/`, `ansible/roles/security_scanners/`, `ansible/roles/linters/`
- This repo: `ansible/playbooks/`, `ansible/roles/devtools/`, `ansible/roles/third_party/`, `ansible/run-dev-setup.sh`
