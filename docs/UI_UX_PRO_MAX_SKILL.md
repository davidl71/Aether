# UI/UX Pro Max skill (Cursor)

[UI UX Pro Max](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill) is an AI skill that provides design intelligence for web and mobile UI/UX (design systems, styles, palettes, typography, stack-specific guidelines). It is wired into this repo for **Cursor** and can be installed via Ansible for the development environment.

## Installation

### Manual (from repo root)

```bash
# Option A: global CLI then init
npm install -g uipro-cli
cd /path/to/ib_box_spread_full_universal
uipro init --ai cursor

# Option B: npx (no global install)
npx --yes uipro-cli init --ai cursor
```

After install, restart Cursor so the skill is loaded. The skill lives under `.cursor/skills/ui-ux-pro-max/`.

### Ansible (development environment)

The **devtools** role installs `uipro-cli` and runs `uipro init --ai cursor` when the playbook is run with the development inventory (so `project_path` is set):

```bash
ansible-playbook -i ansible/inventories/development ansible/playbooks/development.yml
```

Or use the global devtools playbook (installs CLI only; run `uipro init --ai cursor` manually from this repo if needed):

```bash
ansible-playbook ansible/playbooks/setup_devtools.yml
```

See [ANSIBLE_SETUP.md](ANSIBLE_SETUP.md) for Ansible usage.

## Usage

- **Skill mode:** In Cursor, ask for UI/UX work in natural language (e.g. “Build a landing page for my SaaS product”). The skill auto-activates for UI/UX requests.
- **Design system:** From repo root, you can generate a design system with the bundled Python script:
  ```bash
  python3 .cursor/skills/ui-ux-pro-max/scripts/search.py "fintech dashboard" --design-system -p "IB Box Spread"
  ```
- **Stacks:** The skill supports React, Next.js, Vue, Nuxt, Svelte, Astro, SwiftUI, Jetpack Compose, Flutter, React Native, HTML+Tailwind, shadcn/ui. It does **not** include Textual or terminal UI stacks (see below).

## Textual / TUI and ui-ux-pro-max-skill

The [ui-ux-pro-max-skill](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill) repository was checked for **Textual** or **TUI** examples or stack support (e.g. for the Python Textual TUI in `python/tui/`).

**Finding:** The repo has **no Textual or terminal UI support**. Its stacks are web and mobile only:

- **Stacks in repo:** React, Next.js, Vue, Nuxt, Nuxt UI, Svelte, Astro, Flutter, React Native, Jetpack Compose, SwiftUI, HTML+Tailwind, shadcn/ui.
- **Data/templates:** No `textual` or TUI stack CSV under `src/ui-ux-pro-max/data/stacks/`; no terminal/TUI guidelines in the README or data files.

For this project’s **Python Textual TUI** (`python/tui/`), continue to use project docs (e.g. [python/tui/README.md](../python/tui/README.md)), Textual docs, and existing Cursor rules. The UI/UX Pro Max skill is useful for the **web app** (React PWA in `web/`) and any future web/mobile UI work, not for the terminal UI.

## References

- [UI UX Pro Max README](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill)
- [Cursor skills](../.cursor/skills/) — project skills including UI/UX Pro Max
- [AI_EDITOR_SETUP.md](AI_EDITOR_SETUP.md) — canonical AI/editor context
