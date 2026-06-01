import XCTest
@testable import DemoCore

final class DemoCoreTests: XCTestCase {
    func testFNV1a() {
        // Known FNV-1a 64-bit vector for the empty input is the offset basis.
        XCTAssertEqual(Hashing.fnv1a64([]), 0xcbf2_9ce4_8422_2325)
        // "a" -> 0xaf63dc4c8601ec8c
        XCTAssertEqual(Hashing.fnv1a64(Array("a".utf8)), 0xaf63_dc4c_8601_ec8c)
    }

    func testWordCounter() {
        var c = WordCounter()
        c.ingest("The quick brown fox. The QUICK fox!")
        XCTAssertEqual(c.totalWords, 7)
        // "fox", "quick" and "the" all occur twice; ties break alphabetically.
        let top = c.top(3)
        XCTAssertEqual(top.map(\.word), ["fox", "quick", "the"])
        XCTAssertEqual(top.first?.count, 2)
    }

    func testFibonacci() {
        XCTAssertEqual(Fibonacci.value(0), 0)
        XCTAssertEqual(Fibonacci.value(10), 55)
        XCTAssertEqual(Fibonacci.value(90), 2_880_067_194_370_816_120)
        XCTAssertNil(Fibonacci.value(-1))
        XCTAssertNil(Fibonacci.value(94)) // overflows UInt64
    }
}
