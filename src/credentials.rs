use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use ring::{hmac, signature as ring_sig};
use rsa::{
    RsaPrivateKey,
    pkcs1v15::SigningKey,
    pkcs8::DecodePrivateKey,
    signature::{RandomizedSigner, SignatureEncoding},
};
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Result;

/// Signature algorithm type for API authentication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignatureType {
    /// HMAC-SHA256 (default, most common)
    #[default]
    HmacSha256,
    /// RSA with SHA256 (for institutional/enterprise keys)
    RsaSha256,
    /// Ed25519 (modern, fast alternative)
    Ed25519,
}

/// Internal key storage for different signature types.
enum SigningKey_ {
    Hmac(SecretString),
    Rsa(Arc<RsaPrivateKey>),
    Ed25519(Arc<ring_sig::Ed25519KeyPair>),
}

impl Clone for SigningKey_ {
    fn clone(&self) -> Self {
        match self {
            Self::Hmac(s) => Self::Hmac(s.clone()),
            Self::Rsa(k) => Self::Rsa(Arc::clone(k)),
            Self::Ed25519(k) => Self::Ed25519(Arc::clone(k)),
        }
    }
}

/// API credentials for authenticated Binance endpoints.
///
/// Supports three authentication methods:
/// - HMAC-SHA256 (default): Most common, uses API secret key
/// - RSA-SHA256: For institutional accounts with RSA key pairs
/// - Ed25519: Modern, fast signature algorithm
///
/// # Examples
///
/// ## HMAC-SHA256 (Default)
/// ```rust
/// use binance_api_client::Credentials;
///
/// let creds = Credentials::new("api_key", "secret_key");
/// ```
///
/// ## RSA-SHA256
/// ```rust,ignore
/// use binance_api_client::Credentials;
///
/// let pem = std::fs::read_to_string("private_key.pem")?;
/// let creds = Credentials::with_rsa_key("api_key", &pem)?;
/// ```
///
/// ## Ed25519
/// ```rust,ignore
/// use binance_api_client::Credentials;
///
/// let private_key_bytes = std::fs::read("ed25519_private_key.der")?;
/// let creds = Credentials::with_ed25519_key("api_key", &private_key_bytes)?;
/// ```
#[derive(Clone)]
pub struct Credentials {
    api_key: String,
    signing_key: SigningKey_,
    signature_type: SignatureType,
}

