import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor, within } from '@testing-library/react';
import App from '../App';

const snapshotResponse = {
  generated_at: '2025-11-07T16:25:00Z',
  mode: 'DRY-RUN',
  strategy: 'RUNNING',
  account_id: 'DU123456',
  metrics: {
    net_liq: 100523.12,
    buying_power: 80412.44,
    excess_liquidity: 25500.22,
    margin_requirement: 15234.56,
    commissions: 127.89,
    portal_ok: true,
    tws_ok: true,
    orats_ok: true,
    questdb_ok: true
  },
  symbols: [
    {
      symbol: 'SPY',
      last: 509.26,
      bid: 509.12,
      ask: 509.34,
      spread: 0.22,
      roi: 11.45,
      maker_count: 12,
      taker_count: 4,
      volume: 132,
      candle: {
        open: 508.6,
        high: 510.12,
        low: 507.9,
        close: 509.26,
        volume: 112.5,
        entry: 508.1,
        updated: '2025-11-07T16:24:45Z'
      },
      option_chains: []
    }
  ],
  positions: [],
  historic: [],
  orders: [],
  alerts: []
};

const scenarioResponse = {
  as_of: '2025-11-07T13:00:00Z',
  underlying: 'XSP',
  scenarios: [
    {
      width: 1,
      put_bid: 1,
      call_ask: 1,
      synthetic_bid: 99,
      synthetic_ask: 101,
      mid_price: 100,
      annualized_return: 10,
      fill_probability: 50
    }
  ]
};

beforeEach(() => {
  globalThis.fetch = vi.fn((input: RequestInfo | URL) => {
    let url = '';
    if (typeof input === 'string') {
      url = input;
    } else if (input instanceof URL) {
      url = input.href;
    } else if (input instanceof Request) {
      url = input.url;
    }

    if (url.includes('snapshot')) {
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve(snapshotResponse)
      } as unknown as Response);
    }

    return Promise.resolve({
      ok: true,
      json: () => Promise.resolve(scenarioResponse)
    } as unknown as Response);
  });
});

afterEach(() => {
  vi.resetAllMocks();
});

describe('App', () => {
  it('renders snapshot dashboard and scenario table', async () => {
    render(<App />);

    await waitFor(() => expect(screen.getByText('SPY')).toBeInTheDocument());

    const dashboardTable = screen.getByRole('table', { name: /symbol metrics/i });
    expect(within(dashboardTable).getByText('SPY')).toBeInTheDocument();

    const scenarioTable = screen.getByRole('table', { name: /box spread scenarios/i });
    expect(within(scenarioTable).getByText('10.00%')).toBeInTheDocument();
  });
});
