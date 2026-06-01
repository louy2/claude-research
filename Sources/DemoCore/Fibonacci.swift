/// Overflow-aware Fibonacci, used to exercise pure integer computation in the
/// cross-compiled binary.
public enum Fibonacci {
    /// Returns the nth Fibonacci number, or `nil` if it overflows `UInt64`.
    public static func value(_ n: Int) -> UInt64? {
        guard n >= 0 else { return nil }
        if n < 2 { return UInt64(n) }
        var a: UInt64 = 0
        var b: UInt64 = 1
        for _ in 2...n {
            let (sum, overflow) = a.addingReportingOverflow(b)
            if overflow { return nil }
            a = b
            b = sum
        }
        return b
    }
}
