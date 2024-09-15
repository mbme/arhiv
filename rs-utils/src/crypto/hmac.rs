use anyhow::{ensure, Result};

use super::CryptoKey;

// Sign & verify data
pub struct HMAC {
    key: CryptoKey,
}

impl HMAC {
    pub const KEY_SIZE_IN_BYTES: usize = 32;

    pub fn new(key: CryptoKey) -> Result<Self> {
        ensure!(
            key.len() >= Self::KEY_SIZE_IN_BYTES,
            "Crypto key must be {} bytes long, got {} instead",
            Self::KEY_SIZE_IN_BYTES,
            key.len()
        );

        Ok(Self { key })
    }

    pub fn sign(&self, msg: &[u8]) -> [u8; Self::KEY_SIZE_IN_BYTES] {
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
        let original_hash =
            if let Ok(tag) = TryInto::<&[u8; Self::KEY_SIZE_IN_BYTES]>::try_into(tag) {
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
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_hmac() -> Result<()> {
        let key = CryptoKey::from_crypto_bytes([0; 32].as_slice())?;
        let hmac = HMAC::new(key)?;

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
