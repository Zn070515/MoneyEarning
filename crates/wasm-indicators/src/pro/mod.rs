pub mod trend;
pub mod momentum;
pub mod volatility;
pub mod volume;
pub mod cycle;
pub mod composite;

use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput};

pub fn compute(name: &str, df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    match name {
        // PRO Trend
        "kama" => trend::kama::compute(df, params),
        "hma" => trend::hma::compute(df, params),
        "t3" => trend::t3::compute(df, params),
        "vidya" => trend::vidya::compute(df, params),
        "alma" => trend::alma::compute(df, params),
        "lsma" => trend::lsma::compute(df, params),
        "frama" => trend::frama::compute(df, params),
        "chandelier_exit" => trend::chandelier_exit::compute(df, params),
        "jma" => trend::jma::compute(df, params),
        "gmma" => trend::gmma::compute(df, params),

        // PRO Momentum
        "tsi" => momentum::tsi::compute(df, params),
        "smi" => momentum::smi::compute(df, params),
        "stoch_rsi" => momentum::stoch_rsi::compute(df, params),
        "trix" => momentum::trix::compute(df, params),
        "elder_ray" => momentum::elder_ray::compute(df, params),
        "ultimate_osc" => momentum::ultimate_osc::compute(df, params),
        "rmi" => momentum::rmi::compute(df, params),
        "ergotic" => momentum::ergotic::compute(df, params),

        // PRO Volatility
        "hist_vol" => volatility::hist_vol::compute(df, params),
        "gk_vol" => volatility::gk_vol::compute(df, params),
        "parkinson_vol" => volatility::parkinson_vol::compute(df, params),
        "rs_vol" => volatility::rs_vol::compute(df, params),
        "yz_vol" => volatility::yz_vol::compute(df, params),
        "hurst" => volatility::hurst::compute(df, params),

        // PRO Volume
        "kvo" => volume::kvo::compute(df, params),
        "vfi" => volume::vfi::compute(df, params),
        "force_index" => volume::force_index::compute(df, params),
        "mfi2" => volume::mfi2::compute(df, params),
        "volume_osc" => volume::volume_osc::compute(df, params),
        "emv_v2" => volume::emv_v2::compute(df, params),
        "volume_regime" => volume::volume_regime::compute(df, params),

        // PRO Cycle
        "hilbert_sine" => cycle::hilbert_sine::compute(df, params),
        "mesa_sine" => cycle::mesa_sine::compute(df, params),
        "cg" => cycle::cg::compute(df, params),
        "phasor" => cycle::phasor::compute(df, params),

        // PRO Composite
        "ichimoku" => composite::ichimoku::compute(df, params),
        "alligator" => composite::alligator::compute(df, params),
        "fractals" => composite::fractals::compute(df, params),
        "ao" => composite::ao::compute(df, params),
        "ac" => composite::ac::compute(df, params),
        "rainbow_ma" => composite::rainbow_ma::compute(df, params),
        "pivots" => composite::pivots::compute(df, params),
        "regression_channel" => composite::regression_channel::compute(df, params),
        "decision_point" => composite::decision_point::compute(df, params),
        "threshold_bands" => composite::threshold_bands::compute(df, params),

        _ => Err(IndError::InvalidName),
    }
}
