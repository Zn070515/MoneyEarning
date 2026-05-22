use rsa::RsaPublicKey;
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::signature::Verifier;
use rsa::pkcs8::DecodePublicKey;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Embedded RSA-4096 public key in PKCS#8 PEM format
const PUBLIC_KEY_PEM: &str = "\
-----BEGIN PUBLIC KEY-----
MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEA0QAAAP9/f39/f39/f39/
f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f
39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f3
9/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f3
9/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f3
9/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f39/f3//wIDAQAB
-----END PUBLIC KEY-----";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicensePayload {
    pub fingerprint: String,
    pub tier: String,        // "free" | "pro"
    pub expiry: Option<String>, // ISO date, None = perpetual
    pub issued_at: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum LicenseError {
    InvalidFormat,
    InvalidSignature,
    Expired,
    FingerprintMismatch,
    PublicKeyError,
}

impl std::fmt::Display for LicenseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LicenseError::InvalidFormat => write!(f, "授权格式无效"),
            LicenseError::InvalidSignature => write!(f, "授权签名验证失败"),
            LicenseError::Expired => write!(f, "授权已过期"),
            LicenseError::FingerprintMismatch => write!(f, "机器指纹不匹配"),
            LicenseError::PublicKeyError => write!(f, "公钥错误"),
        }
    }
}

/// Parse the embedded public key
fn get_public_key() -> Result<RsaPublicKey, LicenseError> {
    RsaPublicKey::from_public_key_pem(PUBLIC_KEY_PEM).map_err(|_| LicenseError::PublicKeyError)
}

/// Verify RSA signature and return the payload
pub fn verify_signature(license_key: &str, fingerprint: &str) -> Result<LicensePayload, LicenseError> {
    let parts: Vec<&str> = license_key.splitn(2, '.').collect();
    if parts.len() != 2 {
        return Err(LicenseError::InvalidFormat);
    }
    let payload_json = String::from_utf8(BASE64.decode(parts[0]).map_err(|_| LicenseError::InvalidFormat)?)
        .map_err(|_| LicenseError::InvalidFormat)?;
    let signature_bytes = BASE64.decode(parts[1]).map_err(|_| LicenseError::InvalidFormat)?;

    let public_key = get_public_key()?;
    let verifying_key = VerifyingKey::<Sha256>::new_unprefixed(public_key);
    let signature = Signature::try_from(signature_bytes.as_slice())
        .map_err(|_| LicenseError::InvalidFormat)?;

    verifying_key.verify(payload_json.as_bytes(), &signature)
        .map_err(|_| LicenseError::InvalidSignature)?;

    let payload: LicensePayload = serde_json::from_str(&payload_json)
        .map_err(|_| LicenseError::InvalidFormat)?;

    if payload.fingerprint != fingerprint && payload.fingerprint != "*" {
        return Err(LicenseError::FingerprintMismatch);
    }

    if let Some(ref expiry) = payload.expiry {
        if let Ok(exp_date) = chrono::NaiveDate::parse_from_str(expiry, "%Y-%m-%d") {
            let today = chrono::Local::now().date_naive();
            if today > exp_date {
                return Err(LicenseError::Expired);
            }
        }
    }

    Ok(payload)
}

/// Hash a machine fingerprint from hardware identifiers
pub fn hash_fingerprint(mac: &str, hostname: &str, os_serial: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(mac.as_bytes());
    hasher.update(b"|");
    hasher.update(hostname.as_bytes());
    hasher.update(b"|");
    hasher.update(os_serial.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Placeholder for code integrity checksum
pub fn checksum_module() -> [u8; 32] {
    [0u8; 32]
}
