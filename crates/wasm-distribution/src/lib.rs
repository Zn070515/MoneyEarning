use wasm_core::{DataFrame, DistributionResult, VolumeProfileResult, ProfileLevel};

/// Compute traditional volume profile (Market Profile)
/// Groups volume by price levels and finds POC/VA/VWAP
pub fn volume_profile(df: &DataFrame, num_buckets: usize) -> VolumeProfileResult {
    let close = df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    let volume = df.column("volume").map(|c| c.to_f64_vec()).unwrap_or_default();
    let n = close.len();
    if n == 0 {
        return VolumeProfileResult { levels: vec![], poc: 0.0, vah: 0.0, val: 0.0 };
    }

    let min_p = close.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_p = close.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let bucket_size = (max_p - min_p) / num_buckets as f64;

    let mut bucket_vol: Vec<f64> = vec![0.0; num_buckets];
    let mut bucket_price: Vec<f64> = vec![0.0; num_buckets];

    for i in 0..n {
        let idx = ((close[i] - min_p) / bucket_size) as usize;
        let idx = idx.min(num_buckets - 1);
        bucket_vol[idx] += volume[i];
        bucket_price[idx] += close[i] * volume[i];
    }

    let total_vol: f64 = bucket_vol.iter().sum();
    let mut levels: Vec<ProfileLevel> = Vec::new();

    for i in 0..num_buckets {
        if bucket_vol[i] > 0.0 {
            let avg_price = bucket_price[i] / bucket_vol[i];
            levels.push(ProfileLevel {
                price: avg_price,
                volume: bucket_vol[i],
                is_poc: false,
            });
        }
    }

    levels.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal));

    // Find POC
    let mut poc_idx = 0;
    let mut max_vol = 0.0f64;
    for (i, l) in levels.iter().enumerate() {
        if l.volume > max_vol {
            max_vol = l.volume;
            poc_idx = i;
        }
    }
    let poc = levels[poc_idx].price;
    levels[poc_idx].is_poc = true;

    // Value Area (70% of volume around POC)
    let va_target = total_vol * 0.70;
    let mut va_vol = levels[poc_idx].volume;
    let mut lo = poc_idx;
    let mut hi = poc_idx;

    while va_vol < va_target {
        let lo_vol = if lo > 0 { levels[lo - 1].volume } else { 0.0 };
        let hi_vol = if hi + 1 < levels.len() { levels[hi + 1].volume } else { 0.0 };

        if lo_vol >= hi_vol && lo > 0 {
            lo -= 1;
            va_vol += levels[lo].volume;
        } else if hi + 1 < levels.len() {
            hi += 1;
            va_vol += levels[hi].volume;
        } else if lo > 0 {
            lo -= 1;
            va_vol += levels[lo].volume;
        } else {
            break;
        }
    }

    let val = levels[lo].price;
    let vah = levels[hi].price;

    VolumeProfileResult { levels, poc, vah, val }
}

/// Compute cost distribution (筹码分布)
/// Estimates the distribution of holding costs based on price-volume patterns
pub fn cost_distribution(df: &DataFrame) -> DistributionResult {
    let close = df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    let volume = df.column("volume").map(|c| c.to_f64_vec()).unwrap_or_default();
    let high = df.column("high").map(|c| c.to_f64_vec()).unwrap_or_default();
    let low = df.column("low").map(|c| c.to_f64_vec()).unwrap_or_default();
    let n = close.len();

    if n == 0 {
        return DistributionResult {
            price_levels: vec![], chip_volume: vec![],
            avg_cost: 0.0, weighted_avg_cost: 0.0,
        };
    }

    let num_levels = 100usize;
    let min_p = low.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_p = high.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let step = (max_p - min_p) / num_levels as f64;

    // Simulate chip distribution using turnover-based decay
    // Each day, a portion of old chips are "transferred" (churned) based on turnover rate
    let mut chip_levels = vec![0.0f64; num_levels];

    let mut total_chips = 0.0f64;
    let mut total_cost = 0.0f64;

    for i in 0..n {
        let price = close[i];
        let vol = volume[i];
        let turnover_rate = df.column("turnover")
            .and_then(|c| c.f64_values())
            .map(|t| t.get(i).copied().unwrap_or(0.02))
            .unwrap_or(0.02);

        // Decay existing chips
        let decay = turnover_rate.min(1.0);
        for level in chip_levels.iter_mut() {
            *level *= 1.0 - decay;
        }

        // Add new chips at current price
        let idx = ((price - min_p) / step) as usize;
        let idx = idx.min(num_levels - 1);
        chip_levels[idx] += vol;

        total_chips += vol * (1.0 - decay);
        total_cost += price * vol * (1.0 - decay);
    }

    let total: f64 = chip_levels.iter().sum();
    let price_levels: Vec<f64> = (0..num_levels)
        .map(|i| min_p + step * (i as f64 + 0.5))
        .collect();

    // Normalize
    let chip_volume: Vec<f64> = if total > 0.0 {
        chip_levels.iter().map(|v| v / total * 100.0).collect()
    } else {
        chip_levels.clone()
    };

    let avg_cost = if total_chips > 0.0 { total_cost / total_chips } else { close[n - 1] };
    let weighted_avg_cost = if total > 0.0 {
        price_levels.iter().zip(chip_levels.iter())
            .map(|(p, v)| p * v).sum::<f64>() / total
    } else { avg_cost };

    DistributionResult {
        price_levels,
        chip_volume,
        avg_cost,
        weighted_avg_cost,
    }
}

/// Compute moving cost distribution (移动筹码分布)
/// Tracks how chips migrate over time for each day in the recent window
pub fn moving_distribution(df: &DataFrame, window: usize) -> Vec<(String, f64, f64)> {
    let n = df.len();
    let start = if n > window { n - window } else { 0 };

    (start..n)
        .map(|i| {
            let slice = df.slice(0, i + 1);
            let dist = cost_distribution(&slice);
            let date = i.to_string();
            (date, dist.avg_cost, dist.weighted_avg_cost)
        })
        .collect()
}

/// Support/Resistance detection based on volume nodes
pub fn volume_sr_levels(df: &DataFrame, num_levels: usize) -> Vec<(f64, f64, String)> {
    let profile = volume_profile(df, num_levels);

    let mut levels: Vec<(f64, f64, String)> = Vec::new();

    // High volume nodes = potential S/R
    let avg_vol: f64 = profile.levels.iter().map(|l| l.volume).sum::<f64>()
        / profile.levels.len().max(1) as f64;

    let last_price = df.column("close")
        .and_then(|c| c.f64_values())
        .and_then(|v| v.last().copied())
        .unwrap_or(0.0);

    for l in &profile.levels {
        if l.volume > avg_vol * 1.5 {
            let kind = if l.price > last_price { "resistance" } else { "support" };
            levels.push((l.price, l.volume, kind.to_string()));
        }
    }

    // Add POC
    levels.push((profile.poc, profile.levels.iter()
        .find(|l| l.is_poc).map(|l| l.volume).unwrap_or(0.0),
        "poc".to_string()));

    levels.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    levels
}
