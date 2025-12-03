use hex;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

pub fn verify_github_signature(secret: &str, payload: &[u8], signature: &str) -> bool {
    let signature_hex = match signature.strip_prefix("sha256=") {
        Some(hex) => hex,
        None => return false,
    };

    let signature_bytes = match hex::decode(signature_hex) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };

    mac.update(payload);
    let expected = mac.finalize().into_bytes();

    expected.ct_eq(&signature_bytes[..]).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_valid_signature() {
        let secret = "test_secret";
        let payload = b"test payload";

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let result = mac.finalize();
        let signature = format!("sha256={}", hex::encode(result.into_bytes()));

        assert!(verify_github_signature(secret, payload, &signature));
    }

    #[test]
    fn test_verify_invalid_signature() {
        let secret = "test_secret";
        let payload = b"test payload";
        let signature = "sha256=invalid_signature";

        assert!(!verify_github_signature(secret, payload, signature));
    }

    #[test]
    fn test_verify_missing_prefix() {
        let secret = "test_secret";
        let payload = b"test payload";
        let signature = "invalid_format";

        assert!(!verify_github_signature(secret, payload, signature));
    }
}
