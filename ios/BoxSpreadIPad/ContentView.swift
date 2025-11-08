import SwiftUI

struct SpreadRow: Identifiable, Equatable {
  let id: UUID
  let symbol: String
  let days: Int
  let credit: Double
  let apr: Double

  init(id: UUID = UUID(), symbol: String, days: Int, credit: Double, apr: Double) {
    self.id = id
    self.symbol = symbol
    self.days = days
    self.credit = credit
    self.apr = apr
  }
}

@MainActor
final class DashboardViewModel: ObservableObject {
  @Published private(set) var spreads: [SpreadRow]

  init(spreads: [SpreadRow]) {
    self.spreads = spreads
  }

  func apply(snapshot: [SpreadRow]) {
    spreads = snapshot.sorted { lhs, rhs in
      if abs(lhs.apr - rhs.apr) < 0.0001 {
        return lhs.credit > rhs.credit
      }
      return lhs.apr > rhs.apr
    }
  }

  func refreshPreviewData() {
    apply(snapshot: DashboardViewModel.sampleSpreads.shuffled())
  }

  static let preview: DashboardViewModel = {
    let model = DashboardViewModel(spreads: DashboardViewModel.sampleSpreads)
    model.apply(snapshot: sampleSpreads)
    return model
  }()

  private static let sampleSpreads: [SpreadRow] = [
    SpreadRow(symbol: "SPX", days: 30, credit: 1_250.0, apr: 18.3),
    SpreadRow(symbol: "RUT", days: 21, credit: 980.0, apr: 16.1),
    SpreadRow(symbol: "NDX", days: 14, credit: 750.5, apr: 12.4),
    SpreadRow(symbol: "SPXW", days: 7, credit: 420.0, apr: 9.9)
  ]
}

struct ContentView: View {
  @StateObject private var viewModel: DashboardViewModel

  init(viewModel: DashboardViewModel) {
    _viewModel = StateObject(wrappedValue: viewModel)
  }

  var body: some View {
    NavigationStack {
      VStack(alignment: .leading, spacing: 0) {
        summaryHeader
          .padding(.horizontal)
          .padding(.top)
        List {
          Section(header: tableHeader) {
            ForEach(viewModel.spreads) { row in
              SpreadRowView(row: row)
            }
          }
        }
        .listStyle(.insetGrouped)
      }
      .navigationTitle("Box Spread Dashboard")
      .toolbar {
        Button("Refresh") {
          viewModel.refreshPreviewData()
        }
        .accessibilityIdentifier("refresh-button")
      }
    }
  }

  private var summaryHeader: some View {
    VStack(alignment: .leading, spacing: 8) {
      Text("Open Interest Snapshot")
        .font(.headline)
      Text("Sorted by annualized return to highlight the most compelling box spreads available on the shared engine.")
        .font(.subheadline)
        .foregroundStyle(.secondary)
    }
  }

  private var tableHeader: some View {
    HStack {
      Text("Symbol")
        .font(.footnote.weight(.semibold))
        .frame(maxWidth: .infinity, alignment: .leading)
      Text("Days")
        .font(.footnote.weight(.semibold))
        .frame(width: 60, alignment: .trailing)
      Text("Credit")
        .font(.footnote.weight(.semibold))
        .frame(width: 80, alignment: .trailing)
      Text("APR")
        .font(.footnote.weight(.semibold))
        .frame(width: 80, alignment: .trailing)
    }
    .textCase(nil)
  }
}

private struct SpreadRowView: View {
  let row: SpreadRow

  var body: some View {
    HStack {
      Text(row.symbol)
        .frame(maxWidth: .infinity, alignment: .leading)
      Text("\(row.days)")
        .frame(width: 60, alignment: .trailing)
      Text(row.credit as NSNumber, formatter: Self.currencyFormatter)
        .frame(width: 80, alignment: .trailing)
      Text(String(format: "%.1f%%", row.apr))
        .frame(width: 80, alignment: .trailing)
    }
    .font(.body.monospacedDigit())
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("\(row.symbol) spread")
    .accessibilityValue("\(row.days) DTE, credit \(Self.currencyFormatter.string(from: row.credit as NSNumber) ?? "$0"), APR \(String(format: "%.1f%%", row.apr))")
  }

  private static let currencyFormatter: NumberFormatter = {
    let formatter = NumberFormatter()
    formatter.numberStyle = .currency
    formatter.currencyCode = "USD"
    formatter.maximumFractionDigits = 2
    return formatter
  }()
}

#Preview {
  ContentView(viewModel: .preview)
}
