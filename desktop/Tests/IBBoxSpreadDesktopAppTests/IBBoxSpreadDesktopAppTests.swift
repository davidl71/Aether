import XCTest
@testable import IBBoxSpreadDesktopApp

final class IBBoxSpreadDesktopAppTests: XCTestCase {
  @MainActor
  func testSummaryMetricStoresTitleAndValue() {
    let metric = SummaryMetric(title: "Synthetic Rate", value: "0.25")

    XCTAssertEqual(metric.title, "Synthetic Rate")
    XCTAssertEqual(metric.value, "0.25")
  }
}
