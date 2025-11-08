import type { Candle } from '../types/snapshot';

interface SparklineProps {
  candle: Candle;
}

export function Sparkline({ candle }: SparklineProps) {
  const values = [candle.open, candle.high, candle.low, candle.close, candle.entry];
  if (values.some((value) => Number.isNaN(value))) {
    return <span className="sparkline sparkline--empty" aria-hidden="true">—</span>;
  }

  const data = [candle.open, candle.high, candle.low, candle.close];
  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;

  const points = data.map((value, idx) => {
    const x = (idx / (data.length - 1)) * 100;
    const y = 100 - ((value - min) / range) * 100;
    return `${x},${y}`;
  }).join(' ');

  const closeRank = 100 - ((candle.close - min) / range) * 100;
  const entryRank = 100 - ((candle.entry - min) / range) * 100;

  return (
    <span className="sparkline" aria-hidden="true">
      <svg viewBox="0 0 100 100" preserveAspectRatio="none" role="presentation">
        <polyline points={points} className="sparkline__line" />
        <line x1="0" x2="100" y1={closeRank} y2={closeRank} className="sparkline__close" />
        <line x1="0" x2="100" y1={entryRank} y2={entryRank} className="sparkline__entry" />
      </svg>
    </span>
  );
}
