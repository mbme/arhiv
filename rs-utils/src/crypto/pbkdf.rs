use anyhow::{ensure, Result};

use super::SecretBytes;

// Derive secure key from password & salt
pub struct PBKDF {
    key: SecretBytes,
}

impl PBKDF {
    pub const MIN_PASSWORD_LENGTH: usize = 8;
    pub const MIN_SALT_LENGTH: usize = 8;

    pub fn derive(password: &[u8], salt: &str) -> Result<Self> {
        ensure!(
            password.len() >= Self::MIN_PASSWORD_LENGTH,
            "password must consist of at least {} bytes",
            Self::MIN_PASSWORD_LENGTH,
        );

        ensure!(
            salt.len() >= Self::MIN_SALT_LENGTH,
            "salt must consist of at least {} bytes",
            Self::MIN_SALT_LENGTH
        );

        Ok(Self::generate(password, salt))
    }

    fn generate(password: &[u8], salt: &str) -> Self {
        let key = SecretBytes::new(blake3::derive_key(salt, password).into());

        Self { key }
    }

    pub fn get(&self) -> &SecretBytes {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbkdf() -> Result<()> {
        {
            let pbkdf1 = PBKDF::derive(b"12345678", "salt1234")?;
            let pbkdf2 = PBKDF::derive(b"12345678", "salt1234")?;

            assert_eq!(pbkdf1.get().as_bytes(), pbkdf2.get().as_bytes());
        }

        {
            let pbkdf1 = PBKDF::derive(b"12345678", "salt1234")?;
            let pbkdf2 = PBKDF::derive(b"12345678", "salt12345")?;

            assert_ne!(pbkdf1.get().as_bytes(), pbkdf2.get().as_bytes());
        }

        Ok(())
    }
}
