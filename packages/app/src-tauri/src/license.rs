use std::sync::Mutex;
use wasm_license;

static LICENSE_PAYLOAD: Mutex<Option<wasm_license::LicensePayload>> = Mutex::new(None);

pub fn generate_fingerprint() -> Result<String, Box<dyn std::error::Error>> {
    let hostname = hostname::get()?.to_string_lossy().to_string();
    let mac = get_mac_address()?;
    let os_serial = get_os_serial();
    Ok(wasm_license::hash_fingerprint(&mac, &hostname, &os_serial))
}

pub fn activate(license_key: &str, fingerprint: &str) -> Result<super::LicenseInfo, Box<dyn std::error::Error>> {
    let payload = wasm_license::verify_signature(license_key, fingerprint)
        .map_err(|e| e.to_string())?;
    let info = super::LicenseInfo {
        tier: payload.tier.clone(),
        expiry: payload.expiry.clone(),
        features: payload.features.clone(),
        valid: true,
    };
    *LICENSE_PAYLOAD.lock().unwrap() = Some(payload);
    Ok(info)
}

pub fn check() -> Result<super::LicenseStatus, Box<dyn std::error::Error>> {
    let guard = LICENSE_PAYLOAD.lock().unwrap();
    match guard.as_ref() {
        Some(payload) => {
            let mut valid = true;
            if let Some(ref expiry) = payload.expiry {
                // Simple date comparison
                let today = today_str();
                if today > *expiry { valid = false; }
            }
            Ok(super::LicenseStatus {
                valid,
                tier: payload.tier.clone(),
                expiry: payload.expiry.clone(),
                trial_days_left: None,
            })
        }
        None => Ok(super::LicenseStatus {
            valid: false,
            tier: "free".into(),
            expiry: None,
            trial_days_left: Some(30),
        }),
    }
}

fn today_str() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let secs = now.as_secs();
    let days = secs / 86400;
    // Convert epoch days to YYYY-MM-DD
    let y = (days as f64 / 365.2425 + 1970.0) as i64;
    let day_of_year = ((days as f64 - (y - 1970) as f64 * 365.2425) as i64).max(0);
    let month_days = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
        [31,29,31,30,31,30,31,31,30,31,30,31]
    } else {
        [31,28,31,30,31,30,31,31,30,31,30,31]
    };
    let mut m = 1;
    let mut d = day_of_year + 1;
    for &md in &month_days {
        if d <= md { break; }
        d -= md;
        m += 1;
    }
    format!("{:04}-{:02}-{:02}", y, m.min(12).max(1), d.min(31).max(1))
}

fn get_mac_address() -> Result<String, Box<dyn std::error::Error>> {
    let ma = mac_address::get_mac_address()
        .map_err(|e| format!("获取MAC地址失败: {:?}", e))?;
    Ok(format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        ma.bytes()[0], ma.bytes()[1], ma.bytes()[2],
        ma.bytes()[3], ma.bytes()[4], ma.bytes()[5]))
}

fn get_os_serial() -> String {
    // Windows serial placeholder — in production, read from WMI
    #[cfg(target_os = "windows")]
    { String::from("WIN-UNKNOWN") }
    #[cfg(not(target_os = "windows"))]
    { String::from("UNKNOWN") }
}
