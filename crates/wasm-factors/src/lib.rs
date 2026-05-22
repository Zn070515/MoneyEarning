//! Alpha158 因子库 — 158个A股量价因子，兼容Qlib Alpha158规范
//!
//! 6大类：趋势(30) | 反转(22) | 波动(26) | 成交量(28) | 动量(28) | 流动性(24)
//!
//! 设计原则：声明式因子定义 + 统一计算引擎，避免158个重复函数。
//! DAG依赖解析支持因子间复用（如TR→ATR→ADX）。

use wasm_core::DataFrame;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

mod factors;
pub use factors::*;
mod benches;

// ── Factor Definition ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Trend,
    Reversal,
    Volatility,
    Volume,
    Momentum,
    Liquidity,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Trend => "趋势",
            Category::Reversal => "反转",
            Category::Volatility => "波动",
            Category::Volume => "成交量",
            Category::Momentum => "动量",
            Category::Liquidity => "流动性",
        }
    }
}

/// Factor metadata — definition without computation
#[derive(Debug, Clone)]
pub struct FactorMeta {
    pub name: String,
    pub name_cn: String,
    pub category: Category,
    pub formula: Formula,
    pub window: usize,
}

/// Core formula types — covers all 158 Alpha158 factors
#[derive(Debug, Clone, Copy)]
pub enum Formula {
    /// (close - sma(close, N)) / sma(close, N)  → MA偏离度
    MaDev(usize),
    /// (close - sma(close, N)) / std(close, N) → 标准化偏离
    MaZScore(usize),
    /// (ema(close, fast) - ema(close, slow)) / close → MACD相关
    MacdDif,
    MacdDea,
    MacdHist,
    /// Triple-smoothed EMA of returns
    Trix(usize),
    /// True Range
    TrueRange,
    /// ATR(N) = sma(TR, N)
    Atr(usize),
    /// +DI / -DI / ADX
    Pdi(usize),
    Mdi(usize),
    Adx(usize),
    /// Linear regression slope over N
    Plrc(usize),
    /// Rolling highest/lowest
    Hh(usize),
    Ll(usize),
    /// (close - ll(N)) / (hh(N) - ll(N)) → 价格通道位置
    PriceChannel(usize),
    /// RSI(N)
    Rsi(usize),
    /// Williams %R(N)
    Wr(usize),
    /// Bollinger Bands
    BbUpper(usize),
    BbLower(usize),
    BbWidth(usize),
    /// Bias = (close - ma(N)) / ma(N)
    Bias(usize),
    /// CCI(N)
    Cci(usize),
    /// KDJ
    Kdjj,
    Kdjd,
    /// EMA pair ratio
    EmaFastSlow,
    /// Disparity = close / ma(N)
    Disparity(usize),
    /// Rolling standard deviation
    Std(usize),
    /// Rolling volatility = std(daily_returns) * sqrt(N)
    Vol(usize),
    /// Volume ratio = vol / sma(vol, 5) / sma(vol, 20)
    VolRatio,
    /// (high - low) / prev_close
    Amplitude,
    /// Daily volatility ratio
    Dvrat,
    /// Daily average volume over N
    DaVol(usize),
    /// Up/Down volume proportion
    UpVol(usize),
    DnVol(usize),
    /// Higher moments of returns
    VolatilitySkew(usize),
    VolatilityKurt(usize),
    /// Max/min return over N
    MaxRet(usize),
    MinRet(usize),
    /// (high - low) / close
    HighLowRatio,
    /// Price range = (hh(N) - ll(N)) / close
    Range(usize),
    /// Ulcer index
    Ulcer(usize),
    /// Volume MA
    Vma(usize),
    /// Volume standard deviation
    Vstd(usize),
    /// Volume oscillator = (vma(5) - vma(20)) / vma(5)
    Vosc,
    /// On-balance volume
    Obv,
    /// VWAP deviation
    VwapDev(usize),
    /// Price-volume correlation
    PvCorr(usize),
    /// Relative volume
    VolumeRatioFactor,
    /// Volume trend
    VolumeTrend,
    /// Money flow index
    Mfi(usize),
    /// Ease of movement
    Eom,
    /// Force index
    ForceIndex,
    /// Accumulation/Distribution
    Ad,
    /// Chaikin money flow
    Cmf(usize),
    /// Negative/Positive volume index
    Nvi,
    Pvi,
    /// Volume price trend
    VolPriceTrend,
    /// Turnover MA deviation
    TvMa(usize),
    /// Upside/Downside volume
    UpsideVolume(usize),
    DownsideVolume(usize),
    /// Return over N periods (pct_change)
    Ret(usize),
    /// Rolling Sharpe ratio over N
    Sr(usize),
    /// Relative strength = close / sma(close, N)
    Rs(usize),
    /// Distance from N-period high
    HighDist(usize),
    /// Distance from N-period low
    LowDist(usize),
    /// Momentum divergence = ret(N) / ret(N*4)
    MomDiv(usize),
    /// Rate of change
    Roc(usize),
    /// Long-term momentum ratio = ret(60) / ret(20)
    LtmRatio,
    /// Percentile rank of close over N
    PctRank(usize),
    /// Turnover rate deviation
    Turn(usize),
    /// Turnover standard deviation
    TurnStd(usize),
    /// Amihud illiquidity = |ret| / amount
    Amihud(usize),
    /// Dollar volume
    DollarVol(usize),
    /// High-low spread ratio
    Spread(usize),
    /// Volume weighted price deviation
    Vwp(usize),
    /// Volume concentration
    VolConc(usize),
    /// Liquidity ratio
    LiqRatio(usize),
    /// Price impact
    PriceImpact(usize),
    /// Volume breakout
    VolBreak(usize),
    /// Float market cap proxy = amount / turnover
    FloatProxy,
    /// Illiquidity standard deviation
    IlliqStd(usize),
    /// High-low relative amplitude
    HighLow(usize),
    /// Average daily amplitude over N = sma((high-low)/prev_close, N)
    AvgAmplitude(usize),
}

