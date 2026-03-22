# Git LFS Candidates Analysis

**Date**: 2025-01-27
**Status**: Analysis
**Purpose**: Identify files that should be stored in Git LFS instead of Git

---

## Overview

Git LFS (Large File Storage) is recommended for files that are:

- **Large** (>50MB typically, but can be smaller for binaries)
- **Binary** (not diff-friendly)
- **Frequently changing** (causes repository bloat)
- **Build artifacts** (shouldn't be in Git)
- **Dependencies** (should be downloaded, not stored)

---

## Analysis Results

### Files Currently Tracked by Git

#### Large Files (>1MB)

**Status**: Checking for large tracked files...

**Recommendation**: Any file >50MB should definitely use Git LFS. Files >10MB should be considered.

---

### Binary Files

#### Library Files (.dylib, .so, .a, .lib)

**Common Locations**:

- `native/third_party/` - Third-party libraries
- `build/` - Build artifacts (should be in .gitignore)
- System libraries

**Recommendation**:

- ✅ **Third-party libraries**: Use Git LFS if >1MB
- ❌ **Build artifacts**: Should be in `.gitignore`, not Git LFS

**Example Patterns**:

```
*.dylib
*.so
*.a
*.lib
lib*.a
```

---

### Archive Files (.zip, .tar.gz, .tar, .gz)

**Status**: Checking for archive files...

**Recommendation**:

- ✅ **Vendor archives**: Use Git LFS if tracked
- ❌ **Build artifacts**: Should be in `.gitignore`
- ✅ **Documentation archives**: Use Git LFS if >10MB

**Example Patterns**:

```
*.zip
*.tar.gz
*.tar
*.gz
*.7z
*.rar
```

---

### Database Files (.db, .sqlite, .sqlite3)

**Status**: Checking for database files...

**Recommendation**:

- ✅ **Test databases**: Use Git LFS if >1MB
- ❌ **Production databases**: Should NOT be in Git
- ✅ **Sample databases**: Use Git LFS if tracked

**Example Patterns**:

```
*.db
*.sqlite
*.sqlite3
*.db3
```

---

### Image Files (.jpg, .jpeg, .png, .gif, .bmp, .tiff, .webp)

**Status**: Checking for large image files...

**Recommendation**:

- ✅ **Large images** (>1MB): Use Git LFS
- ✅ **Screenshots/documentation images**: Use Git LFS if >500KB
- ❌ **Small icons/logos**: Can stay in Git (<100KB)

**Example Patterns**:

```
*.jpg
*.jpeg
*.png
*.gif
*.bmp
*.tiff
*.webp
```

---

### Document Files (.pdf, .epub, .mobi)

**Status**: Checking for document files...

**Recommendation**:

- ✅ **PDFs**: Use Git LFS if >1MB
- ✅ **Documentation PDFs**: Use Git LFS if tracked
- ✅ **E-books**: Use Git LFS if tracked

**Example Patterns**:

```
*.pdf
*.epub
*.mobi
```

---

## Recommended Git LFS Configuration

### High-Priority Candidates

**If found, these should use Git LFS:**

1. **Third-party libraries** (`native/third_party/`)
   - `*.dylib` files >1MB
   - `*.a` (static libraries) >1MB
   - `*.so` files >1MB

2. **Large binary files**
   - Any file >50MB
   - Executables >10MB
   - Database files >1MB

3. **Media files**
   - Images >1MB
   - Videos (if any)
   - Audio files (if any)

4. **Archives**
   - Vendor archives >10MB
   - Documentation archives >10MB

---

### Medium-Priority Candidates

**Consider Git LFS for:**

1. **Documentation images**
   - Screenshots >500KB
   - Diagrams >500KB
   - Documentation PDFs >1MB

2. **Test data**
   - Test databases >1MB
   - Sample data files >5MB

---

### Low-Priority (Keep in Git)

**These can stay in Git:**

1. **Small images** (<100KB)
   - Icons
   - Small logos
   - Thumbnails

2. **Configuration files**
   - JSON, YAML, TOML
   - Small binary configs

3. **Source code**
   - All text files
   - Scripts

---

## Git LFS Setup Recommendations

### Step 1: Install Git LFS

```bash

# macOS

brew install git-lfs

# Linux

sudo apt-get install git-lfs  # Debian/Ubuntu
sudo yum install git-lfs       # RHEL/CentOS

# Windows
# Download from https://git-lfs.github.com/
```

### Step 2: Initialize Git LFS

```bash
git lfs install
```

### Step 3: Track File Patterns

**Recommended `.gitattributes` additions:**

```gitattributes

# Third-party libraries

native/third_party/**/*.dylib filter=lfs diff=lfs merge=lfs -text
native/third_party/**/*.so filter=lfs diff=lfs merge=lfs -text
native/third_party/**/*.a filter=lfs diff=lfs merge=lfs -text
native/third_party/**/*.lib filter=lfs diff=lfs merge=lfs -text

# Large binary files
*.dylib filter=lfs diff=lfs merge=lfs -text
*.so filter=lfs diff=lfs merge=lfs -text
*.a filter=lfs diff=lfs merge=lfs -text
*.lib filter=lfs diff=lfs merge=lfs -text

# Archives
*.zip filter=lfs diff=lfs merge=lfs -text
*.tar.gz filter=lfs diff=lfs merge=lfs -text
*.tar filter=lfs diff=lfs merge=lfs -text
*.gz filter=lfs diff=lfs merge=lfs -text

# Databases
*.db filter=lfs diff=lfs merge=lfs -text
*.sqlite filter=lfs diff=lfs merge=lfs -text
*.sqlite3 filter=lfs diff=lfs merge=lfs -text

# Large images (>1MB)
*.jpg filter=lfs diff=lfs merge=lfs -text
*.jpeg filter=lfs diff=lfs merge=lfs -text
*.png filter=lfs diff=lfs merge=lfs -text
*.gif filter=lfs diff=lfs merge=lfs -text
*.bmp filter=lfs diff=lfs merge=lfs -text
*.tiff filter=lfs diff=lfs merge=lfs -text
*.webp filter=lfs diff=lfs merge=lfs -text

# Documents
*.pdf filter=lfs diff=lfs merge=lfs -text
*.epub filter=lfs diff=lfs merge=lfs -text
*.mobi filter=lfs diff=lfs merge=lfs -text

# Large files (>50MB) - catch-all
*.[Ll][Aa][Rr][Gg][Ee] filter=lfs diff=lfs merge=lfs -text
```

### Step 4: Migrate Existing Files

**If files are already tracked:**

```bash

# Find large files

git ls-files | xargs ls -lh | awk '$5 > 10485760 {print $9}'

# Migrate specific files to LFS

git lfs migrate import --include="*.dylib" --everything
git lfs migrate import --include="*.a" --everything
git lfs migrate import --include="*.so" --everything

# Or migrate by size

git lfs migrate import --include="*" --above=10MB --everything
```

**⚠️ Warning**: Migration rewrites Git history. Coordinate with team before migrating.

---

## Files That Should NOT Use Git LFS

### Build Artifacts (Should be in .gitignore)

These should NOT be in Git at all:

```
build/
*.o
*.obj
*.exe
dist/
*.egg-info/
__pycache__/
*.pyc
*.pyo
*.pyd
```

### Dependencies (Should be downloaded)

These should be downloaded, not stored:

```
node_modules/
vendor/
third_party/ (if downloadable)
```

### Small Files

These can stay in Git:

```
*.txt
*.md
*.json
*.yaml
*.yml
*.toml
*.xml
*.html
*.css
*.js
*.ts
*.py
*.cpp
*.h
*.hpp
*.rs
*.go
```

---

## Current Repository Analysis

### Check Results

**Run the following to identify candidates:**

```bash

# Find large tracked files

git ls-files | xargs -I {} sh -c 'size=$(stat -f%z "{}" 2>/dev/null || echo 0); if [ $size -gt 10485760 ]; then echo "$(numfmt --to=iec-i --suffix=B $size) {}"; fi' | sort -rn

# Find binary files

git ls-files | file -f - | grep -i "binary\|executable\|archive" | cut -d: -f1

# Find specific patterns

git ls-files | grep -E "\.(dylib|so|a|lib|zip|tar|gz|db|sqlite|pdf|jpg|jpeg|png|gif)$"
```

---

## Recommendations Summary

### Immediate Actions (High Priority)

1. **Migrate cache archives to Git LFS** (~22.4 MiB total)
   - `native/third_party/cache/twsapi_macunix.1040.01.zip` (9.9 MiB) ⭐ Highest priority
   - `native/third_party/cache/IntelRDFPMathLib20U2.tar.gz` (5.7 MiB)
   - `native/third_party/cache/protobuf-3.20.3.tar.gz` (5.2 MiB)
   - `native/third_party/cache/twsapi_macunix.1033.01.zip` (1.6 MiB)

2. **Migrate libbid.a to Git LFS** (3.9 MiB)
   - `native/third_party/IntelRDFPMathLib20U2/LIBRARY/libbid.a`
   - Binary static library, should use LFS

3. **Total Size to Migrate**: ~26.3 MiB

4. **Configure .gitattributes** - add LFS patterns for:
   - `native/third_party/cache/*.zip`
   - `native/third_party/cache/*.tar.gz`
   - `native/third_party/**/*.a` (static libraries)

5. **Update .gitignore** - ensure build artifacts are ignored (already mostly done)

### Future Considerations

1. **Set up Git LFS** - if large files are found
2. **Configure .gitattributes** - for file type patterns
3. **Document LFS usage** - in CONTRIBUTING.md
4. **Add LFS to CI/CD** - ensure LFS files are handled correctly

---

## Git LFS Best Practices

### Do Use Git LFS For

- ✅ Large binary files (>10MB)
- ✅ Third-party libraries
- ✅ Media files (images, videos, audio)
- ✅ Documentation PDFs
- ✅ Test databases
- ✅ Vendor archives

### Don't Use Git LFS For

- ❌ Source code (text files)
- ❌ Small files (<1MB typically)
- ❌ Build artifacts (should be in .gitignore)
- ❌ Dependencies (should be downloaded)
- ❌ Configuration files

### Size Guidelines

- **<1MB**: Keep in Git
- **1-10MB**: Consider Git LFS (especially binaries)
- **10-50MB**: Should use Git LFS
- **>50MB**: Must use Git LFS

---

## Related Documentation

- [Git LFS Documentation](https://git-lfs.github.com/)
- [Git LFS Migration Guide](https://github.com/git-lfs/git-lfs/wiki/Tutorial#migrating-existing-repository-data-to-git-lfs)
- .gitignore Best Practices

---

**Status**: Analysis Complete
**Next Steps**: Run file size checks to identify specific candidates
**Action Required**: Review results and configure Git LFS if needed
