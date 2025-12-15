//! Authentication module
//!
//! Implements enterprise-grade auth:
//! - Argon2id password hashing
//! - JWT access tokens (15 min)
//! - Refresh token rotation (30 days)
//! - Device binding
//! - Token versioning for mass invalidation

mod password;
mod jwt;
mod tokens;
mod middleware;

pub use password::{hash_password, verify_password};
pub use jwt::{create_access_token, decode_access_token, Claims};
pub use tokens::{create_refresh_token, verify_refresh_token, rotate_refresh_token};
pub use middleware::{auth_middleware, AuthUser};
