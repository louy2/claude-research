import ArgumentParser
import DemoCore
import Foundation

struct WCDemo: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "wcdemo",
        abstract: "A small cross-platform demo CLI (built to verify Swift cross-compilation).",
        version: "1.0.0",
        subcommands: [WordFreq.self, Checksum.self, Fib.self, Info.self],
        defaultSubcommand: Info.self
    )
}

struct WordFreq: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "wordfreq",
        abstract: "Count word frequencies across one or more text files."
    )

    @Option(name: .shortAndLong, help: "How many of the most frequent words to show.")
    var top: Int = 10

    @Argument(help: "Text files to analyse.")
    var files: [String]

    func run() throws {
        var counter = WordCounter()
        for path in files {
            let text = try String(contentsOfFile: path, encoding: .utf8)
            counter.ingest(text)
        }
        print("Total words: \(counter.totalWords)")
        print("Unique words: \(counter.counts.count)")
        print("Top \(top):")
        for (rank, entry) in counter.top(top).enumerated() {
            print(String(format: "  %2d. %-20@ %d", rank + 1, entry.word as NSString, entry.count))
        }
    }
}

struct Checksum: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "checksum",
        abstract: "Print the FNV-1a 64-bit hash of each file."
    )

    @Argument(help: "Files to hash.")
    var files: [String]

    func run() throws {
        for path in files {
            let url = URL(fileURLWithPath: path)
            let digest = try Hashing.fnv1a64(ofFileAt: url)
            print("\(digest)  \(path)")
        }
    }
}

struct Fib: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "fib",
        abstract: "Compute the nth Fibonacci number."
    )

    @Argument(help: "Index n (0-based).")
    var n: Int

    func run() throws {
        guard let value = Fibonacci.value(n) else {
            throw ValidationError("fib(\(n)) is out of range for UInt64.")
        }
        print(value)
    }
}

struct Info: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "info",
        abstract: "Print platform and runtime information."
    )

    func run() {
        let info = ProcessInfo.processInfo
        print("wcdemo \(WCDemo.configuration.version)")
        print("Compiled for : \(Platform.description)")
        print("Path sep     : '\(Platform.pathSeparator)'")
        print("Running on   : \(info.operatingSystemVersionString)")
        print("Host name    : \(info.hostName)")
        print("CPU cores    : \(info.activeProcessorCount)")
        let now = ISO8601DateFormatter().string(from: Date())
        print("Timestamp    : \(now)")
    }
}

WCDemo.main()
