//! Password hashing with Argon2id
//!
//! Argon2id is the recommended algorithm for password hashing.
//! It's resistant to both GPU and side-channel attacks.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Algorithm, Params, Version,
};
use anyhow::Result;

/// Hash a password using Argon2id with secure parameters
///
/// Parameters:
/// - Memory: 64 MB (65536 KB)
/// - Iterations: 3
/// - Parallelism: 4
/// - Output length: 32 bytes
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);

    // Use Argon2id with secure parameters
    let params = Params::new(
        65536,  // 64 MB memory
        3,      // 3 iterations
        4,      // 4 parallel lanes
        Some(32), // 32 byte output
    )?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

    Ok(hash.to_string())
}

/// Verify a password against its hash
pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash));
        assert!(!verify_password("WrongPassword", &hash));
    }

    #[test]
    fn test_different_hashes() {
        let password = "SamePassword";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Same password should produce different hashes (different salt)
        assert_ne!(hash1, hash2);

        // Both should verify
        assert!(verify_password(password, &hash1));
        assert!(verify_password(password, &hash2));
    }
}
