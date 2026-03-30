//! Criterion benchmarks: `f64` vs `rust_decimal::Decimal` on workloads similar to
//! position notional, fee-style divisions, and many-tick accumulation.
//!
//! Deterministic drift coverage: `src/numeric_representation.rs` (unit test).
//!
//! Run from `agents/backend`:
//! `cargo bench -p quant --bench trading_numeric_compare`

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

const LOOP_ITERS: usize = 512;

fn f64_notional_loop(qty: f64, px: f64) -> f64 {
    let mut acc = 0.0;
    for _ in 0..LOOP_ITERS {
        acc += black_box(qty) * black_box(px);
    }
    acc
}

fn decimal_notional_loop(qty: Decimal, px: Decimal) -> Decimal {
    let mut acc = Decimal::ZERO;
    for _ in 0..LOOP_ITERS {
        acc += black_box(qty) * black_box(px);
    }
    acc
}

/// Fee / allocation-style: multiply then divide by a divisor (e.g. per-share fee scale).
fn f64_div_chain(qty: f64, px: f64, fee_rate: f64) -> f64 {
    let mut acc = 0.0;
    for _ in 0..LOOP_ITERS {
        let notional = black_box(qty) * black_box(px);
        acc += notional / black_box(fee_rate);
    }
    acc
}

fn decimal_div_chain(qty: Decimal, px: Decimal, fee_rate: Decimal) -> Decimal {
    let mut acc = Decimal::ZERO;
    for _ in 0..LOOP_ITERS {
        let notional = black_box(qty) * black_box(px);
        acc += notional / black_box(fee_rate);
    }
    acc
}

/// Many small credits (rounding / quantization stress vs naive `f64` drift).
fn f64_small_sum(base: f64) -> f64 {
    let mut acc = base;
    for i in 0..LOOP_ITERS {
        acc += (i as f64) * 0.01 + 0.001;
    }
    acc
}

fn decimal_small_sum(base: Decimal) -> Decimal {
    let step = dec!(0.01);
    let tiny = dec!(0.001);
    let mut acc = base;
    for i in 0..LOOP_ITERS {
        let i_d = Decimal::from(i as u64);
        acc += i_d * step + tiny;
    }
    acc
}

fn bench_notional(c: &mut Criterion) {
    let qty_f = 142.0_f64;
    let px_f = 518.375_f64;
    let qty_d = dec!(142);
    let px_d = dec!(518.375);

    let mut g = c.benchmark_group("notional_mul_add");
    g.bench_function("f64", |b| b.iter(|| f64_notional_loop(qty_f, px_f)));
    g.bench_function("decimal", |b| b.iter(|| decimal_notional_loop(qty_d, px_d)));
    g.finish();
}

fn bench_div_chain(c: &mut Criterion) {
    let qty_f = 100.5_f64;
    let px_f = 199.99_f64;
    let fee_f = 1.0007_f64;
    let qty_d = dec!(100.5);
    let px_d = dec!(199.99);
    let fee_d = dec!(1.0007);

    let mut g = c.benchmark_group("notional_div_fee");
    g.bench_function("f64", |b| b.iter(|| f64_div_chain(qty_f, px_f, fee_f)));
    g.bench_function("decimal", |b| {
        b.iter(|| decimal_div_chain(qty_d, px_d, fee_d))
    });
    g.finish();
}

fn bench_small_sums(c: &mut Criterion) {
    let base_f = 1_000_000.0_f64;
    let base_d = dec!(1000000);

    let mut g = c.benchmark_group("small_increment_sum");
    g.bench_function("f64", |b| b.iter(|| f64_small_sum(base_f)));
    g.bench_function("decimal", |b| b.iter(|| decimal_small_sum(base_d)));
    g.finish();
}

/// Allocation-style: create values inside the loop (closer to parsing / API boundaries).
fn bench_construct_and_mul(c: &mut Criterion) {
    let mut g = c.benchmark_group("parse_or_from_int_then_mul");
    g.bench_function("f64_from_i32", |b| {
        b.iter_batched(
            || (),
            |_| {
                let mut acc = 0.0_f64;
                for i in 0..LOOP_ITERS {
                    let q = black_box(i as i32) as f64 * 0.01 + 1.0;
                    let p = black_box(i as i32) as f64 * 0.02 + 100.0;
                    acc += q * p;
                }
                acc
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("decimal_from_i32", |b| {
        b.iter_batched(
            || (),
            |_| {
                let mut acc = Decimal::ZERO;
                for i in 0..LOOP_ITERS {
                    let q = Decimal::from(black_box(i as i32)) * dec!(0.01) + Decimal::ONE;
                    let p = Decimal::from(black_box(i as i32)) * dec!(0.02) + dec!(100);
                    acc += q * p;
                }
                acc
            },
            BatchSize::SmallInput,
        )
    });
    g.finish();
}

criterion_group!(
    benches,
    bench_notional,
    bench_div_chain,
    bench_small_sums,
    bench_construct_and_mul,
);
criterion_main!(benches);