// ── Compute Engine ──

/// Compute a single factor across the entire DataFrame
pub fn compute_factor(df: &DataFrame, formula: Formula, _window: usize) -> Vec<f64> {
    let _o = df.column("open").map(|c| c.to_f64_vec()).unwrap_or_default();
    let h = df.column("high").map(|c| c.to_f64_vec()).unwrap_or_default();
    let l = df.column("low").map(|c| c.to_f64_vec()).unwrap_or_default();
    let c = df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    let v = df.column("volume").map(|c| c.to_f64_vec()).unwrap_or_default();
    let amt = df.column("amount").map(|c| c.to_f64_vec()).unwrap_or_default();
    let to = df.column("turnover").map(|c| c.to_f64_vec()).unwrap_or_default();

    let n = c.len();
    if n == 0 { return vec![]; }

    match formula {
        // ── Trend ──
        Formula::MaDev(w) => {
            let sma_ = sma(&c, w);
            c.iter().enumerate().map(|(i, &ci)| if sma_[i] > 0.0 { ci / sma_[i] - 1.0 } else { 0.0 }).collect()
        }
        Formula::MaZScore(w) => {
            let sma_ = sma(&c, w);
            let std_ = rolling_std(&c, w);
            c.iter().enumerate().map(|(i, &ci)| if std_[i] > 0.0 { (ci - sma_[i]) / std_[i] } else { 0.0 }).collect()
        }
        Formula::MacdDif => {
            let ema12 = ema(&c, 12);
            let ema26 = ema(&c, 26);
            c.iter().enumerate().map(|(i, &ci)| if ci > 0.0 { (ema12[i] - ema26[i]) / ci } else { 0.0 }).collect()
        }
        Formula::MacdDea => {
            let ema12 = ema(&c, 12);
            let ema26 = ema(&c, 26);
            let dif: Vec<f64> = ema12.iter().zip(&ema26).map(|(a, b)| a - b).collect();
            let dea = ema(&dif, 9);
            c.iter().enumerate().map(|(i, &ci)| if ci > 0.0 { dea[i] / ci } else { 0.0 }).collect()
        }
        Formula::MacdHist => {
            let ema12 = ema(&c, 12);
            let ema26 = ema(&c, 26);
            let dif: Vec<f64> = ema12.iter().zip(&ema26).map(|(a, b)| a - b).collect();
            let dea = ema(&dif, 9);
            c.iter().enumerate().map(|(i, &ci)| if ci > 0.0 { (dif[i] - dea[i]) / ci } else { 0.0 }).collect()
        }
        Formula::Trix(w) => {
            let ret = pct_change(&c, 1);
            let e1 = ema(&ret, w);
            let e2 = ema(&e1, w);
            ema(&e2, w)
        }
        Formula::TrueRange => {
            let mut tr = vec![0.0; n];
            for i in 0..n {
                let prev_c = if i > 0 { c[i-1] } else { c[i] };
                tr[i] = (h[i] - l[i]).max((h[i] - prev_c).abs()).max((l[i] - prev_c).abs());
            }
            tr
        }
        Formula::Atr(w) => {
            let tr = compute_factor(df, Formula::TrueRange, 0);
            sma(&tr, w)
        }
        Formula::Pdi(w) => pdi_mdi_adx(&h, &l, &c, w).0,
        Formula::Mdi(w) => pdi_mdi_adx(&h, &l, &c, w).1,
        Formula::Adx(w) => pdi_mdi_adx(&h, &l, &c, w).2,
        Formula::Plrc(w) => {
            let sma_ = sma(&c, w);
            let half = (w as f64 / 2.0).round() as usize;
            c.iter().enumerate().map(|(i, _)| {
                if i < w { return 0.0; }
                let mut num = 0.0;
                let mut den = 0.0;
                for j in 0..w {
                    let x = j as f64 - half as f64;
                    num += x * (c[i-w+1+j] - sma_[i]);
                    den += x * x;
                }
                if den > 0.0 { num / den / c[i].max(0.0001) } else { 0.0 }
            }).collect()
        }
        Formula::Hh(w) => rolling_max(&c, w),
        Formula::Ll(w) => rolling_min(&c, w),
        Formula::PriceChannel(w) => {
            let hh = rolling_max(&c, w);
            let ll = rolling_min(&c, w);
            c.iter().enumerate().map(|(i, &ci)| {
                let den = hh[i] - ll[i];
                if den > 0.0 { (ci - ll[i]) / den } else { 0.5 }
            }).collect()
        }

        // ── Reversal ──
        Formula::Rsi(w) => rsi(&c, w),
        Formula::Wr(w) => {
            let hh = rolling_max(&h, w);
            let ll = rolling_min(&l, w);
            c.iter().enumerate().map(|(i, &ci)| {
                let den = hh[i] - ll[i];
                if den > 0.0 { (hh[i] - ci) / den * -100.0 } else { -50.0 }
            }).collect()
        }
        Formula::BbUpper(w) => {
            let sma_ = sma(&c, w);
            let std_ = rolling_std(&c, w);
            sma_.iter().zip(&std_).map(|(s, d)| s + d * 2.0).collect()
        }
        Formula::BbLower(w) => {
            let sma_ = sma(&c, w);
            let std_ = rolling_std(&c, w);
            sma_.iter().zip(&std_).map(|(s, d)| s - d * 2.0).collect()
        }
        Formula::BbWidth(w) => {
            let sma_ = sma(&c, w);
            let std_ = rolling_std(&c, w);
            sma_.iter().zip(&std_).enumerate().map(|(_i, (s, d))| if *s > 0.0 { 4.0 * d / s } else { 0.0 }).collect()
        }
        Formula::Bias(w) => {
            let sma_ = sma(&c, w);
            c.iter().zip(&sma_).map(|(ci, s)| if *s > 0.0 { (ci - s) / s } else { 0.0 }).collect()
        }
        Formula::Cci(w) => {
            let tp: Vec<f64> = h.iter().zip(&l).zip(&c).map(|((&hi, &li), &ci)| (hi + li + ci) / 3.0).collect();
            let sma_tp = sma(&tp, w);
            let mad: Vec<f64> = tp.iter().enumerate().map(|(i, &_t)| {
                if i < w - 1 { return 0.0; }
                (i + 1 - w..=i).map(|j| (tp[j] - sma_tp[i]).abs()).sum::<f64>() / w as f64
            }).collect();
            tp.iter().zip(&sma_tp).zip(&mad).map(|((&t, &s), &m)| {
                if m > 0.0 { (t - s) / (0.015 * m) } else { 0.0 }
            }).collect()
        }
        Formula::Kdjj => {
            let (k, d) = kdj_values(&h, &l, &c, 9, 3);
            k.iter().zip(&d).map(|(&kv, &dv)| 3.0 * kv - 2.0 * dv).collect()
        }
        Formula::Kdjd => {
            let (_, d) = kdj_values(&h, &l, &c, 9, 3);
            d
        }
        Formula::EmaFastSlow => {
            let ema5 = ema(&c, 5);
            let ema20 = ema(&c, 20);
            ema5.iter().zip(&ema20).map(|(f, s)| if *s > 0.0 { f / s - 1.0 } else { 0.0 }).collect()
        }
        Formula::Disparity(w) => {
            let sma_ = sma(&c, w);
            c.iter().zip(&sma_).map(|(ci, s)| if *s > 0.0 { ci / s } else { 1.0 }).collect()
        }

        // ── Volatility ──
        Formula::Std(w) => rolling_std(&c, w),
        Formula::Vol(w) => {
            let ret = pct_change(&c, 1);
            let std_ = rolling_std(&ret, w);
            std_.iter().map(|s| s * (w as f64).sqrt()).collect()
        }
        Formula::VolRatio => {
            let vol5 = rolling_std(&pct_change(&c, 1), 5);
            let vol20 = rolling_std(&pct_change(&c, 1), 20);
            vol5.iter().zip(&vol20).map(|(v5, v20)| if *v20 > 0.0 { v5 / v20 } else { 1.0 }).collect()
        }
        Formula::Amplitude => {
            c.iter().enumerate().map(|(i, &ci)| {
                let prev = if i > 0 { c[i-1] } else { ci };
                if prev > 0.0 { (h[i] - l[i]) / prev } else { 0.0 }
            }).collect()
        }
        Formula::Dvrat => {
            let ret = pct_change(&c, 1);
            let std_short = rolling_std(&ret, 5);
            let std_long = rolling_std(&ret, 20);
            std_short.iter().zip(&std_long).map(|(s, l)| if *l > 0.0 { s / l } else { 1.0 }).collect()
        }
        Formula::DaVol(w) => sma(&v, w),
        Formula::UpVol(w) => {
            let ret = pct_change(&c, 1);
            let mut result = vec![0.0; n];
            for i in 0..n {
                let start = if i >= w { i - w + 1 } else { 0 };
                let up_sum: f64 = (start..=i).filter(|&j| ret[j] > 0.0).map(|j| v[j]).sum();
                let total: f64 = (start..=i).map(|j| v[j]).sum();
                result[i] = if total > 0.0 { up_sum / total } else { 0.5 };
            }
            result
        }
        Formula::DnVol(w) => {
            let ret = pct_change(&c, 1);
            let mut result = vec![0.0; n];
            for i in 0..n {
                let start = if i >= w { i - w + 1 } else { 0 };
                let dn_sum: f64 = (start..=i).filter(|&j| ret[j] < 0.0).map(|j| v[j]).sum();
                let total: f64 = (start..=i).map(|j| v[j]).sum();
                result[i] = if total > 0.0 { dn_sum / total } else { 0.5 };
            }
            result
        }
        Formula::VolatilitySkew(w) => {
            let ret = pct_change(&c, 1);
            let mut result = vec![0.0; n];
            for i in 0..n {
                if i < w { continue; }
                let slice: Vec<f64> = (i + 1 - w..=i).map(|j| ret[j]).collect();
                let mean = slice.iter().sum::<f64>() / w as f64;
                let std = (slice.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / w as f64).sqrt();
                if std > 0.0 {
                    result[i] = slice.iter().map(|r| ((r - mean) / std).powi(3)).sum::<f64>() / w as f64;
                }
            }
            result
        }
        Formula::VolatilityKurt(w) => {
            let ret = pct_change(&c, 1);
            let mut result = vec![0.0; n];
            for i in 0..n {
                if i < w { continue; }
                let slice: Vec<f64> = (i + 1 - w..=i).map(|j| ret[j]).collect();
                let mean = slice.iter().sum::<f64>() / w as f64;
                let std = (slice.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / w as f64).sqrt();
                if std > 0.0 {
                    result[i] = slice.iter().map(|r| ((r - mean) / std).powi(4)).sum::<f64>() / w as f64 - 3.0;
                }
            }
            result
        }
        Formula::MaxRet(w) => rolling_max_ret(&c, w, true),
        Formula::MinRet(w) => rolling_max_ret(&c, w, false),
        Formula::HighLowRatio => {
            c.iter().enumerate().map(|(i, &ci)| if ci > 0.0 { (h[i] - l[i]) / ci } else { 0.0 }).collect()
        }
        Formula::Range(w) => {
            let hh = rolling_max(&h, w);
            let ll = rolling_min(&l, w);
            c.iter().enumerate().map(|(i, &ci)| if ci > 0.0 { (hh[i] - ll[i]) / ci } else { 0.0 }).collect()
        }
        Formula::Ulcer(w) => {
            let hh = rolling_max(&c, w);
            let mut result = vec![0.0; n];
            for i in 0..n {
                if i < w { continue; }
                let sum: f64 = (i + 1 - w..=i).map(|j| {
                    let r = if hh[j] > 0.0 { (c[j] - hh[j]) / hh[j] * 100.0 } else { 0.0 };
                    r * r
                }).sum();
                result[i] = (sum / w as f64).sqrt();
            }
            result
        }
        Formula::HighLow(w) => {
            let hh = rolling_max(&h, w);
            let ll = rolling_min(&l, w);
            c.iter().enumerate().map(|(i, &ci)| if ci > 0.0 { (hh[i] - ll[i]) / ci } else { 0.0 }).collect()
        }
        Formula::AvgAmplitude(w) => {
            let daily_amp: Vec<f64> = c.iter().enumerate().map(|(i, &ci)| {
                let prev = if i > 0 { c[i - 1] } else { ci };
                if prev > 0.0 { (h[i] - l[i]) / prev } else { 0.0 }
            }).collect();
            sma(&daily_amp, w)
        }

        // ── Volume ──
        Formula::Vma(w) => sma(&v, w),
        Formula::Vstd(w) => rolling_std(&v, w),
        Formula::Vosc => {
            let vma5 = sma(&v, 5);
            let vma20 = sma(&v, 20);
            vma5.iter().zip(&vma20).map(|(a, b)| if *a > 0.0 { (a - b) / a } else { 0.0 }).collect()
        }
        Formula::Obv => {
            let mut obv = vec![0.0; n];
            for i in 1..n {
                obv[i] = obv[i-1] + if c[i] > c[i-1] { v[i] }
                    else if c[i] < c[i-1] { -v[i] }
                    else { 0.0 };
            }
            obv
        }
        Formula::VwapDev(w) => {
            let vwap_ = vwap(&h, &l, &c, &v, w);
            c.iter().zip(&vwap_).map(|(ci, vw)| if *vw > 0.0 { ci / vw - 1.0 } else { 0.0 }).collect()
        }
        Formula::PvCorr(w) => {
            let ret = pct_change(&c, 1);
            let mut result = vec![0.0; n];
            for i in 0..n {
                if i < w { continue; }
                let start = i + 1 - w;
                let r_slice: Vec<f64> = (start..=i).map(|j| ret[j]).collect();
                let v_slice: Vec<f64> = (start..=i).map(|j| v[j]).collect();
                result[i] = pearson_corr(&r_slice, &v_slice);
            }
            result
        }
        Formula::VolumeRatioFactor => {
            let vma5 = sma(&v, 5);
            v.iter().zip(&vma5).map(|(vi, ma)| if *ma > 0.0 { vi / ma } else { 1.0 }).collect()
        }
        Formula::VolumeTrend => {
            let vma5 = sma(&v, 5);
            let vma20 = sma(&v, 20);
            vma5.iter().zip(&vma20).map(|(a, b)| if *b > 0.0 { a / b } else { 1.0 }).collect()
        }
        Formula::Mfi(w) => {
            let tp: Vec<f64> = h.iter().zip(&l).zip(&c).map(|((&hi, &li), &ci)| (hi + li + ci) / 3.0).collect();
            let mf: Vec<f64> = tp.iter().zip(&v).map(|(&t, &vol)| t * vol).collect();
            let mut result = vec![0.0; n];
            for i in 0..n {
                if i < w { continue; }
                let mut pos_flow = 0.0;
                let mut neg_flow = 0.0;
                for j in (i + 1 - w..=i).filter(|&j| j > 0) {
                    if tp[j] > tp[j-1] { pos_flow += mf[j]; }
                    else if tp[j] < tp[j-1] { neg_flow += mf[j]; }
                }
                let total = pos_flow + neg_flow;
                result[i] = if total > 0.0 { 100.0 - 100.0 / (1.0 + pos_flow / total) } else { 50.0 };
            }
            result
        }
        Formula::Eom => {
            let mut result = vec![0.0; n];
            for i in 0..n {
                if i < 1 || v[i] <= 0.0 { continue; }
                let range_ratio = (h[i] - l[i]) / v[i];
                result[i] = range_ratio / ((h[i] + l[i]) / 2.0 - (h[i-1] + l[i-1]) / 2.0).abs().max(0.0001);
            }
            result
        }
        Formula::ForceIndex => {
            let mut fi = vec![0.0; n];
            for i in 1..n {
                fi[i] = (c[i] - c[i-1]) * v[i];
            }
            fi
        }
        Formula::Ad => {
            let mut ad = vec![0.0; n];
            for i in 0..n {
                let hl = h[i] - l[i];
                if hl <= 0.0 { ad[i] = if i > 0 { ad[i-1] } else { 0.0 }; continue; }
                let clv = ((c[i] - l[i]) - (h[i] - c[i])) / hl;
                ad[i] = if i > 0 { ad[i-1] } else { 0.0 } + clv * v[i];
            }
            ad
        }
        Formula::Cmf(w) => {
            let ad = compute_factor(df, Formula::Ad, 0);
            let sum_ad: Vec<f64> = (0..n).map(|i| {
                let start = if i >= w { i - w + 1 } else { 0 };
                (start..=i).map(|j| ad[j]).sum::<f64>()
            }).collect();
            let sum_v: Vec<f64> = (0..n).map(|i| {
                let start = if i >= w { i - w + 1 } else { 0 };
                (start..=i).map(|j| v[j]).sum::<f64>()
            }).collect();
            sum_ad.iter().zip(&sum_v).map(|(a, vol)| if *vol > 0.0 { a / vol } else { 0.0 }).collect()
        }
        Formula::Nvi => {
            let mut nvi = vec![1000.0; n];
            for i in 1..n {
                nvi[i] = if v[i] < v[i-1] {
                    nvi[i-1] * (1.0 + pct_change_single(c[i], c[i-1]))
                } else { nvi[i-1] };
            }
            nvi.iter().map(|x| x / 1000.0 - 1.0).collect()
        }
        Formula::Pvi => {
            let mut pvi = vec![1000.0; n];
            for i in 1..n {
                pvi[i] = if v[i] > v[i-1] {
                    pvi[i-1] * (1.0 + pct_change_single(c[i], c[i-1]))
                } else { pvi[i-1] };
            }
            pvi.iter().map(|x| x / 1000.0 - 1.0).collect()
        }
        Formula::VolPriceTrend => {
            let ret = pct_change(&c, 1);
            let mut vpt = vec![0.0; n];
            for i in 1..n {
                vpt[i] = vpt[i-1] + ret[i].min(1.0).max(-1.0) * v[i] / v.iter().take(i+1).sum::<f64>().max(1.0);
            }
            vpt
        }
        Formula::TvMa(w) => {
            if to.is_empty() { return vec![0.0; n]; }
            let ma = sma(&to, w);
            to.iter().zip(&ma).map(|(t, m)| if *m > 0.0 { t / m - 1.0 } else { 0.0 }).collect()
        }
        Formula::UpsideVolume(w) => {
            let mut result = vec![0.0; n];
            for i in 0..n {
                let start = if i >= w { i - w + 1 } else { 0 };
                let up: f64 = (start..=i).filter(|&j| j == 0 || c[j] > c[j-1]).map(|j| v[j]).sum();
                let total: f64 = (start..=i).map(|j| v[j]).sum();
                result[i] = if total > 0.0 { up / total } else { 0.5 };
            }
            result
        }
        Formula::DownsideVolume(w) => {
            let mut result = vec![0.0; n];
            for i in 0..n {
                let start = if i >= w { i - w + 1 } else { 0 };
                let dn: f64 = (start..=i).filter(|&j| j > 0 && c[j] < c[j-1]).map(|j| v[j]).sum();
                let total: f64 = (start..=i).map(|j| v[j]).sum();
                result[i] = if total > 0.0 { dn / total } else { 0.5 };
            }
            result
        }

        // ── Momentum ──
        Formula::Ret(w) => pct_change(&c, w),
        Formula::Sr(w) => {
            let ret = pct_change(&c, 1);
            let mean_ret = sma(&ret, w);
            let std_ret = rolling_std(&ret, w);
            mean_ret.iter().zip(&std_ret).map(|(m, s)| if *s > 0.0 { m / s } else { 0.0 }).collect()
        }
        Formula::Rs(w) => {
            let sma_ = sma(&c, w);
            c.iter().zip(&sma_).map(|(ci, s)| if *s > 0.0 { ci / s } else { 1.0 }).collect()
        }
        Formula::HighDist(w) => {
            let hh = rolling_max(&c, w);
            c.iter().zip(&hh).map(|(ci, h)| if *h > 0.0 { (h - ci) / h } else { 0.0 }).collect()
        }
        Formula::LowDist(w) => {
            let ll = rolling_min(&c, w);
            c.iter().zip(&ll).map(|(ci, l)| if *l > 0.0 { (ci - l) / l } else { 0.0 }).collect()
        }
        Formula::MomDiv(w) => {
            let ret_n = pct_change(&c, w);
            let ret_4n = pct_change(&c, w * 4);
            ret_n.iter().zip(&ret_4n).map(|(rn, r4)| {
                let den = r4.abs().max(0.0001);
                rn / den
            }).collect()
        }
        Formula::Roc(w) => {
            c.iter().enumerate().map(|(i, &ci)| {
                if i < w { return 0.0; }
                let prev = c[i-w];
                if prev > 0.0 { (ci - prev) / prev } else { 0.0 }
            }).collect()
        }
        Formula::LtmRatio => {
            let ret60 = pct_change(&c, 60);
            let ret20 = pct_change(&c, 20);
            ret60.iter().zip(&ret20).map(|(r60, r20)| {
                if r20.abs() > 0.0001 { r60 / r20 } else { 1.0 }
            }).collect()
        }
        Formula::PctRank(w) => {
            let mut result = vec![0.0; n];
            for i in 0..n {
                let start = if i >= w { i - w + 1 } else { 0 };
                let count = (start..=i).filter(|&j| c[i] > c[j]).count() as f64;
                let total = (i - start + 1) as f64;
                result[i] = count / total.max(1.0);
            }
            result
        }

        // ── Liquidity ──
        Formula::Turn(w) => {
            if to.is_empty() { return vec![0.0; n]; }
            let ma = sma(&to, w);
            to.iter().zip(&ma).map(|(t, m)| if *m > 0.0 { t / m } else { 1.0 }).collect()
        }
        Formula::TurnStd(w) => {
            if to.is_empty() { return vec![0.0; n]; }
            rolling_std(&to, w)
        }
        Formula::Amihud(w) => {
            let ret = pct_change(&c, 1);
            let mut result = vec![0.0; n];
            for i in 0..n {
                let start = if i >= w { i - w + 1 } else { 0 };
                let sum: f64 = (start..=i).map(|j| {
                    if amt[j] > 0.0 { ret[j].abs() / amt[j] * 1e8 } else { 0.0 }
                }).sum();
                result[i] = sum / (i - start + 1) as f64;
            }
            result
        }
        Formula::DollarVol(w) => {
            let amt_ma = sma(&amt, w);
            amt_ma.iter().map(|a| a.ln()).collect()
        }
        Formula::Spread(w) => {
            let mut result = vec![0.0; n];
            for i in 0..n {
                let start = if i >= w { i - w + 1 } else { 0 };
                let sum: f64 = (start..=i).map(|j| {
                    let mid = (h[j] + l[j]) / 2.0;
                    if mid > 0.0 { (h[j] - l[j]) / mid } else { 0.0 }
                }).sum();
                result[i] = sum / (i - start + 1) as f64;
            }
            result
        }
        Formula::Vwp(w) => {
            if to.is_empty() { return vec![0.0; n]; }
            let mut result = vec![0.0; n];
            for i in 0..n {
                let start = if i >= w { i - w + 1 } else { 0 };
                let tp_sum: f64 = (start..=i).map(|j| (h[j] + l[j] + c[j]) / 3.0 * v[j]).sum();
                let v_sum: f64 = (start..=i).map(|j| v[j]).sum();
                let vwp_val = if v_sum > 0.0 { tp_sum / v_sum } else { c[i] };
                result[i] = if vwp_val > 0.0 { c[i] / vwp_val - 1.0 } else { 0.0 };
            }
            result
        }
        Formula::VolConc(w) => {
            let mut result = vec![0.0; n];
            for i in 0..n {
                let start = if i >= w { i - w + 1 } else { 0 };
                let total: f64 = (start..=i).map(|j| v[j]).sum();
                if total <= 0.0 { continue; }
                let max_v = (start..=i).fold(0.0f64, |a, j| a.max(v[j]));
                result[i] = max_v / total;
            }
            result
        }
        Formula::LiqRatio(w) => {
            let ret = pct_change(&c, 1);
            let mut result = vec![0.0; n];
            for i in 0..n {
                let start = if i >= w { i - w + 1 } else { 0 };
                let sum: f64 = (start..=i).map(|j| ret[j].abs()).sum();
                let total_amt: f64 = (start..=i).map(|j| amt[j]).sum();
                result[i] = if total_amt > 0.0 { sum / total_amt * 1e9 } else { 0.0 };
            }
            result
        }
        Formula::PriceImpact(w) => {
            let ret = pct_change(&c, 1);
            let mut result = vec![0.0; n];
            for i in 0..n {
                let start = if i >= w { i - w + 1 } else { 0 };
                let sum: f64 = (start..=i).map(|j| {
                    if v[j] > 0.0 { ret[j].abs() / v[j] } else { 0.0 }
                }).sum();
                result[i] = sum / (i - start + 1) as f64 * 1e6;
            }
            result
        }
        Formula::VolBreak(w) => {
            let vma = sma(&v, w);
            v.iter().zip(&vma).map(|(vi, ma)| if *ma > 0.0 { vi / ma } else { 1.0 }).collect()
        }
        Formula::FloatProxy => {
            if to.is_empty() || amt.is_empty() { return vec![0.0; n]; }
            amt.iter().zip(&to).map(|(a, t)| {
                if *t > 0.0 { a / t / 10000.0 } else { 0.0 }
            }).collect()
        }
        Formula::IlliqStd(w) => {
            let ret = pct_change(&c, 1);
            let illiq: Vec<f64> = ret.iter().zip(&amt).map(|(r, a)| {
                if *a > 0.0 { r.abs() / a * 1e8 } else { 0.0 }
            }).collect();
            rolling_std(&illiq, w)
        }
    }
}

