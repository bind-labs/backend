use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, Result, SaltString,
    },
    Argon2,
};

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let argon2 = Argon2::default();
    let hash = PasswordHash::new(hash)?;
    Ok(argon2.verify_password(password.as_bytes(), &hash).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_uses_salt() {
        let password = "my_secure_password";
        let hash = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();
        assert_ne!(hash, hash2);
    }

    #[test]
    fn test_hash_password() {
        let password = "my_secure_password";
        let hash = hash_password(password).unwrap();
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "my_secure_password";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "my_secure_password";
        let wrong_password = "wrong_password";
        let hash = hash_password(password).unwrap();
        assert!(!verify_password(wrong_password, &hash).unwrap());
    }

    #[test]
    fn test_empty_password() {
        let password = "";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_invalid_hash_format() {
        let password = "my_secure_password";
        let invalid_hash = "invalid_hash_format";
        assert!(verify_password(password, invalid_hash).is_err());
    }
}