impl Credentials {
    /// Create new credentials with HMAC-SHA256 signing.
    ///
    /// This is the default and most common authentication method.
    pub fn new(api_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            signing_key: SigningKey_::Hmac(SecretString::from(secret_key.into())),
            signature_type: SignatureType::HmacSha256,
        }
    }

    /// Create credentials with an RSA private key for RSA-SHA256 signing.
    ///
    /// RSA signatures are commonly used for institutional/enterprise API keys.
    /// The private key should be in PKCS#8 PEM format.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key
    /// * `private_key_pem` - RSA private key in PKCS#8 PEM format
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let pem = r#"-----BEGIN PRIVATE KEY-----
    /// MIIEvQIBADANBg...
    /// -----END PRIVATE KEY-----"#;
    ///
    /// let creds = Credentials::with_rsa_key("api_key", pem)?;
    /// ```
    pub fn with_rsa_key(api_key: impl Into<String>, private_key_pem: &str) -> Result<Self> {
        let private_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem).map_err(|e| {
            crate::error::Error::InvalidCredentials(format!("Invalid RSA key: {}", e))
        })?;

        Ok(Self {
            api_key: api_key.into(),
            signing_key: SigningKey_::Rsa(Arc::new(private_key)),
            signature_type: SignatureType::RsaSha256,
        })
    }

    /// Create credentials with an Ed25519 private key.
    ///
    /// Ed25519 is a modern, fast signature algorithm.
    /// The private key should be the raw 32-byte seed or a PKCS#8 DER-encoded key.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key
    /// * `private_key_bytes` - Ed25519 private key bytes (seed or PKCS#8 DER)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // From raw 32-byte seed
    /// let seed: [u8; 32] = [...];
    /// let creds = Credentials::with_ed25519_key("api_key", &seed)?;
    ///
    /// // From PKCS#8 DER file
    /// let der_bytes = std::fs::read("private_key.der")?;
    /// let creds = Credentials::with_ed25519_key("api_key", &der_bytes)?;
    /// ```
    pub fn with_ed25519_key(api_key: impl Into<String>, private_key_bytes: &[u8]) -> Result<Self> {
        let key_pair = if private_key_bytes.len() == 32 {
            // Raw 32-byte seed
            ring_sig::Ed25519KeyPair::from_seed_unchecked(private_key_bytes).map_err(|e| {
                crate::error::Error::InvalidCredentials(format!("Invalid Ed25519 seed: {}", e))
            })?
        } else {
            // PKCS#8 DER-encoded key
            ring_sig::Ed25519KeyPair::from_pkcs8(private_key_bytes).map_err(|e| {
                crate::error::Error::InvalidCredentials(format!(
                    "Invalid Ed25519 PKCS#8 key: {}",
                    e
                ))
            })?
        };

        Ok(Self {
            api_key: api_key.into(),
            signing_key: SigningKey_::Ed25519(Arc::new(key_pair)),
            signature_type: SignatureType::Ed25519,
        })
    }

    /// Create credentials with an Ed25519 private key from a PEM file.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key
    /// * `pem` - Ed25519 private key in PKCS#8 PEM format
    pub fn with_ed25519_pem(api_key: impl Into<String>, pem: &str) -> Result<Self> {
        // Extract the base64-encoded key from PEM format
        let der_bytes = extract_pem_der(pem, "PRIVATE KEY")?;
        Self::with_ed25519_key(api_key, &der_bytes)
    }

    /// Load credentials from environment variables.
    ///
    /// Expects `BINANCE_API_KEY` and `BINANCE_SECRET_KEY` environment variables.
    /// Uses HMAC-SHA256 signing.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("BINANCE_API_KEY")?;
        let secret_key = std::env::var("BINANCE_SECRET_KEY")?;
        Ok(Self::new(api_key, secret_key))
    }

    /// Load credentials from environment variables with custom names.
    ///
    /// Uses HMAC-SHA256 signing.
    pub fn from_env_with_prefix(prefix: &str) -> Result<Self> {
        let api_key = std::env::var(format!("{}_API_KEY", prefix))?;
        let secret_key = std::env::var(format!("{}_SECRET_KEY", prefix))?;
        Ok(Self::new(api_key, secret_key))
    }

    /// Get the API key.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get the signature type being used.
    pub fn signature_type(&self) -> SignatureType {
        self.signature_type
    }

    /// Sign a message using the configured signing method.
    ///
    /// Returns the signature as a hex string for HMAC, or base64 for RSA/Ed25519.
    pub fn sign(&self, message: &str) -> String {
        match &self.signing_key {
            SigningKey_::Hmac(secret) => {
                let key = hmac::Key::new(hmac::HMAC_SHA256, secret.expose_secret().as_bytes());
                let signature = hmac::sign(&key, message.as_bytes());
                hex::encode(signature.as_ref())
            }
            SigningKey_::Rsa(private_key) => {
                let signing_key = SigningKey::<Sha256>::new((**private_key).clone());
                let mut rng = rand::thread_rng();
                let signature = signing_key.sign_with_rng(&mut rng, message.as_bytes());
                BASE64.encode(signature.to_bytes())
            }
            SigningKey_::Ed25519(key_pair) => {
                let signature = key_pair.sign(message.as_bytes());
                BASE64.encode(signature.as_ref())
            }
        }
    }
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("api_key", &self.api_key)
            .field("signature_type", &self.signature_type)
            .field("secret_key", &"[REDACTED]")
            .finish()
    }
}

/// Extract DER bytes from a PEM-encoded string.
fn extract_pem_der(pem: &str, expected_label: &str) -> Result<Vec<u8>> {
    let begin_marker = format!("-----BEGIN {}-----", expected_label);
    let end_marker = format!("-----END {}-----", expected_label);

    let start = pem.find(&begin_marker).ok_or_else(|| {
        crate::error::Error::InvalidCredentials(format!("Missing {} begin marker", expected_label))
    })? + begin_marker.len();

    let end = pem.find(&end_marker).ok_or_else(|| {
        crate::error::Error::InvalidCredentials(format!("Missing {} end marker", expected_label))
    })?;

    let base64_content: String = pem[start..end]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    BASE64
        .decode(&base64_content)
        .map_err(|e| crate::error::Error::InvalidCredentials(format!("Invalid PEM base64: {}", e)))
}

/// Get the current timestamp in milliseconds since Unix epoch.
pub fn get_timestamp() -> Result<u64> {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(duration.as_millis() as u64)
}

