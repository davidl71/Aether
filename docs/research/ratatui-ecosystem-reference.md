# Ratatui Ecosystem Research Document

**Date**: March 2026  
**Purpose**: Comprehensive reference of Ratatui/TUI ecosystem crates for Aether TUI development  
**Scope**: Widgets, frameworks, utilities, and applications built with Ratatui

---

## Executive Summary

This document catalogs 50+ crates in the Ratatui ecosystem, organized by category and relevance to Aether's TUI service. The ecosystem is vibrant with community contributions spanning widgets, state management, testing, and developer tooling.

**Key Findings**:
- **Core Framework**: Ratatui 0.30+ with modular architecture (ratatui-core, ratatui-widgets)
- **Input Abstraction**: terminput provides multi-backend support (crossterm, termion, termwiz)
- **Rich Widget Ecosystem**: 30+ specialized widgets for forms, navigation, visualization
- **Developer Experience**: tui-pantry provides Storybook-like component development

---

## Table of Contents

1. [Frameworks & State Management](#frameworks--state-management)
2. [Widget Composition & Styling](#widget-composition--styling)
3. [Input Handling](#input-handling)
4. [Data Display Widgets](#data-display-widgets)
5. [Forms & Input Widgets](#forms--input-widgets)
6. [Feedback & Status Widgets](#feedback--status-widgets)
7. [Utilities](#utilities)
8. [Testing & Performance](#testing--performance)
9. [Developer Tools](#developer-tools)
10. [Applications](#applications)
11. [Recommendations for Aether](#recommendations-for-aether)

---

## Frameworks & State Management

### rat-salsa
- **Author**: thscharler
- **Repository**: https://github.com/thscharler/rat-salsa
- **Downloads**: 28K+
- **Purpose**: Application event-loop with tasks, timers, focus handling, dialog windows
- **Key Features**:
  - Poll multiple event-sources fairly
  - Run background tasks in worker threads or async
  - Define timers and message queue
  - Focus handling with SalsaContext
  - Autonomous dialog window stack
- **Aether Relevance**: ⭐⭐⭐⭐⭐ Reference for event handling architecture

### rat-widget (Part of rat-salsa)
- **Author**: thscharler
- **Downloads**: 39K+
- **Purpose**: Extended widget suite with focus management
- **Widgets Included**:
  - Input: TextInput, MaskedInput, DateInput, NumberInput
  - Structural: Tabbed, Split, View, Multi-page
  - Data: EditTable, EditList, FileDialog
  - UI: Menubar, StatusLine, Calendar, Checkbox, Choice
- **Features**: Built-in event handling, focus management, scrolling
- **Aether Relevance**: ⭐⭐⭐⭐⭐ Comprehensive widget library

### ratatui-interact
- **Author**: Brainwires
- **Repository**: https://github.com/Brainwires/ratatui-interact
- **Purpose**: Interactive TUI components with focus management and mouse support
- **Components** (30+ total):
  - **Interactive**: CheckBox, Input, TextArea, Button, Select, ContextMenu, MenuBar, PopupDialog, HotkeyDialog
  - **Display**: AnimatedText, Toast, Progress, MarqueeText, Spinner, MousePointer
  - **Navigation**: ListPicker, TreeView, FileExplorer, Accordion, Breadcrumb
  - **Layout**: TabView, SplitPane
  - **Viewer**: LogViewer, DiffViewer, StepDisplay, ScrollableContent
- **Utilities**: ANSI parsing, View/Copy mode, exit strategies
- **Aether Relevance**: ⭐⭐⭐⭐⭐ **Most comprehensive component library**

### tuirealm
- **Author**: veeso
- **Downloads**: 170K+
- **Purpose**: Ratatui framework inspired by Elm and React
- **Features**:
  - Event-driven architecture
  - React/Elm patterns with messages and events
  - Re-usable components with properties and states
  - View management (mounting/unmounting, focus, event forwarding)
  - Standard library of components available
- **Aether Relevance**: ⭐⭐⭐ Alternative to current App state

### tui-react
- **Author**: Byron (Sebastian Thiel)
- **Downloads**: 333K+
- **Purpose**: TUI widgets using React-like paradigm with mutable state
- **Features**:
  - Stateful components
  - TopLevelComponent trait for rendering
  - Flexible state management
- **Aether Relevance**: ⭐⭐⭐ State management reference

### widgetui
- **Author**: EmeraldPandaTurtle
- **Downloads**: 264
- **Purpose**: Bevy-like widget system for Ratatui
- **Pattern**: Resource-based (ResMut<WidgetFrame>, ResMut<Events>)
- **Aether Relevance**: ⭐⭐ Different paradigm, less relevant

---

## Widget Composition & Styling

### ratatui-garnish
- **Author**: franklaranja
- **Repository**: https://github.com/franklaranja/ratatui-garnish
- **Purpose**: Powerful composition system for Ratatui widgets
- **Features**:
  - Layer borders, titles, shadows, padding, styles
  - Type-safe (no trait objects)
  - Zero-cost abstractions
  - Composable in any order
  - Runtime modification
- **Example**:
  ```rust
  Text::raw("Hello")
      .garnish(RoundedBorder::default())
      .garnish(Title::<Above>::raw("My App"))
      .garnish(Style::default().bg(Color::Blue))
      .garnish(Padding::uniform(1));
  ```
- **Aether Relevance**: ⭐⭐⭐⭐⭐ Excellent for consistent widget styling

### ratatui-macros
- **Repository**: https://github.com/ratatui/ratatui-macros
- **Status**: **Merged into main Ratatui crate**
- **Purpose**: Macros for simplifying UI boilerplate
- **Macros**: constraints!, horizontal!, vertical!, span!, line!
- **Note**: No longer a separate dependency; use main ratatui crate

---

## Input Handling

### tui-input
- **Author**: sayanarijit
- **Downloads**: 1.1M+
- **Purpose**: Headless input library for TUI apps
- **Features**:
  - Multi-backend support (crossterm, termion)
  - ratatui integration
  - Input buffer with cursor support
  - InputRequest/InputResponse pattern
- **Aether Relevance**: ⭐⭐⭐⭐⭐ **Essential for text input**

### tui-prompts
- **Author**: joshka
- **Downloads**: 112K+
- **Purpose**: Friendly prompts and input flows
- **Prompts**: Text, Password, Invisible, Number, Confirm, List, Toggle, Select, Multi-select, Autocomplete, Date
- **Features**:
  - Readline/emacs key bindings
  - Soft wrapping
  - Multi-line input
  - Scrolling
  - Backend agnostic
- **Aether Relevance**: ⭐⭐⭐⭐⭐ **Excellent for user dialogs**

### terminput
- **Author**: aschey
- **Downloads**: 42K+
- **Purpose**: Abstraction over various input backends
- **Supported Backends**:
  - crossterm (0.28, 0.29)
  - termion (4)
  - termwiz (0.22, 0.23)
  - termina (0.1)
  - egui (0.32, 0.33)
  - web-sys (0.3)
- **Features**:
  - Event parsing from ANSI sequences
  - Event encoding for child PTY
  - Feature matrix support
- **Aether Relevance**: ⭐⭐⭐⭐ Multi-backend support reference

### termprofile
- **Author**: aschey
- **Purpose**: Detect and handle terminal color/styling support
- **Detection Levels**:
  - True color (24-bit RGB)
  - ANSI 256 (indexed colors)
  - ANSI 16 (basic colors)
  - No Color (modifiers only)
  - No TTY (no escape sequences)
- **Aether Relevance**: ⭐⭐⭐ Theme adaptation

---

## Data Display Widgets

### tui-tree-widget
- **Author**: EdJoPaTo
- **Downloads**: 548K+
- **Purpose**: Tree view with expand/collapse
- **Features**:
  - TreeItem with children
  - TreeState for selection
  - Scrollbar support
- **Aether Relevance**: ⭐⭐⭐⭐⭐ **Hierarchical data (accounts, strategies)**

### tui-widget-list
- **Author**: preiter93
- **Downloads**: 113K+
- **Purpose**: Versatile widget list implementation
- **Features**:
  - Horizontal/vertical scrolling
  - Scroll padding
  - Infinite scrolling
  - Mouse support (hit_test)
  - ListBuilder pattern
- **Aether Relevance**: ⭐⭐⭐⭐ Custom scrollable lists

### tui-nodes
- **Author**: jaxter184
- **Downloads**: 18K+
- **Purpose**: Node graph visualization
- **Aether Relevance**: ⭐⭐ Complex relationship visualization

### tui-piechart
- **Author**: sorinirimies
- **Downloads**: 2K+
- **Purpose**: Pie chart widget with high-resolution mode
- **Features**:
  - Standard and high resolution (braille patterns)
  - Customizable colors, labels, percentages
  - Legend support with positioning
  - Custom symbols
- **Aether Relevance**: ⭐⭐⭐ Portfolio allocation charts

### tui-big-text
- **Purpose**: Large text display using font8x8
- **Aether Relevance**: ⭐⭐ Headers/titles

---

## Forms & Input Widgets

### tui-checkbox
- **Author**: sorinirimies
- **Downloads**: 1K+
- **Purpose**: Customizable checkbox widget
- **Features**:
  - Custom symbols (Unicode, emoji, ASCII)
  - Separate styling for checkbox and label
  - Label positioning
  - Block wrapper
- **Aether Relevance**: ⭐⭐⭐⭐ Settings toggles

### tui-slider
- **Author**: sorinirimies
- **Downloads**: 568
- **Purpose**: Horizontal/vertical sliders
- **Features**:
  - Customizable symbols, colors
  - Label and value display
  - State management (SliderState)
  - Keyboard/mouse control
- **Aether Relevance**: ⭐⭐⭐ Volume/progress indicators

### tui-menu
- **Repository**: https://github.com/shuoli84/tui-menu
- **Purpose**: Nested menu bar (File/Edit/About)
- **Features**:
  - Sub-menu groups
  - Intuitive movement
  - Generic item data (Clone)
- **Aether Relevance**: ⭐⭐⭐ Menu system

### tui-tabs
- **Purpose**: Tab navigation with bordered boxes
- **Features**:
  - Individual bordered boxes
  - Rounded junction corners
  - Highlight styling
- **Aether Relevance**: ⭐⭐⭐⭐ Enhanced tab navigation

### ratatui-textarea
- **Purpose**: Text editor widget
- **Features**: vim-like editing, multi-line
- **Aether Relevance**: ⭐⭐⭐ Note-taking/editing

---

## Feedback & Status Widgets

### tui-logger
- **Author**: gin66
- **Downloads**: 1.4M+
- **Purpose**: Logger with smart widget
- **Features**:
  - Log filtering by target
  - Hot/cold log separation
  - Circular buffer
  - Target selector widget
  - Scrollback
  - slog and tracing support
  - File logging
- **Aether Relevance**: ⭐⭐⭐⭐⭐ **Essential for Logs tab**

### ratatui-toaster
- **Author**: JayanAXHF
- **Downloads**: 333
- **Purpose**: Toast notifications
- **Features**:
  - Customizable messages and durations
  - Toast types (success, error, info)
  - Tokio integration
  - Auto-expiry
- **Aether Relevance**: ⭐⭐⭐⭐⭐ **Toast notifications (existing task)**

### throbber-widgets-tui
- **Author**: arkbig
- **Downloads**: 512K+
- **Purpose**: Loading spinners/activity indicators
- **Features**:
  - 12+ frame styles (dots, braille, line)
  - With labels
  - Random or specified step
- **Aether Relevance**: ⭐⭐⭐⭐ Loading states

### tui-popup
- **Author**: joshka (now part of tui-widgets)
- **Purpose**: Centered popup widget
- **Aether Relevance**: ⭐⭐⭐⭐ Modal dialogs

### tui-statusbar
- **Author**: kdheepak
- **Purpose**: Status bar widget
- **Aether Relevance**: ⭐⭐⭐ Bottom status

---

## Utilities

### ansi-to-tui
- **Author**: ratatui-org
- **Downloads**: 1.8M+
- **Purpose**: Convert ANSI color codes to Ratatui Text
- **Features**:
  - SGR sequence parsing
  - Named colors (3/4-bit)
  - Indexed colors (8-bit, 256-color)
  - Truecolor (24-bit RGB)
  - Zero-copy option
  - SIMD support
- **Aether Relevance**: ⭐⭐⭐⭐⭐ **Styled log output**

### color-to-tui
- **Purpose**: Parse colors and convert to ratatui::style::Colors
- **Aether Relevance**: ⭐⭐ Theme customization

### tachyonfx
- **Purpose**: Shader-like effects library
- **Aether Relevance**: ⭐⭐ Visual polish

---

## Testing & Performance

### rlt
- **Author**: wfxr
- **Repository**: https://github.com/wfxr/rlt
- **Downloads**: 369K+
- **Purpose**: Rust Load Testing framework with real-time TUI
- **Features**:
  - Universal framework (HTTP, gRPC, DB, custom)
  - Real-time TUI monitoring
  - Rich statistics
  - Stateless and stateful benchmarks
  - CLI with standard options
- **Aether Relevance**: ⭐⭐⭐⭐⭐ **Backend API testing**

### xan
- **Author**: medialab
- **Repository**: https://github.com/medialab/xan
- **Purpose**: CSV magician with SIMD parsing
- **Features**:
  - SIMD CSV parser
  - Expression language
  - Terminal visualizations (histograms, scatter plots)
  - Streaming processing
- **Aether Relevance**: ⭐⭐⭐⭐ Data export/import patterns

---

## Developer Tools

### tui-pantry
- **Author**: jharsono
- **Repository**: https://github.com/taho-inc/tui-pantry
- **Purpose**: Component-driven development tool (Storybook for Ratatui)
- **Features**:
  - Build/preview widgets in isolation
  - Interactive terminal browser
  - Color depth emulation
  - Stylesheet support
  - Four tabs: Widgets, Panes, Views, Styles
  - `cargo pantry` subcommand
- **Usage**:
  ```bash
  cargo install tui-pantry
  cargo pantry init  # Scaffold everything
  cargo pantry       # Open the pantry
  ```
- **Aether Relevance**: ⭐⭐⭐⭐⭐ **Widget development workflow**

---

## Applications

### ATAC
- **Author**: Julien-cpsn
- **Repository**: https://github.com/Julien-cpsn/ATAC
- **Stars**: 3.2K+
- **Purpose**: Terminal API client (Postman/Insomnia alternative)
- **Philosophy**: Free, account-less, offline
- **TUI Dependencies**:
  - ratatui 0.30.0
  - tui-big-text
  - tui-tree-widget
  - tui-scrollview
  - tui-textarea
  - ratatui-image
- **Features**:
  - Collection/request management
  - HTTP methods (GET, POST, PUT, etc.)
  - Authentication (Basic, Bearer)
  - WebSocket support
  - Pre/post-request scripts
  - Vim keybindings
  - Import from Postman
- **Aether Relevance**: ⭐⭐⭐⭐⭐ **Reference TUI application architecture**

### Chamber
- **Author**: mikeleppane
- **Repository**: https://github.com/mikeleppane/chamber
- **Purpose**: Secure, local-first secrets manager
- **TUI Framework**: Ratatui + Crossterm
- **Features**:
  - ChaCha20-Poly1305 encryption with Argon2 key derivation
  - SQLite backend with WAL mode
  - Multiple export formats (JSON, CSV, Backup)
  - Backup system with automatic protection
  - Flexible item types (passwords, API keys, SSH keys, certificates)
  - Zero-knowledge architecture with local-only storage
- **Aether Relevance**: ⭐⭐⭐⭐ Reference for secure TUI applications

### Envx
- **Author**: mikeleppane
- **Repository**: https://github.com/mikeleppane/envx
- **Purpose**: Environment variable manager with TUI
- **TUI Framework**: Ratatui
- **Features**:
  - Secure environment variable management
  - Intuitive Terminal User Interface
  - Comprehensive CLI
- **Aether Relevance**: ⭐⭐⭐ Configuration management patterns

### Feedr
- **Author**: bahdotsh
- **Repository**: https://github.com/bahdotsh/feedr
- **Purpose**: Terminal-based RSS feed reader
- **TUI Framework**: Ratatui
- **Features**:
  - Dashboard view with latest articles
  - Feed management with categories
  - Starred articles
  - Advanced filtering (category, age, author, read status)
  - Dual themes (dark cyberpunk, light zen)
  - Live search across feeds
  - OPML import
  - Vim-style navigation (j/k)
  - Background refresh with rate limiting
  - Authenticated feeds support
- **Aether Relevance**: ⭐⭐⭐⭐ Complex list/table navigation patterns

### FiTui
- **Author**: ayanchavand
- **Repository**: https://github.com/ayanchavand/fitui
- **Purpose**: Personal finance tracker
- **TUI Framework**: Ratatui
- **Features**:
  - Transaction management (add, edit, delete)
  - Stats view with spending breakdowns by tag
  - Recurring transactions
  - Local SQLite storage
  - Configurable tags and currency
  - Keyboard-driven interface
- **Aether Relevance**: ⭐⭐⭐⭐ Financial data visualization patterns

### fzf-make
- **Author**: kyu08
- **Repository**: https://github.com/kyu08/fzf-make
- **Purpose**: Command runner with fuzzy finder and preview window
- **TUI Framework**: Ratatui
- **Supported Tools**: make, npm, pnpm, yarn, just, task
- **Features**:
  - Fuzzy finding commands
  - Preview window
  - Multiple build tool support
- **Aether Relevance**: ⭐⭐⭐ Fuzzy search + preview patterns

### get-blessed
- **Author**: josueBarretogit
- **Repository**: https://github.com/josueBarretogit/get_blessed_rs
- **Purpose**: Curated crate discovery tool (blessed.rs client)
- **TUI Framework**: Ratatui
- **Features**:
  - Browse blessed.rs curated crates
  - View documentation (press `<d>`)
  - Open crates.io page (press `<c>`)
  - Category navigation with Tab/Shift+Tab
  - Vim-style navigation (j/k)
  - Select crates with features (press `<f>`)
  - Add selected crates to project (press `<Enter>`)
- **Aether Relevance**: ⭐⭐⭐⭐ Selection + action patterns

### Invoice Pilot
- **Author**: adolfousier
- **Repository**: https://github.com/adolfousier/invoicepilot
- **Purpose**: Automated invoice and bank statement management
- **TUI Framework**: Ratatui
- **Backend**: Rust, PostgreSQL, Docker
- **Features**:
  - Gmail integration for fetching invoices/statements
  - Automatic financial institution detection
  - Document attachment downloads (PDF, CSV, XLS, etc.)
  - Google Drive organization with smart filenames
  - Duplicate detection and skipping
  - Missing month backfill
  - Email notifications
  - OAuth2 authentication with token caching
  - Scheduled execution via Docker
- **Aether Relevance**: ⭐⭐⭐⭐ External API integration patterns

### Oracle
- **Author**: yashksaini-coder
- **Repository**: https://github.com/yashksaini-coder/oracle
- **Purpose**: Rust code inspector for the terminal
- **TUI Framework**: Ratatui
- **Features**:
  - Parses Rust source files using `syn`
  - Browse functions, structs, enums, traits, impl blocks
  - Smart search with fuzzy matching and real-time filtering
  - Dependency analysis (Cargo.toml visualization)
  - Multiple themes (Dark, Nord, Catppuccin Mocha, Dracula)
  - Smooth animations and tab transitions
  - Vim-style navigation (j/k, / for search)
- **Aether Relevance**: ⭐⭐⭐⭐ Code navigation and tree view patterns

### Rex
- **Author**: TheRustyPickle
- **Repository**: https://github.com/TheRustyPickle/Rex
- **Purpose**: Terminal UI for managing incomes, expenses, and transactions
- **TUI Framework**: Ratatui
- **Features**:
  - View, add, edit, delete transactions
  - Real-time balance changes
  - Chart for visualizing balance changes (month/year/all)
  - Summary with income/expense insights
  - SQLite database (local storage)
  - Search transactions with partial matching
  - Custom tags for filtering
  - Works fully offline
- **Aether Relevance**: ⭐⭐⭐⭐ Financial data visualization and charting

### Sheetsui
- **Author**: zaphar
- **Repository**: https://github.com/zaphar/sheetsui
- **Purpose**: Terminal spreadsheet viewer and editor
- **TUI Framework**: Ratatui (presumed)
- **Features**:
  - View and edit xlsx files
  - CSV import/export (planned)
  - Locale and timezone support
  - Built on ironcalc engine
- **Installation**: `cargo install --git https://github.com/zaphar/sheetsui`
- **Aether Relevance**: ⭐⭐⭐⭐ Data table/grid editing patterns

### Tabiew
- **Author**: shshemi
- **Repository**: https://github.com/shshemi/tabiew
- **Purpose**: Lightweight TUI for viewing and querying tabular data
- **TUI Framework**: Ratatui
- **Features**:
  - Vim-style keybindings
  - SQL query support
  - Multiple formats: CSV, TSV, Parquet, JSON, JSONL, Arrow, FWF, SQLite, Excel, Logfmt
  - Fuzzy search
  - Scripting support
  - Multi-table functionality
  - Plotting
  - 400+ themes
- **Aether Relevance**: ⭐⭐⭐⭐⭐ **Data exploration and SQL interface patterns**

### tickrs
- **Author**: tarkah
- **Repository**: https://github.com/tarkah/tickrs
- **Purpose**: Real-time stock ticker data in terminal
- **TUI Framework**: Rust TUI (custom or crossterm-based)
- **Data Source**: Yahoo! Finance
- **Features**:
  - Real-time ticker data streaming
  - Multiple chart types (line, candle, kagi)
  - Time frames (1D, 1W, 1M, 3M, 6M, 1Y, 5Y)
  - Pre/post market hours support
  - Volume graphs
  - Summary mode
  - Configurable update intervals
- **Installation**: `brew install tickrs`
- **Aether Relevance**: ⭐⭐⭐⭐⭐ **Real-time financial data streaming patterns**

### TSHTS
- **Author**: SamuelSchlesinger
- **Repository**: https://github.com/SamuelSchlesinger/tshts
- **Purpose**: Terminal-based spreadsheet application
- **Language**: Rust (Edition 2024)
- **Features**:
  - Formula engine with arithmetic, string, and logical operations
  - Functions: SUM, AVERAGE, MIN, MAX, IF, AND, OR, CONCAT, LEN, etc.
  - Cell references (A1, B2, AA123) and range support (A1:C3)
  - CSV import/export
  - Search across cells and formulas (`/`)
  - Undo/redo support
  - Autofill functionality
  - Circular reference detection
  - Vim-style navigation (hjkl) + arrow keys
- **Aether Relevance**: ⭐⭐⭐⭐ Formula engine and grid calculation patterns

### openapi-tui
- **Author**: zaghaghi
- **Repository**: https://github.com/zaghaghi/openapi-tui
- **Purpose**: Terminal UI to browse and run OpenAPI v3.0/v3.1 APIs
- **TUI Framework**: Ratatui
- **Features**:
  - OpenAPI spec browser (JSON/YAML)
  - List and browse API endpoints
  - Execute API calls from TUI
  - Nested component references
  - Webhooks support
  - Multiple server support
  - Filter/search functionality
  - Fullscreen mode
  - Remote spec loading via URL
- **Aether Relevance**: ⭐⭐⭐⭐ API documentation and testing interface patterns

---

## Deep Learning Framework (Separate Category)

### Burn
- **Organization**: tracel-ai
- **Repository**: https://github.com/tracel-ai/burn
- **Stars**: 10K+
- **Purpose**: Next-generation tensor library and deep learning framework
- **Core Principles**:
  - Hardware portability (CPU, CUDA, ROCm, Metal, Vulkan, WebGPU, WASM)
  - Zero-cost abstractions
  - Flexibility (training and inference)
- **Key Technologies**:
  - CubeCL: Multi-platform compute language
  - CubeK: High-performance kernels
  - Dynamic graphs with static performance
- **Backends**: WGPU, CUDA, ROCm, Metal, Vulkan, NdArray
- **Features**:
  - ONNX import/export
  - Autodiff
  - no_std support
  - WebAssembly deployment
- **Note**: Deep learning framework, not directly related to TUI development
- **Aether Relevance**: ⭐⭐ Potential for ML-powered analytics (future consideration)

---

## Recommendations for Aether

### Tier 1: Immediate Integration

| Crate | Use Case | Priority |
|-------|----------|----------|
| **ratatui-interact** | Focus management, mouse support, forms | ⭐⭐⭐⭐⭐ |
| **tui-logger** | Logs tab consolidation | ⭐⭐⭐⭐⭐ |
| **ratatui-toaster** | Toast notifications | ⭐⭐⭐⭐⭐ |
| **tui-input** | Text input state management | ⭐⭐⭐⭐⭐ |
| **ansi-to-tui** | Styled log output | ⭐⭐⭐⭐⭐ |
| **tui-pantry** | Widget development workflow | ⭐⭐⭐⭐⭐ |

### Tier 2: High Value

| Crate | Use Case |
|-------|----------|
| **tui-tree-widget** | Account/strategy hierarchy |
| **tui-tabs** | Enhanced tab navigation |
| **tui-popup** | Modal dialogs |
| **throbber-widgets-tui** | Loading indicators |
| **tui-prompts** | User input dialogs |
| **ratatui-garnish** | Consistent widget styling |

### Tier 3: Architecture Reference

| Crate | Use Case |
|-------|----------|
| **rat-salsa** | Event loop patterns |
| **terminput** | Multi-backend input handling |
| **termprofile** | Terminal capability detection |
| **ATAC** | Complex TUI application patterns |

---

## Statistics Summary

- **Total Crates Reviewed**: 50+
- **Tier 1 (Immediate)**: 6 crates
- **Tier 2 (High Value)**: 6 crates
- **Tier 3 (Reference)**: 4 crates
- **Official ratatui-org**: 10+
- **Community Crates**: 40+
- **Applications**: 16 (ATAC, Chamber, Envx, Feedr, FiTui, fzf-make, get-blessed, Invoice Pilot, Oracle, Rex, Sheetsui, Tabiew, tickrs, TSHTS, openapi-tui, and more)
- **Deep Learning**: 1 (Burn)

---

## Related Aether Tasks

- T-1774457062545031000: Consolidate TUI Alerts and Logs into a single tab
- T-1774285315059562000: Implement TUI Toast notifications
- T-1774477098254544000: Evaluate rlt integration for backend API load testing

---

## Document Maintenance

**Last Updated**: March 2026  
**Review Schedule**: Quarterly  
**Owner**: Aether Development Team

---

*This document is a living reference. Add new crates as they are discovered and update relevance ratings based on evolving requirements.*
