# Project skills

Reusable workflows and checklists for this repo. Use by opening or @-mentioning a skill file when asking the AI to perform the task.

**Canonical context:** AGENTS.md and CLAUDE.md in repo root. Skills reference these and project scripts/commands.

| Skill | File | When to use |
|-------|------|--------------|
| **exarp-go** | [exarp-go/SKILL.md](exarp-go/SKILL.md) | Task management, project health, session prime/handoff, scorecards (`PROJECT_ROOT` = target repo) |
| **Aether Todo2 + exarp** | [aether-todo2-exarp/SKILL.md](aether-todo2-exarp/SKILL.md) | **This repo’s** `.todo2/`: sync, bulk Review→Done, `task_workflow` JSON, Cargo.lock, TUI workspace helpers |
| **build-shortcuts** | [build-shortcuts/SKILL.md](build-shortcuts/SKILL.md) | Make, cargo, CMake presets, build/test/lint shortcuts |
| **oh-my-opencode** | [.opencode/oh-my-opencode/README.md](/.opencode/oh-my-opencode/README.md) | Oh My OpenCode config - themes, shortcuts, hooks, enhanced workflow |
| **UI/UX Pro Max** | [ui-ux-pro-max/SKILL.md](ui-ux-pro-max/SKILL.md) | Web/PWA design systems, landing pages, dashboards (design-system generator, styles, palettes) |
| **When to use subagents** | [when-to-use-subagents.md](when-to-use-subagents.md) | Code review, refactor, tests, trading audit, tasks/reports — which subagent or exarp-go to use |
| Pull with uncommitted changes | [git-pull-with-wip.md](git-pull-with-wip.md) | Before pulling; you have local WIP |
| Add native C++ module | [add-native-module.md](add-native-module.md) | Adding a new .cpp/.h and test |
| Before commit | [before-commit.md](before-commit.md) | Checklist before committing |
| Trading safety | [trading-safety.md](trading-safety.md) | Any change touching orders, config, or live trading |
| Build from clean | [build-from-clean.md](build-from-clean.md) | Clean configure + build (deps, presets) |

**Commands:** Many of these map to [.cursor/commands.json](../commands.json) (e.g. `git:pull-safe`, `build:debug`, `build:ai-friendly`, `test:run`, `lint:run`).
