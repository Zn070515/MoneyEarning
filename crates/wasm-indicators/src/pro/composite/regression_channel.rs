use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

/// Compute OLS linear regression through `period` points.
/// Returns (slope, intercept) for the line y = a + b*x.
fn ols(x: &[f64], y: &[f64]) -> (f64, f64) {
    let n = x.len() as f64;
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
    let sum_xx: f64 = x.iter().map(|xi| xi * xi).sum();

    let denom = n * sum_xx - sum_x * sum_x;
    if denom.abs() < 1e-12 {
        return (0.0, sum_y / n);
    }

    let b = (n * sum_xy - sum_x * sum_y) / denom;
    let a = (sum_y - b * sum_x) / n;
    (a, b)
}

/// Linear Regression Channel — regression line with upper/lower bands at +/- N standard errors.
///
/// Params: period (default 20), deviation (default 2.0)
///
/// Output: center line (Line) + Band { upper, lower }
pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let deviation = params.get("deviation").copied().unwrap_or(2.0);
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    if n < period {
        return Err(IndError::DataInsufficient(period));
    }

    let x: Vec<f64> = (0..period).map(|v| v as f64).collect();

    let mut center = vec![f64::NAN; n];
    let mut upper = vec![f64::NAN; n];
    let mut lower = vec![f64::NAN; n];

    for i in (period - 1)..n {
        let start = i + 1 - period;
        let y_slice: Vec<f64> = c[start..=i].to_vec();
        let (a, b) = ols(&x, &y_slice);

        // Center = regression endpoint (at x = period - 1)
        let endpoint = a + b * (period as f64 - 1.0);
        center[i] = endpoint;

        // Compute standard error of the regression
        let mut residuals_sq = 0.0;
        for (j, &y_val) in y_slice.iter().enumerate() {
            let y_pred = a + b * (j as f64);
            residuals_sq += (y_val - y_pred) * (y_val - y_pred);
        }
        let std_err = if period > 2 {
            (residuals_sq / (period as f64 - 2.0)).sqrt()
        } else {
            0.0
        };

        upper[i] = endpoint + deviation * std_err;
        lower[i] = endpoint - deviation * std_err;
    }

    Ok(vec![
        IndicatorOutput {
            name: format!("REG_CH_MID({})", period),
            values: Column::F64(center),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("REG_CH_BAND({})", period),
            values: Column::F64(upper.clone()),
            style: OutputStyle::Band {
                upper: Column::F64(upper),
                lower: Column::F64(lower),
            },
        },
    ])
}
