//! `f64` vs `rust_decimal::Decimal` for trading-style accumulations.
//!
//! Throughput benchmarks live in `benches/trading_numeric_compare.rs` (run from
//! `agents/backend`: `cargo bench -p quant --bench trading_numeric_compare`).

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

const LOOP_ITERS: usize = 512;

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
        acc += Decimal::from(i as u64) * step + tiny;
    }
    acc
}

#[test]
fn small_increment_accumulation_drifts_vs_decimal() {
    let base_f = 1_000_000.0_f64;
    let base_d = dec!(1000000);
    let f = f64_small_sum(base_f);
    let d = decimal_small_sum(base_d);
    let f_as_dec = Decimal::from_f64_retain(f).expect("finite f64");
    let diff = (f_as_dec - d).abs();
    assert_ne!(
        f_as_dec, d,
        "after many fp adds, f64 total should not match exact decimal accumulation (diff={diff})",
    );
}
