pub fn mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().sum::<f64>() / data.len() as f64
}

pub fn std_dev(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let m = mean(data);
    let variance: f64 = data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
    variance.sqrt()
}

pub fn variance(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let m = mean(data);
    data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (data.len() - 1) as f64
}

pub fn skew(data: &[f64]) -> f64 {
    if data.len() < 3 {
        return 0.0;
    }
    let m = mean(data);
    let sd = std_dev(data);
    if sd == 0.0 {
        return 0.0;
    }
    let n = data.len() as f64;
    n / ((n - 1.0) * (n - 2.0)) * data.iter().map(|x| ((x - m) / sd).powi(3)).sum::<f64>()
}

pub fn kurtosis(data: &[f64]) -> f64 {
    if data.len() < 4 {
        return 0.0;
    }
    let m = mean(data);
    let sd = std_dev(data);
    if sd == 0.0 {
        return 0.0;
    }
    let n = data.len() as f64;
    let s4 = data.iter().map(|x| ((x - m) / sd).powi(4)).sum::<f64>();
    (n * (n + 1.0)) / ((n - 1.0) * (n - 2.0) * (n - 3.0)) * s4
        - 3.0 * (n - 1.0).powi(2) / ((n - 2.0) * (n - 3.0))
}

pub fn quantile(data: &[f64], q: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut sorted: Vec<f64> = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = (q * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

pub fn zscore(data: &[f64]) -> Vec<f64> {
    let m = mean(data);
    let sd = std_dev(data);
    if sd == 0.0 {
        return vec![0.0; data.len()];
    }
    data.iter().map(|x| (x - m) / sd).collect()
}

pub fn winsorize(data: &[f64], lower: f64, upper: f64) -> Vec<f64> {
    let lo = quantile(data, lower);
    let hi = quantile(data, upper);
    data.iter().map(|&x| x.clamp(lo, hi)).collect()
}

pub fn pearson_corr(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len());
    if n < 2 {
        return 0.0;
    }
    let mx = mean(&x[..n]);
    let my = mean(&y[..n]);
    let sdx = std_dev(&x[..n]);
    let sdy = std_dev(&y[..n]);
    if sdx == 0.0 || sdy == 0.0 {
        return 0.0;
    }
    let cov: f64 = (0..n).map(|i| (x[i] - mx) * (y[i] - my)).sum::<f64>() / (n - 1) as f64;
    cov / (sdx * sdy)
}

pub fn spearman_rank(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len());
    if n < 2 {
        return 0.0;
    }
    let rank_x = rank(&x[..n]);
    let rank_y = rank(&y[..n]);
    pearson_corr(&rank_x, &rank_y)
}

fn rank(data: &[f64]) -> Vec<f64> {
    let n = data.len();
    let mut indexed: Vec<(usize, f64)> = data.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n && indexed[j].1 == indexed[i].1 {
            j += 1;
        }
        let avg_rank = ((i + j - 1) as f64 / 2.0) + 1.0;
        for k in i..j {
            ranks[indexed[k].0] = avg_rank;
        }
        i = j;
    }
    ranks
}

pub fn covariance_matrix(data: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let m = data.len();
    if m == 0 {
        return vec![];
    }
    let means: Vec<f64> = data.iter().map(|row| mean(row)).collect();
    let n = data[0].len();
    let mut cov = vec![vec![0.0; m]; m];
    for i in 0..m {
        for j in 0..m {
            let c: f64 = (0..n).map(|k| (data[i][k] - means[i]) * (data[j][k] - means[j])).sum::<f64>() / (n - 1) as f64;
            cov[i][j] = c;
        }
    }
    cov
}

pub fn correlation_matrix(data: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let cov = covariance_matrix(data);
    let m = cov.len();
    let mut corr = vec![vec![0.0; m]; m];
    for i in 0..m {
        for j in 0..m {
            let denom = (cov[i][i] * cov[j][j]).sqrt();
            corr[i][j] = if denom == 0.0 { 0.0 } else { cov[i][j] / denom };
        }
    }
    corr
}

