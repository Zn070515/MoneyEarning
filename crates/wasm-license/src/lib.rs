use rsa::RsaPublicKey;
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::signature::Verifier;
use rsa::pkcs8::DecodePublicKey;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Embedded RSA-4096 public key in PKCS#8 PEM format
const PUBLIC_KEY_PEM: &str = "\
-----BEGIN PUBLIC KEY-----
MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAroFFidKRCThaPahV05rK
k3nz/NP4hgvJGD3xzjWK5xf7w+/canMNhdyLvzzi4AOhz4le293icbaK5lUYq1Hv
4jE5ficnR69Htc6z9V1e6bYSV6lmD2j1qy/xHr1O5C3/KEYIUU2Q2NTGYhZh/GbP
IcgPC+GyTVfSkytSKHW0rOd4QHKNKpSlSDGG2Nib7il3Tlwo8aJsDT2gXqgEjHTC
BkuGXo1d2svymaJsiEgoIuToNXIgTsvPA7zsLf5A/ngunHUaTYA5OVwWwS/7ZHIV
mZ4tbTfGHvWBohMn4QmmRnMeQLFdhGWSjbQr8wQbyltkk/HOSzneBhstU1ac8fAj
tzoSurMoyvFbeK2p/3HTknwlhEqnhte8Rfbzv1xLVmh7QuFKW/qnhFoozo9IvzOb
ikDWj0BGVurB0kMQ8+Eo5F/Fg83fA2RId0LqYmk5CMhRnQvYnjCrcUrh3LoBBhiX
7b//gleh+Xd6DGjWyIuV+wWXAXncPGmup3znLhtoIsxOzvfbxlhLHlNGZHQvoAo0
ik9zQ9s/w/UCttdERCqT63PkWPtmu7LffQJJAIg9I8pRy323aP9f+irk5gD7mQLZ
AbJeNkxNdddllM4d5qPLrk1YY5yg12CpSsL/JWLRXr9bUCB8/xSICduNj4rdFkF5
RVWLJQR/w3JCNAGTa2L8+B8CAwEAAQ==
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

/// Sign a license payload with the RSA private key (feature-gated: `signing`).
/// Returns a license key string: `base64(payload).base64(signature)`.
#[cfg(feature = "signing")]
pub fn sign_license(
    fingerprint: &str,
    tier: &str,
    expiry: Option<&str>,
    features: &[String],
    private_key_pem: &str,
) -> Result<String, String> {
    use rsa::pkcs8::DecodePrivateKey;
    use rsa::pkcs1v15::SigningKey;
    use rsa::signature::{Signer, SignatureEncoding};

    let private_key = rsa::RsaPrivateKey::from_pkcs8_pem(private_key_pem)
        .map_err(|e| format!("无法加载私钥: {}", e))?;

    let payload = LicensePayload {
        fingerprint: fingerprint.to_string(),
        tier: tier.to_string(),
        expiry: expiry.map(|s| s.to_string()),
        issued_at: chrono::Local::now().date_naive().to_string(),
        features: features.to_vec(),
    };

    let payload_json = serde_json::to_string(&payload)
        .map_err(|e| format!("序列化失败: {}", e))?;

    let signing_key = SigningKey::<Sha256>::new_unprefixed(private_key);
    let signature = signing_key.sign(payload_json.as_bytes());

    let payload_b64 = BASE64.encode(payload_json.as_bytes());
    let sig_b64 = BASE64.encode(signature.to_bytes());

    Ok(format!("{}.{}", payload_b64, sig_b64))
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

/// Compute SHA-256 checksum of WASM binary for integrity verification.
/// The result is compared against an expected hash embedded at build time.
pub fn checksum_module(wasm_bytes: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(wasm_bytes);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Verify WASM module integrity by comparing checksum against expected value
pub fn verify_integrity(wasm_bytes: &[u8], expected_hash: &[u8; 32]) -> bool {
    let actual = checksum_module(wasm_bytes);
    constant_time_eq(&actual, expected_hash)
}

/// Constant-time comparison to prevent timing attacks
fn constant_time_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}