/// Batch compute multiple factors
pub fn compute_batch(df: &DataFrame, formulas: &[(Formula, usize)]) -> HashMap<String, Vec<f64>> {
    let mut results = HashMap::new();
    for (formula, window) in formulas {
        let name = format!("{:?}_{}", formula, window);
        results.insert(name, compute_factor(df, *formula, *window));
    }
    results
}

/// Compute all 158 factors at once (for downstream ML models)
pub fn compute_all(df: &DataFrame) -> HashMap<String, Vec<f64>> {
    let all_factors = factors::list_all();
    let mut results = HashMap::new();
    for fm in &all_factors {
        let values = compute_factor(df, fm.formula, fm.window);
        results.insert(fm.name.clone(), values);
    }
    results
}

/// Compute factor by name
pub fn compute_by_name(df: &DataFrame, name: &str) -> Option<Vec<f64>> {
    let factor = factors::find_by_name(name)?;
    Some(compute_factor(df, factor.formula, factor.window))
}

// ── Math Helpers ──

fn sma(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    let mut sum = 0.0;
    for i in 0..n {
        sum += data[i];
        if i >= period { sum -= data[i - period]; }
        let count = (i + 1).min(period) as f64;
        result[i] = sum / count;
    }
    result
}

fn ema(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    if n == 0 { return result; }
    let alpha = 2.0 / (period as f64 + 1.0);
    result[0] = data[0];
    for i in 1..n {
        result[i] = alpha * data[i] + (1.0 - alpha) * result[i-1];
    }
    result
}

