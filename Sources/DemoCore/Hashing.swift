import Foundation

/// A small collection of dependency-free hashing helpers used by the CLI.
///
/// These are deliberately implemented from scratch (rather than reaching for
/// `Crypto`) so the demo stays free of C/system crypto dependencies and
/// cross-compiles cleanly to every target.
public enum Hashing {
    /// 64-bit FNV-1a hash of an arbitrary byte sequence.
    public static func fnv1a64<S: Sequence>(_ bytes: S) -> UInt64 where S.Element == UInt8 {
        var hash: UInt64 = 0xcbf2_9ce4_8422_2325
        let prime: UInt64 = 0x0000_0100_0000_01b3
        for byte in bytes {
            hash ^= UInt64(byte)
            hash = hash &* prime
        }
        return hash
    }

    /// FNV-1a hash of a file's contents, returned as a zero-padded hex string.
    public static func fnv1a64(ofFileAt url: URL) throws -> String {
        let data = try Data(contentsOf: url)
        let value = fnv1a64(data)
        return String(format: "%016llx", value)
    }
}
