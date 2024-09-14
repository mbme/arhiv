use anyhow::Result;

use super::PBKDF;

// Sign & verify data
pub struct HMAC {
    key: PBKDF,
}

impl HMAC {
    pub fn new_from_password(password: impl AsRef<[u8]>, salt: impl AsRef<str>) -> Result<Self> {
        let key = PBKDF::derive(password.as_ref(), salt.as_ref())?;

        Ok(Self { key })
    }

    pub fn new(key: PBKDF) -> Self {
        Self { key }
    }

    pub fn sign(&self, msg: &[u8]) -> [u8; 32] {
        let key = self
            .key
            .get()
            .as_bytes()
            .try_into()
            .expect("key must have correct length");

        let hash = blake3::keyed_hash(key, msg);

        hash.into()
    }

    pub fn verify(&self, msg: &[u8], tag: &[u8]) -> bool {
        let original_hash = if let Ok(tag) = TryInto::<&[u8; 32]>::try_into(tag) {
            blake3::Hash::from_bytes(*tag)
        } else {
            return false;
        };

        let hash = blake3::Hash::from_bytes(self.sign(msg));

        hash == original_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac() -> Result<()> {
        let hmac = HMAC::new_from_password(b"12345678", "salt1234")?;

        let msg1 = b"message1";
        let tag1 = hmac.sign(msg1);

        let msg2 = b"message2";
        let tag2 = hmac.sign(msg2);

        assert!(hmac.verify(msg1, &tag1));
        assert!(hmac.verify(msg2, &tag2));

        assert!(!hmac.verify(msg2, &tag1));
        assert!(!hmac.verify(msg1, &tag2));

        Ok(())
    }
}