fn rolling_std(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    for i in 0..n {
        sum += data[i];
        sum_sq += data[i] * data[i];
        if i >= period {
            sum -= data[i - period];
            sum_sq -= data[i - period] * data[i - period];
        }
        let count = (i + 1).min(period) as f64;
        let mean = sum / count;
        let variance = (sum_sq / count - mean * mean).max(0.0);
        result[i] = variance.sqrt();
    }
    result
}

fn rolling_max(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    for i in 0..n {
        let start = if i >= period { i - period + 1 } else { 0 };
        result[i] = (start..=i).fold(f64::NEG_INFINITY, |a, j| a.max(data[j]));
    }
    result
}

fn rolling_min(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    for i in 0..n {
        let start = if i >= period { i - period + 1 } else { 0 };
        result[i] = (start..=i).fold(f64::INFINITY, |a, j| a.min(data[j]));
    }
    result
}

fn pct_change(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    for i in period..n {
        if data[i-period] != 0.0 {
            result[i] = (data[i] - data[i-period]) / data[i-period];
        }
    }
    result
}

fn pct_change_single(curr: f64, prev: f64) -> f64 {
    if prev != 0.0 { (curr - prev) / prev } else { 0.0 }
}

fn rolling_max_ret(data: &[f64], period: usize, is_max: bool) -> Vec<f64> {
    let ret = pct_change(data, 1);
    let n = data.len();
    let mut result = vec![0.0; n];
    for i in 0..n {
        if i < period { continue; }
        let start = i - period + 1;
        result[i] = if is_max {
            (start..=i).fold(f64::NEG_INFINITY, |a, j| a.max(ret[j]))
        } else {
            (start..=i).fold(f64::INFINITY, |a, j| a.min(ret[j]))
        };
    }
    result
}

