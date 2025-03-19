use anyhow::{ensure, Result};

use crate::{
    concat_bytes, crypto_key::SIGNATURE_SIZE, decode_url_safe_base64, new_random_crypto_byte_array,
    to_url_safe_base64,
};

use super::crypto_key::{CryptoKey, Signature};

const PLAIN_TEXT_LEN: usize = 6;

type PlainText = [u8; PLAIN_TEXT_LEN];

#[derive(Debug, PartialEq)]
pub struct AuthToken {
    plain_text: PlainText,
    signature: Signature,
}

impl AuthToken {
    pub fn generate(key: &CryptoKey) -> Self {
        let plain_text: PlainText = new_random_crypto_byte_array();

        let signature = key.sign(&plain_text);

        Self {
            plain_text,
            signature,
        }
    }

    pub fn assert_is_valid(&self, key: &CryptoKey) -> Result<()> {
        let is_valid = key.verify_signature(&self.plain_text, &self.signature);

        ensure!(is_valid, "Auth token is invalid");

        Ok(())
    }

    pub fn serialize(&self) -> String {
        to_url_safe_base64(&concat_bytes(&self.plain_text, &self.signature))
    }

    pub fn parse(value: &str) -> Result<Self> {
        let data = decode_url_safe_base64(value)?;

        const AUTH_TOKEN_LEN: usize = PLAIN_TEXT_LEN + SIGNATURE_SIZE;

        ensure!(
            data.len() == AUTH_TOKEN_LEN,
            "Wrong AuthToken len: {} instead of {AUTH_TOKEN_LEN}",
            data.len()
        );

        let (first, second) = data.split_at(PLAIN_TEXT_LEN);

        let plain_text: PlainText = first.try_into().expect("Invalid plain text size");
        let signature: Signature = second.try_into().expect("Invalid signature size");

        Ok(AuthToken {
            plain_text,
            signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{crypto::crypto_key::CryptoKey, AuthToken};

    #[test]
    fn test_auth_token_parse_serialize() -> Result<()> {
        let key =
            CryptoKey::derive_subkey([0; 32].as_slice(), CryptoKey::salt_from_data("test1234")?)?;

        let token = AuthToken::generate(&key);

        let token_str = token.serialize();

        let parsed_token = AuthToken::parse(&token_str).unwrap();

        assert_eq!(parsed_token, token);

        Ok(())
    }
}
