import { useEffect, useRef, useState } from 'react';
import { createChart, ColorType, IChartApi, ISeriesApi } from 'lightweight-charts';
import type { Timeframe } from '../types/chart';
import type { CandlestickData } from '../types/chart';

interface CandlestickChartProps {
  symbol: string;
  data: CandlestickData[];
  timeframe?: Timeframe;
  height?: number;
  onTimeframeChange?: (timeframe: Timeframe) => void;
}

/**
 * Candlestick chart component using TradingView Lightweight Charts
 * Displays OHLC price data with volume overlay
 */
export function CandlestickChart({
  symbol,
  data,
  timeframe = '1D',
  height = 400,
  onTimeframeChange
}: CandlestickChartProps) {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRef = useRef<ISeriesApi<'Candlestick'> | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (!chartContainerRef.current) return;

    // Create chart
    const chart = createChart(chartContainerRef.current, {
      layout: {
        background: { type: ColorType.Solid, color: '#1a1a1a' },
        textColor: '#d1d5db'
      },
      grid: {
        vertLines: { color: '#2a2a2a' },
        horzLines: { color: '#2a2a2a' }
      },
      width: chartContainerRef.current.clientWidth,
      height: height,
      timeScale: {
        timeVisible: true,
        secondsVisible: false
      }
    });

    chartRef.current = chart;

    // Create candlestick series
    const candlestickSeries = chart.addCandlestickSeries({
      upColor: '#26a69a',
      downColor: '#ef5350',
      borderVisible: false,
      wickUpColor: '#26a69a',
      wickDownColor: '#ef5350'
    });

    seriesRef.current = candlestickSeries;

    // Handle resize
    const handleResize = () => {
      if (chartContainerRef.current && chartRef.current) {
        chartRef.current.applyOptions({
          width: chartContainerRef.current.clientWidth
        });
      }
    };

    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      chart.remove();
    };
  }, [height]);

  // Update data when it changes
  useEffect(() => {
    if (seriesRef.current && data.length > 0) {
      setIsLoading(true);
      // Convert data format if needed
      // Lightweight Charts expects time as number (Unix timestamp in seconds) or string (YYYY-MM-DD)
      const formattedData = data.map((candle) => ({
        time: candle.time,
        open: candle.open,
        high: candle.high,
        low: candle.low,
        close: candle.close
      }));

      seriesRef.current.setData(formattedData);
      setIsLoading(false);
    }
  }, [data]);

  // Update chart title when symbol changes
  useEffect(() => {
    if (chartRef.current) {
      chartRef.current.applyOptions({
        layout: {
          background: { type: ColorType.Solid, color: '#1a1a1a' },
          textColor: '#d1d5db'
        }
      });
    }
  }, [symbol]);

  return (
    <div style={{ position: 'relative', width: '100%' }}>
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: '8px',
          padding: '0 8px'
        }}
      >
        <h4 style={{ margin: 0, color: '#d1d5db' }}>{symbol} - {timeframe}</h4>
        {onTimeframeChange && (
          <div style={{ display: 'flex', gap: '4px' }}>
            {(['1D', '1W', '1M', '3M', '1Y'] as Timeframe[]).map((tf) => (
              <button
                key={tf}
                type="button"
                onClick={() => onTimeframeChange(tf)}
                style={{
                  padding: '4px 8px',
                  fontSize: '12px',
                  backgroundColor: timeframe === tf ? '#3b82f6' : '#374151',
                  color: '#fff',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: 'pointer'
                }}
              >
                {tf}
              </button>
            ))}
          </div>
        )}
      </div>
      {isLoading && (
        <div
          style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            color: '#9ca3af',
            zIndex: 10
          }}
        >
          Loading chart data...
        </div>
      )}
      <div ref={chartContainerRef} style={{ width: '100%', height: `${height}px` }} />
      {data.length === 0 && (
        <div
          style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            color: '#9ca3af',
            zIndex: 10
          }}
        >
          No chart data available
        </div>
      )}
    </div>
  );
}
