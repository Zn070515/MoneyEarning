use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

/// Floor Pivot Points — classic floor trader pivot system (S3,S2,S1,P,R1,R2,R3).
///
/// Params: period (default 1) — how many bars back to find high/low/close reference.
///
/// For period=1, uses previous bar's HLC. For period>1, uses max high, min low,
/// and the most recent close from the preceding `period` bars.
///
/// Output: 7 lines — P, R1, R2, R3, S1, S2, S3
pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(1.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = h.len();

    let min_required = period + 1;
    if n < min_required {
        return Err(IndError::DataInsufficient(min_required));
    }

    let mut p_line = vec![f64::NAN; n];
    let mut r1 = vec![f64::NAN; n];
    let mut r2 = vec![f64::NAN; n];
    let mut r3 = vec![f64::NAN; n];
    let mut s1 = vec![f64::NAN; n];
    let mut s2 = vec![f64::NAN; n];
    let mut s3 = vec![f64::NAN; n];

    for i in period..n {
        // Highest high, lowest low, and close over the preceding `period` bars
        let start = i - period;
        let end = i - 1;
        let h_prev = h[start..=end].iter().cloned().fold(f64::MIN, f64::max);
        let l_prev = l[start..=end].iter().cloned().fold(f64::MAX, f64::min);
        let c_prev = c[end]; // most recent close

        let pp = (h_prev + l_prev + c_prev) / 3.0;
        p_line[i] = pp;
        r1[i] = 2.0 * pp - l_prev;
        r2[i] = pp + (h_prev - l_prev);
        r3[i] = h_prev + 2.0 * (pp - l_prev);
        s1[i] = 2.0 * pp - h_prev;
        s2[i] = pp - (h_prev - l_prev);
        s3[i] = l_prev - 2.0 * (h_prev - pp);
    }

    Ok(vec![
        IndicatorOutput {
            name: format!("PIVOT_P({})", period),
            values: Column::F64(p_line),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("PIVOT_R1({})", period),
            values: Column::F64(r1),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("PIVOT_R2({})", period),
            values: Column::F64(r2),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("PIVOT_R3({})", period),
            values: Column::F64(r3),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("PIVOT_S1({})", period),
            values: Column::F64(s1),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("PIVOT_S2({})", period),
            values: Column::F64(s2),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("PIVOT_S3({})", period),
            values: Column::F64(s3),
            style: OutputStyle::Line,
        },
    ])
}
