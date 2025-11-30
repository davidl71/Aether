# TUI Alternatives Analysis for IBKR Box Spread Trading System

**Date**: 2025-11-20
**Purpose**: Comprehensive analysis of TUI framework alternatives, clarifying KICKS misunderstanding and evaluating modern options

---

## Executive Summary

**Key Finding**: KICKS is **NOT a TUI framework** - it's a CICS replacement for IBM mainframes (MVS 3.8, z/OS, VM/370). It cannot be used as a TUI alternative for modern C++/Python trading applications.

**Current State**: The project has two TUI implementations:

- **C++ TUI**: Using FTXUI (`native/src/tui_app.cpp`) ✅
- **Python TUI**: Using Textual (`python/tui/app.py`) ✅

**Recommendation**: Continue with current implementations. Both FTXUI (C++) and Textual (Python) are excellent choices for trading applications.

---

## Clarification: KICKS is Not a TUI Framework

### What KICKS Actually Is

**KICKS** (KICKS for TSO/CMS) is:

- A **CICS replacement** for IBM mainframe environments
- Designed for **MVS 3.8, z/OS, VM/370** systems
- Allows running CICS applications **without installing CICS**
- Operates in the **user's address space** (unlike CICS which runs in its own address space)
- Uses **CICS-compatible EXEC API** for transaction processing

