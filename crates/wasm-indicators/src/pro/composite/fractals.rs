use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

/// Bill Williams Fractals — detects up and down fractals (5-bar patterns by default).
///
/// Params: period (default 2) — number of bars on each side of the potential fractal.
///
/// Output: up_fractal(Dots) + down_fractal(Dots)
pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(2.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let n = h.len();

    let min_required = 2 * period + 1;
    if n < min_required {
        return Err(IndError::DataInsufficient(min_required));
    }

    let mut up_fractal = vec![f64::NAN; n];
    let mut down_fractal = vec![f64::NAN; n];

    for i in period..(n - period) {
        // Up fractal: high[i] is the highest among [i-period .. i+period]
        let mut is_up = true;
        let center = h[i];
        for j in i - period..=i + period {
            if j == i {
                continue;
            }
            if h[j] >= center {
                is_up = false;
                break;
            }
        }
        if is_up {
            up_fractal[i] = center;
        }

        // Down fractal: low[i] is the lowest among [i-period .. i+period]
        let mut is_down = true;
        let center = l[i];
        for j in i - period..=i + period {
            if j == i {
                continue;
            }
            if l[j] <= center {
                is_down = false;
                break;
            }
        }
        if is_down {
            down_fractal[i] = center;
        }
    }

    Ok(vec![
        IndicatorOutput {
            name: "FRACTAL_UP".to_string(),
            values: Column::F64(up_fractal),
            style: OutputStyle::Dots,
        },
        IndicatorOutput {
            name: "FRACTAL_DOWN".to_string(),
            values: Column::F64(down_fractal),
            style: OutputStyle::Dots,
        },
    ])
}
