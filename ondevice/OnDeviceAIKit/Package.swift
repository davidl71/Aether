// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "OnDeviceAIKit",
    platforms: [
        .macOS(.v13),
        .iOS(.v16)
    ],
    products: [
        .library(name: "OnDeviceAIKit", targets: ["OnDeviceAIKit"]),
        .executable(name: "OnDeviceAIKitExample", targets: ["OnDeviceAIKitExample"]),
        .executable(name: "OnDeviceAIKitCLI", targets: ["OnDeviceAIKitCLI"])
    ],
    targets: [
        .target(
            name: "OnDeviceAIKit",
            path: "Sources/OnDeviceAIKit"
        ),
        .executableTarget(
            name: "OnDeviceAIKitExample",
            dependencies: ["OnDeviceAIKit"],
            path: "Sources/OnDeviceAIKitExample"
        ),
        .executableTarget(
            name: "OnDeviceAIKitCLI",
            dependencies: ["OnDeviceAIKit"],
            path: "Sources/OnDeviceAIKitCLI"
        )
    ]
)