fn rsi(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;
    for i in 1..n {
        let change = data[i] - data[i-1];
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { -change } else { 0.0 };
        if i < period {
            avg_gain += gain / period as f64;
            avg_loss += loss / period as f64;
        } else {
            avg_gain = (avg_gain * (period - 1) as f64 + gain) / period as f64;
            avg_loss = (avg_loss * (period - 1) as f64 + loss) / period as f64;
        }
        if avg_loss == 0.0 { result[i] = 100.0; }
        else { result[i] = 100.0 - 100.0 / (1.0 + avg_gain / avg_loss); }
    }
    result
}

fn kdj_values(h: &[f64], l: &[f64], c: &[f64], period: usize, smooth: usize) -> (Vec<f64>, Vec<f64>) {
    let n = c.len();
    let mut k = vec![50.0; n];
    let mut d = vec![50.0; n];
    for i in 0..n {
        let start = if i >= period { i - period + 1 } else { 0 };
        let hh = (start..=i).fold(f64::NEG_INFINITY, |a, j| a.max(h[j]));
        let ll = (start..=i).fold(f64::INFINITY, |a, j| a.min(l[j]));
        let rsv = if hh - ll > 0.0 { (c[i] - ll) / (hh - ll) * 100.0 } else { 50.0 };
        if i == 0 { k[i] = 50.0; d[i] = 50.0; continue; }
        k[i] = (k[i-1] * (smooth - 1) as f64 + rsv) / smooth as f64;
        d[i] = (d[i-1] * (smooth - 1) as f64 + k[i]) / smooth as f64;
    }
    (k, d)
}