**Source**: [KICKS for TSO Website](https://www.kicksfortso.com/) - "KICKS is an enhancement for CMS & TSO that lets you run your CICS applications directly instead of having to 'install' those apps in CICS."

### Why KICKS Cannot Be Used

1. **Platform Incompatibility**: Designed for IBM mainframes (MVS 3.8, z/OS), not modern Unix/macOS/Linux
2. **Not a TUI Library**: It's a transaction processing system, not a terminal UI framework
3. **Language Mismatch**: Designed for COBOL/CICS applications, not C++20/Python
4. **Architecture Mismatch**: Mainframe-specific (TSO, CMS, JCL), not applicable to modern trading systems

**Conclusion**: KICKS is completely unrelated to TUI development for modern trading applications.

---

## Current TUI Implementations

### 1. C++ TUI with FTXUI ✅

**Location**: `native/src/tui_app.cpp`
**Framework**: [FTXUI](https://github.com/ArthurSonzogni/FTXUI)
**Status**: Production-ready

**Features**:

- Modern C++20 declarative UI
- Real-time data updates
- Tab navigation (Dashboard, Positions, Orders, Alerts)
- Keyboard shortcuts (Q, F1-F10, Tab, etc.)
- Color support
- Mouse support
- Multiple providers (Mock, REST, File)

**Pros**:

- ✅ Native C++ performance
- ✅ Header-only library (easy CMake integration)
- ✅ Modern declarative API
- ✅ Good documentation
- ✅ Active development
- ✅ Cross-platform (macOS, Linux, Windows)

**Cons**:

- ⚠️ C++ complexity for rapid UI development

### 2. Python TUI with Textual ✅

**Location**: `python/tui/app.py`
**Framework**: [Textual](https://textual.textualize.io/)
**Status**: Production-ready

**Features**:

- Modern Python reactive UI framework
- Shared data models with PWA
- Multiple providers (Mock, REST, File)
- Tab navigation
- Keyboard shortcuts
- Real-time updates

**Pros**:

- ✅ Rapid development
- ✅ Easy Python API integration
- ✅ Shared models with PWA (TypeScript)
- ✅ Excellent documentation
- ✅ Modern reactive framework
- ✅ Rich widget library

**Cons**:

- ⚠️ Python performance overhead (acceptable for TUI)

---

## Modern TUI Framework Alternatives

### For C++

#### 1. FTXUI (Current Choice) ⭐⭐⭐⭐⭐

**Status**: Currently in use
**Why It's Best**:

- Modern C++20 design
- Declarative API
- Header-only (easy CMake integration)
- Active maintenance
- Good performance
- Cross-platform

**Documentation**: https://github.com/ArthurSonzogni/FTXUI

#### 2. notcurses ⭐⭐⭐⭐

**Framework**: [notcurses](https://github.com/dankamongmen/notcurses)

**Features**:

- Modern ncurses replacement
- True color support
- Multimedia support (images, video)
- High performance
- C++ bindings available

**When to Consider**: If you need multimedia support or maximum performance

#### 3. ncurses / ncurses++ ⭐⭐⭐

**Framework**: Traditional ncurses

**Features**:

- Industry standard
- Mature and stable
- Wide platform support

**When to Consider**: If you need maximum compatibility or are already familiar with ncurses

### For Python

#### 1. Textual (Current Choice) ⭐⭐⭐⭐⭐

**Status**: Currently in use
**Why It's Best**:

- Modern reactive framework
- Excellent documentation
- Rich widget library
- Active development
- Built by Textualize (Rich creators)

**Documentation**: https://textual.textualize.io/

#### 2. Rich ⭐⭐⭐⭐

**Framework**: [Rich](https://github.com/Textualize/rich)

**Features**:

- Beautiful terminal output
- Rich formatting
- Progress bars, tables, syntax highlighting

**When to Consider**: For CLI tools or simple displays, not full interactive TUI

#### 3. prompt_toolkit ⭐⭐⭐

**Framework**: [prompt_toolkit](https://github.com/prompt-toolkit/python-prompt-toolkit)

**Features**:

- Advanced input handling
- Auto-completion
- Syntax highlighting

**When to Consider**: For command-line tools with advanced input, not full TUI

---

## Comparison Matrix

| Framework | Language | Modern | Performance | Ease of Use | Widgets | Docs | Rating |
|-----------|----------|--------|-------------|-------------|---------|------|--------|
| **FTXUI** | C++ | ✅ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Textual** | Python | ✅ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| notcurses | C++ | ✅ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| Rich | Python | ✅ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| ncurses | C++ | ❌ | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |

---

## Recommendations

### ✅ Keep Current Implementations

**Why**:

- FTXUI is the best modern C++ TUI framework available
- Textual is the best modern Python TUI framework available
- Both are actively maintained and well-documented
- Both support real-time updates (critical for trading)
- Both have good keyboard/mouse support
- Both meet all trading system requirements

### Trading System Requirements Met

**Core Requirements**:

- ✅ Real-time data updates (500ms-1s refresh)
- ✅ Multiple tabs (Dashboard, Positions, Orders, Alerts)
- ✅ Keyboard shortcuts (Q, F1-F10, Tab navigation)
- ✅ Color support (status indicators, alerts)
- ✅ Multiple data providers (Mock, REST, File)
- ✅ Cross-platform (macOS, Linux)

**Advanced Features**:

- ✅ Modal dialogs/popovers
- ✅ Data tables with sorting
- ✅ Real-time metrics display
- ✅ Alert/notification system
- ✅ Configuration management

---

## Conclusion

**KICKS is not a TUI framework** - it's a CICS replacement for IBM mainframes and cannot be used for modern trading applications.

**Your current TUI implementations are excellent choices**:

- **FTXUI (C++)** is the best modern C++ TUI framework
- **Textual (Python)** is the best modern Python TUI framework

**Recommendation**: Continue with current implementations. Both are well-suited for trading applications with real-time data requirements.

---

## References

- [KICKS for TSO Website](https://www.kicksfortso.com/) - Confirms KICKS is CICS replacement, not TUI
- [FTXUI GitHub](https://github.com/ArthurSonzogni/FTXUI) - Current C++ TUI framework
- [Textual Documentation](https://textual.textualize.io/) - Current Python TUI framework
- [notcurses GitHub](https://github.com/dankamongmen/notcurses) - Alternative C++ TUI
- [Rich GitHub](https://github.com/Textualize/rich) - Python CLI formatting library

---

**Last Updated**: 2025-11-20
**Status**: Analysis Complete ✅
