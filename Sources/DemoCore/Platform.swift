import Foundation

/// Describes the platform the binary was compiled for. The values are resolved
/// entirely at compile time via target conditionals, which makes this a handy
/// smoke test for cross-compilation: a Windows build reports "Windows" even
/// when produced on a Linux host.
public enum Platform {
    public static var osName: String {
        #if os(Windows)
        return "Windows"
        #elseif os(macOS)
        return "macOS"
        #elseif os(Linux)
        return "Linux"
        #else
        return "Unknown"
        #endif
    }

    public static var arch: String {
        #if arch(x86_64)
        return "x86_64"
        #elseif arch(arm64)
        return "arm64"
        #elseif arch(i386)
        return "i386"
        #else
        return "unknown"
        #endif
    }

    /// The native path separator for the target platform.
    public static var pathSeparator: Character {
        #if os(Windows)
        return "\\"
        #else
        return "/"
        #endif
    }

    public static var description: String {
        "\(osName) (\(arch))"
    }
}
