// swift-tools-version: 6.0

import PackageDescription

let package = Package(
  name: "IBBoxSpreadDesktop",
  platforms: [
    .macOS(.v13)
  ],
  products: [
    .executable(
      name: "IBBoxSpreadDesktop",
      targets: ["IBBoxSpreadDesktopApp"]
    )
  ],
  targets: [
    .executableTarget(
      name: "IBBoxSpreadDesktopApp",
      path: "Sources/IBBoxSpreadDesktopApp"
    ),
    .testTarget(
      name: "IBBoxSpreadDesktopAppTests",
      dependencies: ["IBBoxSpreadDesktopApp"],
      path: "Tests/IBBoxSpreadDesktopAppTests"
    )
  ]
)
