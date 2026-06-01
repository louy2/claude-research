// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "wcdemo",
    products: [
        .executable(name: "wcdemo", targets: ["wcdemo"]),
        .library(name: "DemoCore", targets: ["DemoCore"]),
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-argument-parser.git", from: "1.5.0"),
    ],
    targets: [
        .target(
            name: "DemoCore"
        ),
        .executableTarget(
            name: "wcdemo",
            dependencies: [
                "DemoCore",
                .product(name: "ArgumentParser", package: "swift-argument-parser"),
            ]
        ),
        .testTarget(
            name: "DemoCoreTests",
            dependencies: ["DemoCore"]
        ),
    ]
)