fn pdi_mdi_adx(h: &[f64], l: &[f64], c: &[f64], period: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = c.len();
    let mut pdi = vec![0.0; n];
    let mut mdi = vec![0.0; n];
    let mut adx = vec![0.0; n];

    let tr = {
        let mut tr = vec![0.0; n];
        for i in 0..n {
            let prev_c = if i > 0 { c[i-1] } else { c[i] };
            tr[i] = (h[i] - l[i]).max((h[i] - prev_c).abs()).max((l[i] - prev_c).abs());
        }
        tr
    };

    let atr = sma(&tr, period);
    let mut plus_dm = vec![0.0; n];
    let mut minus_dm = vec![0.0; n];

    for i in 1..n {
        let up = h[i] - h[i-1];
        let dn = l[i-1] - l[i];
        plus_dm[i] = if up > dn && up > 0.0 { up } else { 0.0 };
        minus_dm[i] = if dn > up && dn > 0.0 { dn } else { 0.0 };
    }

    let sma_pdm = sma(&plus_dm, period);
    let sma_mdm = sma(&minus_dm, period);

    for i in 0..n {
        if atr[i] > 0.0 {
            pdi[i] = sma_pdm[i] / atr[i] * 100.0;
            mdi[i] = sma_mdm[i] / atr[i] * 100.0;
        }
        let dx_den = pdi[i] + mdi[i];
        if dx_den > 0.0 {
            let dx = (pdi[i] - mdi[i]).abs() / dx_den * 100.0;
            if i > 0 {
                adx[i] = (adx[i-1] * (period - 1) as f64 + dx) / period as f64;
            } else {
                adx[i] = dx;
            }
        }
    }
    (pdi, mdi, adx)
}

