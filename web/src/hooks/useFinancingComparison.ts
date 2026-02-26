import { useEffect, useState, useMemo } from 'react';
import { useTreasuryYields } from './useTreasuryYields';
import type { BoxSpreadScenario, TreasuryBenchmark } from '../types';

export interface ComparisonRow {
  tenorDays: number;
  tenorLabel: string;
  boxSpread: {
    grossRate: number;
    afterTaxRate: number;
    returnOnCapital: number;
    marginLeverage: number;
    capitalEfficiency: number;
    liquidityScore: number;
  } | null;
  treasury: {
    grossRate: number;
    afterTaxRate: number;
    maturity: string;
    type: string;
  } | null;
  spreadBpsPretax: number | null;
  spreadBpsAftertax: number | null;
  winner: 'box_spread' | 'treasury' | 'tie' | null;
}

export interface FinancingComparisonResult {
  rows: ComparisonRow[];
  boxSpreadWins: number;
  treasuryWins: number;
  ties: number;
  taxConfig: {
    federalRate: number;
    section1256Blended: number;
    treasuryTaxRate: number;
  };
}

interface TaxConfig {
  federalRate: number;
  stateRate: number;
  ltcgRate: number;
  stcgRate: number;
  stateExemptTreasuries: boolean;
}

const DEFAULT_TAX: TaxConfig = {
  federalRate: 0.37,
  stateRate: 0.05,
  ltcgRate: 0.20,
  stcgRate: 0.37,
  stateExemptTreasuries: true,
};

function section1256BlendedRate(tax: TaxConfig): number {
  return 0.60 * tax.ltcgRate + 0.40 * tax.stcgRate;
}

function treasuryTaxRate(tax: TaxConfig): number {
  return tax.stateExemptTreasuries ? tax.federalRate : tax.federalRate + tax.stateRate;
}

function tenorLabel(dte: number): string {
  const labels: Record<number, string> = {
    1: 'O/N', 30: '1M', 60: '2M', 90: '3M', 180: '6M',
    365: '1Y', 730: '2Y', 1095: '3Y', 1825: '5Y', 3650: '10Y',
  };
  for (const [key, label] of Object.entries(labels)) {
    if (Math.abs(dte - Number(key)) <= 5) return label;
  }
  if (dte < 30) return `${dte}d`;
  const months = Math.round(dte / 30);
  if (months <= 12) return `${months}M`;
  return `${(dte / 365).toFixed(1)}Y`;
}

function findClosest(dte: number, benchmarks: TreasuryBenchmark[], tolerance = 30): TreasuryBenchmark | null {
  let best: TreasuryBenchmark | null = null;
  let bestDiff = Infinity;
  for (const b of benchmarks) {
    const diff = Math.abs(b.maturityDays - dte);
    if (diff <= tolerance && diff < bestDiff) {
      bestDiff = diff;
      best = b;
    }
  }
  return best;
}

export function useFinancingComparison(
  scenarios: BoxSpreadScenario[],
  taxConfig: Partial<TaxConfig> = {},
  marginType: 'reg_t' | 'portfolio' = 'reg_t',
): { data: FinancingComparisonResult | null; isLoading: boolean } {
  const tax: TaxConfig = { ...DEFAULT_TAX, ...taxConfig };
  const { data: treasuryData, isLoading: treasuryLoading } = useTreasuryYields();

  const data = useMemo<FinancingComparisonResult | null>(() => {
    const european = scenarios.filter(s => s.option_style === 'European');
    if (european.length === 0) return null;

    const blended = section1256BlendedRate(tax);
    const tsyTax = treasuryTaxRate(tax);

    // Group by DTE (closest expiration)
    const byDte = new Map<number, BoxSpreadScenario[]>();
    for (const s of european) {
      const dte = s.days_to_expiry ?? 0;
      if (dte <= 0) continue;
      if (!byDte.has(dte)) byDte.set(dte, []);
      byDte.get(dte)!.push(s);
    }

    const benchmarks = treasuryData?.benchmarks ?? [];
    const rows: ComparisonRow[] = [];

    for (const [dte, group] of Array.from(byDte.entries()).sort((a, b) => a[0] - b[0])) {
      const avgRate = group.reduce((sum, s) => sum + s.annualized_return, 0) / group.length;
      const avgLiquidity = group.reduce((sum, s) => sum + (s.fill_probability ?? 0), 0) / group.length;

      const marginMultiplier = marginType === 'portfolio' ? 0.25 : 1.0;
      const leverage = 1.0 / marginMultiplier;
      const afterTaxBs = avgRate * (1.0 - blended);
      const roc = afterTaxBs * leverage;

      const bench = findClosest(dte, benchmarks);
      let tsyGross: number | null = null;
      let tsyAfterTax: number | null = null;
      if (bench) {
        tsyGross = bench.yield;
        tsyAfterTax = bench.yield * (1.0 - tsyTax);
      }

      const spreadPre = tsyGross !== null ? (avgRate - tsyGross) * 100 : null;
      const spreadPost = tsyAfterTax !== null ? (afterTaxBs - tsyAfterTax) * 100 : null;

      let winner: 'box_spread' | 'treasury' | 'tie' | null = null;
      if (spreadPost !== null) {
        if (spreadPost > 5) winner = 'box_spread';
        else if (spreadPost < -5) winner = 'treasury';
        else winner = 'tie';
      }

      rows.push({
        tenorDays: dte,
        tenorLabel: tenorLabel(dte),
        boxSpread: {
          grossRate: avgRate,
          afterTaxRate: afterTaxBs,
          returnOnCapital: roc,
          marginLeverage: leverage,
          capitalEfficiency: leverage * 100,
          liquidityScore: avgLiquidity,
        },
        treasury: bench ? {
          grossRate: bench.yield,
          afterTaxRate: tsyAfterTax!,
          maturity: bench.maturity,
          type: bench.type,
        } : null,
        spreadBpsPretax: spreadPre,
        spreadBpsAftertax: spreadPost,
        winner,
      });
    }

    const boxWins = rows.filter(r => r.winner === 'box_spread').length;
    const tsyWins = rows.filter(r => r.winner === 'treasury').length;

    return {
      rows,
      boxSpreadWins: boxWins,
      treasuryWins: tsyWins,
      ties: rows.length - boxWins - tsyWins,
      taxConfig: {
        federalRate: tax.federalRate,
        section1256Blended: blended,
        treasuryTaxRate: tsyTax,
      },
    };
  }, [scenarios, treasuryData, tax.federalRate, tax.stateRate, tax.ltcgRate, tax.stcgRate, tax.stateExemptTreasuries, marginType]);

  return { data, isLoading: treasuryLoading };
}
