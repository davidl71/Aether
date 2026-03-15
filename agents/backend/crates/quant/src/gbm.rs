//! Minimal Geometric Brownian Motion and Euler–Maruyama paths (no RustQuant/polars).

use rand::Rng;

/// Euler–Maruyama paths for GBM: dS = mu*S*dt + sigma*S*dW.
/// Returns paths[sample_idx][time_step]; paths have length (steps + 1).
pub fn euler_maruyama(
    rng: &mut impl Rng,
    s0: f64,
    mu: f64,
    sigma: f64,
    t: f64,
    steps: usize,
    n_paths: usize,
) -> Vec<Vec<f64>> {
    let dt = t / steps as f64;
    let sqrt_dt = dt.sqrt();
    let mut paths = Vec::with_capacity(n_paths);
    for _ in 0..n_paths {
        let mut path = Vec::with_capacity(steps + 1);
        path.push(s0);
        let mut s = s0;
        for _ in 0..steps {
            let z = box_muller(rng);
            s = s * (1.0 + mu * dt + sigma * sqrt_dt * z);
            path.push(s);
        }
        paths.push(path);
    }
    paths
}

/// Box–Muller for one standard normal sample.
fn box_muller(rng: &mut impl Rng) -> f64 {
    let u: f64 = rng.gen_range(1e-10..=1.0);
    let v: f64 = rng.gen();
    (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;

    #[test]
    fn test_gbm_paths_shape() {
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let paths = euler_maruyama(&mut rng, 100.0, 0.05, 0.2, 1.0, 100, 10);
        assert_eq!(paths.len(), 10);
        assert_eq!(paths[0].len(), 101);
        assert!((paths[0][0] - 100.0).abs() < 1e-10);
    }
}