fn vwap(h: &[f64], l: &[f64], c: &[f64], v: &[f64], period: usize) -> Vec<f64> {
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 0..n {
        let start = if i >= period { i - period + 1 } else { 0 };
        let mut tp_v_sum = 0.0;
        let mut v_sum = 0.0;
        for j in start..=i {
            let tp = (h[j] + l[j] + c[j]) / 3.0;
            tp_v_sum += tp * v[j];
            v_sum += v[j];
        }
        result[i] = if v_sum > 0.0 { tp_v_sum / v_sum } else { c[i] };
    }
    result
}

fn pearson_corr(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    if n < 2.0 { return 0.0; }
    let mx = x.iter().sum::<f64>() / n;
    let my = y.iter().sum::<f64>() / n;
    let (mut cov, mut sx2, mut sy2) = (0.0, 0.0, 0.0);
    for (xi, yi) in x.iter().zip(y) {
        let dx = xi - mx;
        let dy = yi - my;
        cov += dx * dy;
        sx2 += dx * dx;
        sy2 += dy * dy;
    }
    let den = (sx2 * sy2).sqrt();
    if den > 0.0 { cov / den } else { 0.0 }
}

// ── Factor Registry ──

/// List all factor metadata
pub fn list_all_factors() -> Vec<FactorMeta> {
    factors::list_all()
}

/// Get factors by category
pub fn list_by_category(category: Category) -> Vec<FactorMeta> {
    factors::list_all().into_iter().filter(|f| f.category == category).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factor_count() {
        let all = factors::list_all();
        assert!(all.len() >= 140, "Should have at least 140 factors, got {}", all.len());
        let trend = all.iter().filter(|f| f.category == Category::Trend).count();
        assert!(trend >= 25, "Trend: {} factors", trend);
    }
}
