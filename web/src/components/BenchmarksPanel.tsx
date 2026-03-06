import { useRiskFreeRateBenchmarks } from '../hooks/useRiskFreeRateBenchmarks';

const SOURCE_LABEL = 'Source: FRED (St. Louis Fed) via Risk-Free Rate service';

function formatTs(ts: string | null | undefined): string {
  if (!ts) return '—';
  return ts.replace('T', ' ').split('.')[0] ?? '—';
}

export function BenchmarksPanel() {
  const { sofr, treasury, isLoading, error } = useRiskFreeRateBenchmarks(60_000);

  if (isLoading && !sofr && !treasury) {
    return (
      <div className="panel">
        <div className="panel__header"><h3>Benchmarks (SOFR &amp; Treasury)</h3></div>
        <div style={{ padding: '20px', textAlign: 'center', color: '#9ca3af' }}>
          Loading benchmark data…
        </div>
      </div>
    );
  }

  if (error && !sofr && !treasury) {
    return (
      <div className="panel">
        <div className="panel__header"><h3>Benchmarks (SOFR &amp; Treasury)</h3></div>
        <div style={{ padding: '20px', color: '#f87171' }}>
          {error}
        </div>
      </div>
    );
  }

  const sofrOvernight = sofr?.overnight;
  const sofrTermRates = sofr?.term_rates ?? [];
  const treasuryRates = treasury?.rates ?? [];

  return (
    <div className="panel">
      <div className="panel__header">
        <h3>Benchmarks (SOFR &amp; Treasury)</h3>
        <p style={{ fontSize: '0.85em', color: '#9ca3af', marginTop: '4px' }}>{SOURCE_LABEL}</p>
      </div>
      <div className="table-wrapper">
        <table className="data-table" aria-label="Benchmark rates">
          <thead>
            <tr>
              <th>Series</th>
              <th>Tenor</th>
              <th>Rate %</th>
              <th>Updated</th>
            </tr>
          </thead>
          <tbody>
            {sofrOvernight && sofrOvernight.rate != null && (
              <tr>
                <td>SOFR</td>
                <td>Overnight</td>
                <td>{sofrOvernight.rate.toFixed(2)}</td>
                <td>{formatTs(sofrOvernight.timestamp)}</td>
              </tr>
            )}
            {sofrTermRates.map((r) => (
              <tr key={r.tenor}>
                <td>SOFR</td>
                <td>{r.tenor}</td>
                <td>{r.rate.toFixed(2)}</td>
                <td>{formatTs(r.timestamp)}</td>
              </tr>
            ))}
            {treasuryRates.map((r) => (
              <tr key={r.tenor}>
                <td>Treasury</td>
                <td>{r.tenor}</td>
                <td>{r.rate.toFixed(2)}</td>
                <td>{formatTs(r.timestamp)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {error && (sofr || treasury) && (
        <div style={{ padding: '8px 16px', fontSize: '0.85em', color: '#fbbf24' }}>
          Partial data: {error}
        </div>
      )}
    </div>
  );
}
