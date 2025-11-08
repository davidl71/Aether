import XCTest
@testable import BoxSpreadIPad

final class BoxSpreadIPadTests: XCTestCase {
  @MainActor
  func testApplySortsByAnnualizedReturnThenCredit() {
    let candidates = [
      SpreadRow(symbol: "RUT", days: 24, credit: 950, apr: 12.2),
      SpreadRow(symbol: "SPX", days: 18, credit: 1_100, apr: 12.2),
      SpreadRow(symbol: "NDX", days: 14, credit: 780, apr: 9.5)
    ]
    let viewModel = DashboardViewModel(spreads: [])

    viewModel.apply(snapshot: candidates)

    XCTAssertEqual(viewModel.spreads.map { $0.symbol }, ["SPX", "RUT", "NDX"])
  }

  @MainActor
  func testRefreshPreviewDataProducesSampleRows() {
    let viewModel = DashboardViewModel(spreads: [])

    viewModel.refreshPreviewData()

    XCTAssertFalse(viewModel.spreads.isEmpty)
  }
}
