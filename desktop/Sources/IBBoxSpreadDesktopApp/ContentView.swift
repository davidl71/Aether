import SwiftUI

struct ContentView: View {
  @State private var contractMultiplier: Double = 100.0
  @State private var syntheticRate: Double = 0.0

  var body: some View {
    VStack(alignment: .leading, spacing: 24) {
      Text("IB Box Spread Desktop")
        .font(.largeTitle)
        .fontWeight(.semibold)

      HStack(spacing: 16) {
        SummaryMetric(title: "Contract Multiplier", value: contractMultiplier.formatted(.number.precision(.fractionLength(0))))
        SummaryMetric(title: "Synthetic Rate", value: syntheticRate.formatted(.number.precision(.fractionLength(2))))
      }

      Text("Hook this SwiftUI view up to shared pricing logic by importing libib_box_spread or exposing a C++ bridge.")
        .foregroundStyle(.secondary)

      Spacer()
    }
    .padding(32)
  }
}

#Preview {
  ContentView()
}

struct SummaryMetric: View {
  let title: String
  let value: String

  var body: some View {
    VStack(alignment: .leading, spacing: 8) {
      Text(title.uppercased())
        .font(.caption)
        .fontWeight(.medium)
        .foregroundStyle(.secondary)
      Text(value)
        .font(.title3)
        .monospaced()
    }
    .padding()
    .frame(maxWidth: .infinity, alignment: .leading)
    .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
  }
}
