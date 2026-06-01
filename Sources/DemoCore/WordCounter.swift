import Foundation

/// Tally of word occurrences within a body of text.
public struct WordCounter {
    public private(set) var counts: [String: Int] = [:]
    public private(set) var totalWords: Int = 0

    public init() {}

    /// Folds text into the running tally, normalising to lower case and
    /// splitting on any non-alphanumeric character (Unicode aware).
    public mutating func ingest(_ text: String) {
        let lowered = text.lowercased()
        var current = ""
        for scalar in lowered.unicodeScalars {
            if CharacterSet.alphanumerics.contains(scalar) {
                current.unicodeScalars.append(scalar)
            } else if !current.isEmpty {
                record(current)
                current = ""
            }
        }
        if !current.isEmpty { record(current) }
    }

    private mutating func record(_ word: String) {
        counts[word, default: 0] += 1
        totalWords += 1
    }

    /// The `limit` most frequent words, ties broken alphabetically so output
    /// is deterministic across platforms.
    public func top(_ limit: Int) -> [(word: String, count: Int)] {
        counts
            .sorted { lhs, rhs in
                lhs.value == rhs.value ? lhs.key < rhs.key : lhs.value > rhs.value
            }
            .prefix(limit)
            .map { (word: $0.key, count: $0.value) }
    }
}
