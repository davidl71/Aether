# IB Box Spread Desktop (SwiftUI)

This directory packages the macOS SwiftUI shell that will visualize box spread analytics.

## Getting Started

1. Install Xcode 15 or newer (Swift 6 toolchain).
2. From this directory, run `swift build` to compile the executable.
3. Run `swift run` to launch the placeholder window.
4. Add your unit or snapshot tests under `Tests/` and they will run via `swift test`.

## Project Layout

- `Package.swift` declares an executable Swift Package targeting macOS 13 and newer.
- `Sources/IBBoxSpreadDesktopApp/` holds the SwiftUI `App` declaration and initial view hierarchy.
- `Tests/` is empty for now—mirror production module names when introducing coverage.

## Next Steps

- Replace the placeholder `ContentView` summary metrics with real pricing data pulled from the shared C++ core (via bridging header, C interop, or IPC service).
- Introduce previews and snapshot tests to validate layout changes.
- Integrate CI by teaching the desktop agent to run `swift test` in addition to repository-wide checks.