pub fn loewdin_orthogonalize(ic_matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = ic_matrix.len();
    if n == 0 {
        return vec![];
    }
    let s_inv_sqrt = matrix_pow_half(ic_matrix, -0.5);
    matrix_multiply(&s_inv_sqrt, ic_matrix)
}

fn matrix_pow_half(a: &[Vec<f64>], power: f64) -> Vec<Vec<f64>> {
    let n = a.len();
    let mut a_clone = a.to_vec();
    let (eigenvals, eigenvecs) = eigenvalue_decomp_sym(&mut a_clone);
    let mut result = vec![vec![0.0; n]; n];
    for i in 0..n {
        let ev_pow = eigenvals[i].powf(power);
        for r in 0..n {
            for c in 0..n {
                result[r][c] += eigenvecs[r][i] * ev_pow * eigenvecs[c][i];
            }
        }
    }
    result
}

fn matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = a.len();
    let mut result = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

fn eigenvalue_decomp_sym(a: &mut [Vec<f64>]) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = a.len();
    let mut v = vec![vec![0.0; n]; n];
    for i in 0..n {
        v[i][i] = 1.0;
    }
    let mut d: Vec<f64> = (0..n).map(|i| a[i][i]).collect();
    let _e: Vec<f64> = (0..n).map(|_| 0.0).collect();

    let max_iter = 50;
    for _iter in 0..max_iter {
        let mut sm = 0.0;
        for i in 0..n {
            for j in (i + 1)..n {
                sm += a[i][j].abs();
            }
        }
        if sm < 1e-12 {
            break;
        }
        for p in 0..(n - 1) {
            for q in (p + 1)..n {
                if (d[q] - d[p]).abs() > 1e-12 {
                    let theta = 0.5 * (2.0 * a[p][q]).atan2(d[p] - d[q]);
                    let mut t = 1.0 / (theta.abs() + (1.0 + theta * theta).sqrt());
                    if theta < 0.0 {
                        t = -t;
                    }
                    let c = 1.0 / (1.0 + t * t).sqrt();
                    let s = t * c;
                    let tau = s / (1.0 + c);
                    let h = t * a[p][q];
                    d[p] -= h;
                    d[q] += h;
                    a[p][q] = 0.0;
                    for j in 0..p {
                        let g = a[j][p];
                        let h = a[j][q];
                        a[j][p] = g - s * (h + g * tau);
                        a[j][q] = h + s * (g - h * tau);
                    }
                    for j in (p + 1)..q {
                        let g = a[p][j];
                        let h = a[j][q];
                        a[p][j] = g - s * (h + g * tau);
                        a[j][q] = h + s * (g - h * tau);
                    }
                    for j in (q + 1)..n {
                        let g = a[p][j];
                        let h = a[q][j];
                        a[p][j] = g - s * (h + g * tau);
                        a[q][j] = h + s * (g - h * tau);
                    }
                    for j in 0..n {
                        let g = v[j][p];
                        let h = v[j][q];
                        v[j][p] = g - s * (h + g * tau);
                        v[j][q] = h + s * (g - h * tau);
                    }
                }
            }
        }
    }
    (d, v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        assert!((mean(&[1.0, 2.0, 3.0]) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_std_dev() {
        let s = std_dev(&[1.0, 2.0, 3.0]);
        assert!((s - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_pearson_corr_perfect() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        assert!((pearson_corr(&x, &y) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_spearman_rank() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        assert!((spearman_rank(&x, &y) + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_zscore() {
        let z = zscore(&[1.0, 2.0, 3.0]);
        assert!((z[0] + 1.0).abs() < 1e-10);
        assert!((z[2] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_winsorize() {
        let w = winsorize(&[1.0, 2.0, 3.0, 4.0, 100.0], 0.1, 0.9);
        assert!(w[4] < 100.0);
    }
}
