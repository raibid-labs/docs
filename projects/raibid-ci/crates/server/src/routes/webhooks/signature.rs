//! Webhook signature verification for GitHub and Gitea

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Verify Gitea webhook signature
///
/// Gitea uses HMAC-SHA256 and sends the signature as a hex string
/// in the X-Gitea-Signature header.
pub fn verify_gitea_signature(payload: &str, signature: &str, secret: &str) -> bool {
    // Create HMAC instance
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };

    // Update with payload
    mac.update(payload.as_bytes());

    // Get expected signature as hex string
    let expected = hex::encode(mac.finalize().into_bytes());

    // Constant-time comparison
    constant_time_compare(&expected, signature)
}

/// Verify GitHub webhook signature
///
/// GitHub uses HMAC-SHA256 and sends the signature as "sha256=<hex>"
/// in the X-Hub-Signature-256 header.
pub fn verify_github_signature(payload: &str, signature: &str, secret: &str) -> bool {
    // GitHub signature format: "sha256=<hex>"
    let signature = match signature.strip_prefix("sha256=") {
        Some(sig) => sig,
        None => return false,
    };

    // Create HMAC instance
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };

    // Update with payload
    mac.update(payload.as_bytes());

    // Get expected signature as hex string
    let expected = hex::encode(mac.finalize().into_bytes());

    // Constant-time comparison
    constant_time_compare(&expected, signature)
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (a_byte, b_byte) in a.bytes().zip(b.bytes()) {
        result |= a_byte ^ b_byte;
    }

    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitea_signature_verification() {
        let payload = r#"{"test": "payload"}"#;
        let secret = "my-secret-key";

        // Generate a valid signature
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        // Verify signature
        assert!(verify_gitea_signature(payload, &signature, secret));

        // Verify invalid signature
        assert!(!verify_gitea_signature(payload, "invalid", secret));

        // Verify wrong secret
        assert!(!verify_gitea_signature(payload, &signature, "wrong-secret"));
    }

    #[test]
    fn test_github_signature_verification() {
        let payload = r#"{"test": "payload"}"#;
        let secret = "my-secret-key";

        // Generate a valid signature
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        // Verify signature
        assert!(verify_github_signature(payload, &signature, secret));

        // Verify invalid signature
        assert!(!verify_github_signature(payload, "sha256=invalid", secret));

        // Verify missing prefix
        let sig_no_prefix = signature.strip_prefix("sha256=").unwrap();
        assert!(!verify_github_signature(payload, sig_no_prefix, secret));

        // Verify wrong secret
        assert!(!verify_github_signature(
            payload,
            &signature,
            "wrong-secret"
        ));
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("abc123", "abc123"));
        assert!(!constant_time_compare("abc123", "abc124"));
        assert!(!constant_time_compare("abc123", "abc12"));
        assert!(!constant_time_compare("abc", "abc123"));
    }

    #[test]
    fn test_signature_verification_with_special_chars() {
        let payload = r#"{"message": "Test with special chars: !@#$%^&*()"}"#;
        let secret = "secret-with-!@#$%";

        // Gitea
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());
        assert!(verify_gitea_signature(payload, &signature, secret));

        // GitHub
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        assert!(verify_github_signature(payload, &signature, secret));
    }
}
