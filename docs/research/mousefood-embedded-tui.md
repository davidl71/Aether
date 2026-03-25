# Mousefood Embedded TUI Backend Research

**Source:** [ratatui/mousefood](https://github.com/ratatui/mousefood)  
**Date:** 2026-03-26  
**Status:** Completed

## Overview

Mousefood is a no-std embedded-graphics backend for Ratatui. It enables rendering Ratatui TUI applications to embedded displays (LCD, OLED, e-paper) via the embedded-graphics ecosystem.

## Current Aether TUI Architecture

```
Backend: crossterm 0.28 (terminal emulator)
Target: Standard terminal/SSH sessions
Platform: Desktop/server environments
Location: agents/backend/services/tui_service/
```

Current dependencies in `Cargo.toml`:
```toml
ratatui   = "0.30"
crossterm = { version = "0.28", features = ["event-stream"] }
```

## Mousefood Capabilities

| Feature | Details |
|---------|---------|
| **Backend Type** | embedded-graphics → Ratatui bridge |
| **Std Support** | no_std compatible |
| **License** | MIT OR Apache-2.0 |
| **Latest Version** | v0.5.0 (MSRV 1.86) |
| **Crate Size** | 18.0 KB |

### Supported Microcontrollers

- ESP32 (Xtensa)
- ESP32-C6 (RISC-V)
- STM32
- RP2040
- RP2350

### Supported Displays

| Driver | Displays |
|--------|----------|
| ssd1306 | SSD1306 OLED |
| mipidsi | ILI9341, ST7735, etc. |
| epd-waveshare | Waveshare e-paper displays |
| weact-studio-epd | WeAct Studio e-paper displays |

### Font Handling

- **Default**: `embedded-graphics-unicodefonts` — supports box-drawing glyphs, Braille, and special characters required by Ratatui widgets
- **Alternative**: Disable `fonts` feature and use smaller fonts like `ibm437` for space-constrained deployments
- **Custom**: Bold/italic font support via `EmbeddedBackendConfig`

### Color Support

- Color theme remapping via `color_theme` on `EmbeddedBackendConfig`
- Default: ANSI palette
- Supports RGB888 and other pixel color formats

## Implementation Pattern

```rust
use mousefood::embedded_graphics::{mock_display::MockDisplay, pixelcolor::Rgb888};
use mousefood::prelude::*;
use ratatui::widgets::{Block, Paragraph};
use ratatui::{Frame, Terminal};

fn main() -> Result<(), Box<dyn Error>> {
    let mut display = MockDisplay::<Rgb888>::new();
    let backend = EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());
    let mut terminal = Terminal::new(backend)?;
    
    terminal.draw(draw)?;
    Ok(())
}

fn draw(frame: &mut Frame) {
    let block = Block::bordered().title("Mousefood");
    let paragraph = Paragraph::new("Hello from Mousefood!").block(block);
    frame.render_widget(paragraph, frame.area());
}
```

## Potential Use Cases for Aether

| Scenario | Value | Effort |
|----------|-------|--------|
| **Kiosk/Status Display** | Wall-mounted market overview display | Medium |
| **Embedded Edge Device** | Field-deployed monitoring unit (no desktop) | High |
| **E-paper Dashboard** | Low-power always-on portfolio summary | Medium |
| **Secondary Screen** | HDMI/DSI display alongside main terminal | Low |

## Integration Assessment

### Can Coexist with Crossterm?

✅ **Yes** — feature-flag approach:

```toml
[features]
default = ["terminal"]
embedded = ["mousefood", "embedded-graphics"]

[dependencies]
crossterm = { version = "0.28", optional = true, features = ["event-stream"] }
mousefood = { version = "0.5", optional = true }
```

### Required Code Changes

1. **Backend Selection**: Terminal vs embedded at startup
2. **Display Size**: Fixed resolution for embedded (vs flexible terminal)
3. **Input Handling**: GPIO/buttons vs keyboard
4. **Build Targets**: Cross-compilation for embedded targets

### Dependencies Impact

```toml
[dependencies]
# Existing (terminal only)
ratatui = "0.30"
crossterm = { version = "0.28", features = ["event-stream"] }

# Additional (embedded only)
mousefood = "0.5"
embedded-graphics = "0.8"
# + display-specific driver (ssd1306, mipidsi, etc.)
```

## Evaluation for Aether

| Aspect | Assessment |
|--------|------------|
| **Complexity** | Medium — adds embedded toolchain complexity |
| **Maintenance** | Moderate — separate build targets, hardware testing |
| **Value** | Low-Current — no embedded deployment requirement exists |
| **Integration** | Straightforward — feature flags, conditional compilation |

### Pros

- Opens embedded deployment options (kiosk, edge device, e-paper)
- Clean Ratatui code reuse — same widgets, different backend
- Hardware-agnostic — supports many display types
- Active development — part of ratatui official ecosystem

### Cons

- Current Aether is desktop/terminal-first
- No embedded deployment requirement exists
- Adds build complexity (cross-compilation, hardware testing)
- Maintenance surface increases

## Recommendation

**DEFER** — Do not implement now. Document for future reference.

### Rationale

1. **Focus Mismatch**: Aether is currently desktop/terminal-first; no embedded deployment requirement exists
2. **Maintenance Surface**: Adds embedded toolchain complexity (cross-compilation, hardware testing)
3. **Priority Context**: Active work on TUI UX foundation, NATS health metrics, and workspace navigation takes precedence
4. **Future Option**: Can revisit if/when field hardware deployment becomes a requirement

### When to Revisit

Consider Mousefood integration if:
- Field deployment without desktop infrastructure becomes a requirement
- Kiosk/wall-mounted display use case emerges
- Low-power always-on dashboard (e-paper) is requested
- Edge device monitoring without SSH access is needed

## Related Resources

- [Mousefood GitHub](https://github.com/ratatui/mousefood)
- [Mousefood on crates.io](https://crates.io/crates/mousefood)
- [Mousefood Documentation](https://docs.rs/mousefood)
- [Ratatui Embedded Guide](https://ratatui.rs/ecosystem/mousefood/)
- [embedded-graphics crate](https://crates.io/crates/embedded-graphics)

## Related Tasks

None currently. Future work could be tracked if embedded deployment becomes a priority.
