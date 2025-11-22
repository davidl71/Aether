# Repository Rename Plan: T-210

**Date**: 2025-01-27
**Task**: T-210 - Phase 3 Repository Rename
**Status**: ⚠️ **HIGH RISK - Coordination Required**
**Current Repository**: `ib_box_spread_full_universal`
**Target Repository**: `synthetic-financing-platform`

---

## ⚠️ Risk Assessment

### Current NATS Implementation Status
- ⏳ **T-173**: In Progress - Deploy NATS server
- ⏳ **T-174**: In Progress - Create Rust NATS adapter crate
- ⏳ **T-175**: In Progress - Integrate NATS adapter into Rust backend service
- 📋 **T-193**: Todo - Add NATS health check
- ⏳ **T-194**: In Progress - Create topic registry
- 📋 **T-195**: Todo - Integrate NATS adapter into Rust backend service

### Risk Level: **HIGH**

**Per PHASE_CONFLICT_ANALYSIS.md:**
- Repository rename affects ALL external references
- NATS documentation may reference repository name
- **Recommendation**: DO NOT START until NATS work complete OR coordinate explicitly with NATS agent

---

## 📋 Pre-Rename Checklist

### ✅ Phase 1 Complete (Documentation Organization)
- [x] All documentation updated to "Synthetic Financing Platform"
- [x] Box-spread docs moved to `docs/strategies/box-spread/`
- [x] Platform docs moved to `docs/platform/`
- [x] Cross-references updated

### ✅ Phase 2 Complete (Code Reorganization)
- [x] Box-spread code moved to `native/src/strategies/box_spread/`
- [x] Include statements updated
- [x] CMakeLists.txt updated with new paths

### ✅ Phase 3 Complete (Configuration Updates)
- [x] CMakeLists.txt project names updated
- [x] python/pyproject.toml updated
- [x] homebrew-tap/README.md updated
- [x] Package metadata reflects new identity

### ⚠️ Coordination Required
- [ ] **Coordinate with NATS agent** before proceeding
- [ ] Verify NATS documentation doesn't reference old repository name
- [ ] Ensure NATS implementation work won't be disrupted

---

## 📝 Files Requiring Updates

### Documentation Files (Safe to Update)

**High Priority:**
- `docs/homebrew-tap/README.md` - Repository URLs
- `docs/HOMEBREW_TAP.md` - Repository references
- `docs/DOCUMENTATION_INDEX.md` - Repository URLs
- `docs/PROJECT_RENAME_AND_SPLIT_ANALYSIS.md` - Repository name references
- `docs/DOCUMENTATION_CONSISTENCY_REVIEW.md` - Repository references
- `docs/NEXT_STEPS_RENAME_AND_SPLIT.md` - Repository references

**Medium Priority:**
- All documentation files with `github.com/davidl71/ib_box_spread_full_universal` URLs
- Homebrew formula files (if repository URL is referenced)

**Low Priority:**
- Learning documents (can note future rename)
- Historical references (may keep for context)

### Configuration Files (Requires Coordination)

**⚠️ Wait for NATS Coordination:**
- `.git/config` - Remote URL (manual update after GitHub rename)
- CI/CD configurations (if any reference repository name)
- Scripts that reference repository URL

### Homebrew Tap Files

**Files to Update:**
- `homebrew-tap/Formula/ib-box-spread.rb` - Repository URL
- `homebrew-tap/Formula/ib-box-spread-tui.rb` - Repository URL
- `homebrew-tap/README.md` - Repository references (already updated in T-209)

---

## 🚀 Manual Steps Required

### Step 1: Update Documentation References (Safe - Can Do Now)

Update all documentation files to use new repository name in URLs:
- `github.com/davidl71/ib_box_spread_full_universal` → `github.com/davidl71/synthetic-financing-platform`
- Note: Links won't work until actual GitHub rename is complete

### Step 2: GitHub Repository Rename (Manual - Requires Coordination)

1. Go to GitHub repository settings: https://github.com/davidl71/ib_box_spread_full_universal/settings
2. Navigate to "Repository name" section
3. Rename to: `synthetic-financing-platform`
4. Confirm rename

**⚠️ Important:**
- GitHub will automatically redirect old URLs
- All clones will need to update remote URL
- Notify any collaborators before rename

### Step 3: Update Local Git Remote (After GitHub Rename)

```bash
git remote set-url origin git@github.com:davidl71/synthetic-financing-platform.git
git remote -v  # Verify update
```

### Step 4: Update Homebrew Tap (After GitHub Rename)

1. Update formula repository URLs
2. Update tap repository name (if applicable)
3. Test Homebrew installation

### Step 5: Update CI/CD (If Applicable)

1. Update repository references in CI/CD configurations
2. Test CI/CD pipelines
3. Verify all builds succeed

---

## 📊 Impact Analysis

### Files That Reference Repository Name

**Documentation (20+ files):**
- Repository URLs in documentation
- Homebrew tap references
- GitHub link references

**Configuration:**
- Git remote URL (manual update after GitHub rename)
- CI/CD configurations (if any)

**External:**
- Homebrew tap repository name
- Any external documentation or bookmarks

---

## ✅ Safe Actions (Can Do Now)

1. **Update Documentation URLs** (Safe - Forward-looking references)
   - Update URLs to new repository name
   - Note that links won't work until GitHub rename
   - Add comment: "Will be available after repository rename"

2. **Create Repository Rename Checklist** (Done - this document)

3. **Document Manual Steps** (Done - above)

---

## ⚠️ Actions Requiring Coordination

1. **GitHub Repository Rename** (Manual - Coordinate with NATS agent)
2. **Git Remote URL Update** (After GitHub rename)
3. **CI/CD Updates** (If applicable - check for conflicts)

---

## 🎯 Recommended Approach

### Phase A: Safe Updates (Now)
1. Update documentation references to new repository name
2. Add notes about pending repository rename
3. Document manual steps

### Phase B: GitHub Rename (After Coordination)
1. Coordinate with NATS agent
2. Verify NATS work won't be disrupted
3. Perform GitHub repository rename
4. Update local git remote

### Phase C: Verification (After Rename)
1. Test all links and references
2. Verify CI/CD still works
3. Test Homebrew tap installation
4. Update any remaining references

---

## 📚 References

- **Conflict Analysis**: `docs/PHASE_CONFLICT_ANALYSIS.md`
- **Rename Analysis**: `docs/PROJECT_RENAME_AND_SPLIT_ANALYSIS.md`
- **Next Steps**: `docs/NEXT_STEPS_RENAME_AND_SPLIT.md`

---

**Last Updated**: 2025-01-27
**Status**: ⚠️ **Pending NATS Coordination**
