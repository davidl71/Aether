# Draft: Execute Task (Script/CMake refactor verification + optional SCRIPTS_AUDIT follow-up)

Use this as a Todo2/exarp task or as an execution plan (e.g. with executing-plans skill or `scripts/shortcuts/draft_plan_from_clipboard.sh`).

---

## Option A: Todo2 / exarp task (create_todos payload)

**Title:** Execute script and CMake refactor verification

**long_description:**

```markdown
🎯 **Objective:** Verify the recent shell-script deduplication and CMake-preset refactor, then optionally execute remaining SCRIPTS_AUDIT items.

📋 **Acceptance criteria:**
- [ ] Run `./scripts/build_fast.sh` on at least one platform (macOS or Linux); build succeeds or fails with a clear preset error.
- [ ] Run `./scripts/build_distributed.sh` on at least one platform; build uses preset and `cmake --build` (no `make -C`).
- [ ] Run `./scripts/test_repo_install.sh` (dry-run) and `sudo ./scripts/test_repo_install.sh --install` in a safe env if applicable.
- [ ] Confirm `release_x86.sh` uses `cmake --preset macos-x86_64-release` (already done; spot-check only).
- [ ] Document outcome in a result comment (pass/fail, platform, any fixes made).

🚫 **Scope:**
- **Included:** Verification of build_fast.sh, build_distributed.sh, test_repo_install.sh, release_x86.sh; optional execution of SCRIPTS_AUDIT §5/§6.3 items.
- **Excluded:** Full implementation of all SCRIPTS_AUDIT “Unite” items (e.g. merge update_stale_docs_*); exarp-go migration (see EXARP_GO_MIGRATION_LEFTOVERS.md).

🔧 **Technical:**
- Presets used: `*-release-sccache`, `*-release-ccache`, `*-release-distcc` (see CMakePresets.json).
- Reference: `scripts/SCRIPTS_AUDIT.md` §5 (action order), §6 (shell vs CMake).

📁 **Files:** scripts/build_fast.sh, scripts/build_distributed.sh, scripts/test_repo_install.sh, scripts/release_x86.sh, CMakePresets.json, scripts/SCRIPTS_AUDIT.md.

🧪 **Verification:** Build and test commands above; ctest after build if desired (`ctest --preset <preset>`).
```

**Suggested tags:** `#verification` `#scripts` `#cmake` `#automation`  
**Priority:** medium  
**Dependencies:** None  

---

## Option B: Execution plan (for executing-plans skill or clipboard → Notes)

**Plan title:** Execute script/CMake refactor verification

**Steps:**

1. **Verify build_fast.sh**  
   From repo root run `./scripts/build_fast.sh`. Expect preset configure + build (sccache or ccache). If missing sccache/ccache, script may install ccache. On success, binary at `build/<preset>/bin/ib_box_spread`. Mark done and note platform.

2. **Verify build_distributed.sh**  
   Run `./scripts/build_distributed.sh`. Expect `cmake --preset` and `cmake --build` (no `make -C`). If distcc/ccache available, preset should be `*-release-distcc`. Mark done and note platform.

3. **Verify test_repo_install.sh**  
   Run `./scripts/test_repo_install.sh` (dry-run). Then, only in a safe environment, optionally run `sudo ./scripts/test_repo_install.sh --install` and confirm it runs `install_deb_repo.sh`. Mark done.

4. **Spot-check release_x86.sh**  
   Open `scripts/release_x86.sh` and confirm first build block is `cmake --preset macos-x86_64-release` and `cmake --build --preset macos-x86_64-release`. No need to run full release. Mark done.

5. **Report**  
   Summarize: platforms tested, any failures or fixes. If all pass, add result comment to task or note “Execute task complete” in SCRIPTS_AUDIT or a follow-up task.

**Optional follow-up (separate batch):**  
- setup_worktree.sh: switch verify step to `cmake --preset <host-preset>`.  
- test_nats_e2e.sh: add preset with ENABLE_NATS=ON if desired.

---

## Option C: One-line summary (for clipboard / quick note)

**Execute task:** Verify script/CMake refactor: run `./scripts/build_fast.sh`, `./scripts/build_distributed.sh`, `./scripts/test_repo_install.sh`; confirm `release_x86.sh` uses presets; document result. Ref: scripts/SCRIPTS_AUDIT.md §5–6.
