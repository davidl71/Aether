# TUI Keyboard Shortcuts Reference

Complete reference for all keyboard shortcuts in the IBKR Box Spread TUI.

## Navigation

| Key | Action | Description |
|-----|--------|-------------|
| `Tab` | Next Tab | Cycle forward through tabs |
| `Shift+Tab` | Previous Tab | Cycle backward through tabs |
| `↑` `↓` | Navigate Up/Down | Move selection in lists/tables |
| `←` `→` | Navigate Left/Right | Move selection horizontally |
| `Enter` | Select/Open | Open detail popup for selected item |
| `Esc` | Cancel/Close | Close popup or cancel action |

## Strategy Control

| Key | Action | Description |
|-----|--------|-------------|
| `S` | Start Strategy | Start or resume trading strategy |
| `T` | Stop Strategy | Stop strategy gracefully (waits for orders) |
| `K` | Cancel Orders | Cancel all open strategy orders |
| `D` | Toggle Dry-Run | Switch between dry-run and live mode |

## Trading Actions

| Key | Action | Description |
|-----|--------|-------------|
| `B` | Buy Combo | Submit additional box spread to lower cost basis |
| `Shift+S` | Sell Combo | Submit offsetting spread to realize gains |
| `C` | Close Position | Close selected position (if implemented) |

## View Controls

| Key | Action | Description |
|-----|--------|-------------|
| `?` | Help | Show keyboard shortcuts help |
| `R` | Refresh | Refresh current view (if implemented) |
| `F` | Filter | Toggle filter/search (if implemented) |
| `/` | Search | Open search dialog (if implemented) |

## Application

| Key | Action | Description |
|-----|--------|-------------|
| `Q` | Quit | Exit TUI application |
| `Ctrl+C` | Force Quit | Force exit (emergency) |

## Tab-Specific Shortcuts

### Dashboard Tab
- `Enter` on symbol: Show symbol details
- `Enter` on position: Show position details
- `Enter` on order: Show order details

### Positions Tab
- `Enter` on position: Show full position details
- `B` on position: Buy combo for selected position
- `Shift+S` on position: Sell combo for selected position

### Orders Tab
- `Enter` on order: Show order execution details
- `K` on order: Cancel selected order (if pending)

### Alerts Tab
- `Enter` on alert: Show full alert details
- `C` on alert: Clear/dismiss alert (if implemented)

## Tips

1. **Always check mode**: Look for "DRY-RUN" or "LIVE" indicator in header
2. **Use Tab navigation**: Fastest way to switch between views
3. **Press ? for help**: Shows all shortcuts in-app
4. **Dry-run first**: Press `D` to enable dry-run mode before live trading
5. **Monitor alerts**: Check Alerts tab regularly for system notifications

## Safety Features

- **Dry-Run Mode**: Default safe mode - no real trades executed
- **Confirmation Prompts**: Critical actions require confirmation
- **Visual Indicators**: Color coding shows status at a glance
- **Error Messages**: Clear error messages for failed actions

## Accessibility

- **Keyboard-Only**: Full functionality without mouse
- **Color Blind Friendly**: Uses symbols and text in addition to colors
- **Monochrome Support**: Works in terminals without color support

## See Also

- **Man Page**: `man ib-box-spread-tui` for complete documentation
- **TUI Design**: `docs/TUI_DESIGN.md` for layout details
- **Testing**: `docs/TUI_TESTING.md` for test automation