/// Build a query string from key-value pairs.
pub fn build_query_string<I, K, V>(params: I) -> String
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    params
        .into_iter()
        .filter(|(k, _)| !k.as_ref().is_empty())
        .map(|(k, v)| format!("{}={}", k.as_ref(), v.as_ref()))
        .collect::<Vec<_>>()
        .join("&")
}

/// Build a signed query string with timestamp and signature.
pub fn build_signed_query_string<I, K, V>(
    params: I,
    credentials: &Credentials,
    recv_window: u64,
) -> Result<String>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let timestamp = get_timestamp()?;

    // Build the base query string
    let mut query_parts: Vec<String> = Vec::new();

    // Add recv_window if specified
    if recv_window > 0 {
        query_parts.push(format!("recvWindow={}", recv_window));
    }

    // Add timestamp
    query_parts.push(format!("timestamp={}", timestamp));

    // Add user params
    for (k, v) in params {
        if !k.as_ref().is_empty() {
            query_parts.push(format!("{}={}", k.as_ref(), v.as_ref()));
        }
    }

    let query_string = query_parts.join("&");

    // Sign and append signature
    let signature = credentials.sign(&query_string);
    Ok(format!("{}&signature={}", query_string, signature))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_new() {
        let creds = Credentials::new("my_api_key", "my_secret_key");
        assert_eq!(creds.api_key(), "my_api_key");
        assert_eq!(creds.signature_type(), SignatureType::HmacSha256);
    }

    #[test]
    fn test_credentials_debug_redacts_secret() {
        let creds = Credentials::new("my_api_key", "my_secret_key");
        let debug_output = format!("{:?}", creds);
        assert!(debug_output.contains("my_api_key"));
        assert!(debug_output.contains("[REDACTED]"));
        assert!(!debug_output.contains("my_secret_key"));
    }

    #[test]
    fn test_sign_hmac() {
        // Test vector: known key and message should produce known signature
        let creds = Credentials::new(
            "api_key",
            "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j",
        );
        let message = "symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559";
        let signature = creds.sign(message);
        // This is the expected signature from Binance's documentation
        assert_eq!(
            signature,
            "c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71"
        );
    }

    #[test]
    fn test_signature_type_default() {
        assert_eq!(SignatureType::default(), SignatureType::HmacSha256);
    }

    #[test]
    fn test_build_query_string() {
        let params = [("symbol", "BTCUSDT"), ("limit", "100")];
        let query = build_query_string(params);
        assert_eq!(query, "symbol=BTCUSDT&limit=100");
    }

    #[test]
    fn test_build_query_string_empty_key_filtered() {
        let params = [("symbol", "BTCUSDT"), ("", "ignored"), ("limit", "100")];
        let query = build_query_string(params);
        assert_eq!(query, "symbol=BTCUSDT&limit=100");
    }

    #[test]
    fn test_get_timestamp() {
        let ts = get_timestamp().unwrap();
        // Timestamp should be reasonable (after Jan 1, 2020 in milliseconds)
        assert!(ts > 1577836800000);
    }

    #[test]
    fn test_build_signed_query_string() {
        let creds = Credentials::new("api_key", "secret_key");
        let params = [("symbol", "BTCUSDT")];
        let query = build_signed_query_string(params, &creds, 5000).unwrap();

        // Should contain recvWindow, timestamp, symbol, and signature
        assert!(query.contains("recvWindow=5000"));
        assert!(query.contains("timestamp="));
        assert!(query.contains("symbol=BTCUSDT"));
        assert!(query.contains("signature="));
    }

    #[test]
    fn test_build_signed_query_string_no_recv_window() {
        let creds = Credentials::new("api_key", "secret_key");
        let params = [("symbol", "BTCUSDT")];
        let query = build_signed_query_string(params, &creds, 0).unwrap();

        // Should NOT contain recvWindow when set to 0
        assert!(!query.contains("recvWindow="));
        assert!(query.contains("timestamp="));
        assert!(query.contains("symbol=BTCUSDT"));
        assert!(query.contains("signature="));
    }

    #[test]
    fn test_ed25519_signing() {
        // Generate a test Ed25519 key pair using ring
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes = ring_sig::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();

        let creds = Credentials::with_ed25519_key("api_key", pkcs8_bytes.as_ref()).unwrap();
        assert_eq!(creds.signature_type(), SignatureType::Ed25519);

        let message = "test message";
        let signature = creds.sign(message);

        // Ed25519 signatures should be base64 encoded
        assert!(BASE64.decode(&signature).is_ok());
    }
}
