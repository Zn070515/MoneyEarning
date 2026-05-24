use std::sync::Mutex;
use wasm_license;

static LICENSE_CACHE: Mutex<Option<super::LicenseStatus>> = Mutex::new(None);

const TRIAL_DAYS: i64 = 14;

pub fn generate_fingerprint() -> Result<String, Box<dyn std::error::Error>> {
    let hostname = hostname::get()?.to_string_lossy().to_string();
    let mac = get_mac_address()?;
    let os_serial = get_os_serial();
    Ok(wasm_license::hash_fingerprint(&mac, &hostname, &os_serial))
}

pub fn activate(
    app: &tauri::AppHandle,
    license_key: &str,
    fingerprint: &str,
) -> Result<super::LicenseInfo, Box<dyn std::error::Error>> {
    let payload = wasm_license::verify_signature(license_key, fingerprint)
        .map_err(|e| format!("激活失败: {}", e))?;

    let guard = crate::db::get_db(app)?;
    crate::db::save_license(
        &guard,
        license_key,
        &payload.tier,
        payload.expiry.as_deref(),
        fingerprint,
        license_key, // store full license key as signature for re-verification
    )?;
    drop(guard);

    let info = super::LicenseInfo {
        tier: payload.tier.clone(),
        expiry: payload.expiry.clone(),
        features: payload.features.clone(),
        valid: true,
    };
    *LICENSE_CACHE.lock().unwrap_or_else(|e| e.into_inner()) = Some(super::LicenseStatus {
        valid: true,
        tier: payload.tier.clone(),
        expiry: payload.expiry.clone(),
        trial_days_left: None,
    });
    Ok(info)
}

/// Fast check from in-memory cache only. Used for UI polling.
pub fn check_cached() -> super::LicenseStatus {
    let guard = LICENSE_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    match guard.as_ref() {
        Some(cached) => cached.clone(),
        None => super::LicenseStatus {
            valid: false,
            tier: "free".into(),
            expiry: None,
            trial_days_left: Some(TRIAL_DAYS as i32),
        },
    }
}

/// Full check: memory → DB → trial calculation. Called at startup and before PRO ops.
pub fn check_with_db(app: &tauri::AppHandle) -> Result<super::LicenseStatus, String> {
    // 1. Memory cache (includes trial status now)
    {
        let cache = LICENSE_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(cached) = cache.as_ref() {
            if cached.valid {
                return Ok(cached.clone());
            }
            // If cached is invalid (expired), fall through to re-check
        }
    }

    // 2. DB-activated license
    let guard = crate::db::get_db(app).map_err(|e| e.to_string())?;
    if let Some(record) = crate::db::load_license(&guard).map_err(|e| e.to_string())? {
        let fingerprint =
            generate_fingerprint().map_err(|e| format!("指纹生成失败: {}", e))?;
        match wasm_license::verify_signature(&record.license_key, &fingerprint) {
            Ok(payload) => {
                let valid = check_expiry(&payload.expiry);
                let status = super::LicenseStatus {
                    valid,
                    tier: payload.tier.clone(),
                    expiry: payload.expiry.clone(),
                    trial_days_left: None,
                };
                if valid {
                    *LICENSE_CACHE.lock().unwrap_or_else(|e| e.into_inner()) = Some(status.clone());
                } else {
                    // Expired — clear from DB and cache
                    let _ = crate::db::clear_license(&guard);
                    *LICENSE_CACHE.lock().unwrap_or_else(|e| e.into_inner()) = None;
                }
                return Ok(status);
            }
            Err(_) => {
                // Signature invalid — clear stale record
                let _ = crate::db::clear_license(&guard);
            }
        }
    }

    // 3. Trial from install_date
    let status = if let Some(install_date) =
        crate::db::get_config(&guard, "install_date").map_err(|e| e.to_string())?
    {
        let elapsed = days_since(&install_date);
        let left = (TRIAL_DAYS - elapsed).max(0) as i32;
        let tier = if left > 0 { "trial" } else { "free" };
        super::LicenseStatus {
            valid: left > 0,
            tier: tier.into(),
            expiry: None,
            trial_days_left: Some(left),
        }
    } else {
        super::LicenseStatus {
            valid: false,
            tier: "free".into(),
            expiry: None,
            trial_days_left: Some(14),
        }
    };
    drop(guard);
    *LICENSE_CACHE.lock().unwrap_or_else(|e| e.into_inner()) = Some(status.clone());
    Ok(status)
}

// ── Date helpers ──

fn check_expiry(expiry: &Option<String>) -> bool {
    match expiry {
        Some(exp) => today_str().as_str() <= exp.as_str(),
        None => true,
    }
}

fn today_str() -> String {
    let unix_days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        / 86400;
    days_to_ymd(unix_days)
}

fn days_since(date: &str) -> i64 {
    let unix_days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        / 86400;
    unix_days - ymd_to_days(date)
}

/// Howard Hinnant algorithm: Y-M-D → days since Unix epoch
fn ymd_to_days(date: &str) -> i64 {
    let parts: Vec<i64> = date.split('-').filter_map(|s| s.parse().ok()).collect();
    if parts.len() != 3 {
        return 0;
    }
    let (y, m, d) = (parts[0], parts[1], parts[2]);
    let yy = y - if m <= 2 { 1 } else { 0 };
    let era = if yy >= 0 {
        yy / 400
    } else {
        (yy - 399) / 400
    };
    let yoe = yy - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719_468
}

/// Howard Hinnant algorithm: days since Unix epoch → YYYY-MM-DD
fn days_to_ymd(z: i64) -> String {
    let z = z + 719_468;
    let era = if z >= 0 {
        z / 146_097
    } else {
        (z - 146_096) / 146_097
    };
    let doe = (z - era * 146_097) as u32;
    let yoe = ((doe - doe / 1460 + doe / 36524 - doe / 146096) / 365) as i64;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe as u32 + yoe as u32 / 4 - yoe as u32 / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

// ── Machine identity ──

fn get_mac_address() -> Result<String, Box<dyn std::error::Error>> {
    let ma = mac_address::get_mac_address()
        .map_err(|e| format!("获取MAC地址失败: {:?}", e))?;
    Ok(format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        ma.bytes()[0],
        ma.bytes()[1],
        ma.bytes()[2],
        ma.bytes()[3],
        ma.bytes()[4],
        ma.bytes()[5]
    ))
}

fn get_os_serial() -> String {
    #[cfg(target_os = "windows")]
    {
        String::from("WIN-UNKNOWN")
    }
    #[cfg(not(target_os = "windows"))]
    {
        String::from("UNKNOWN")
    }
}
