use anyhow::{ensure, Result};

use crate::{bytes_to_hex_string, generate_alpanumeric_string, hex_string_to_bytes};

use super::crypto_key::CryptoKey;

#[derive(Debug, PartialEq)]
pub struct AuthToken {
    plain_text: String,
    signature: Vec<u8>,
}

impl AuthToken {
    pub fn generate(key: &CryptoKey) -> Self {
        AuthToken::generate_with_length(key, 256)
    }

    pub fn generate_with_length(key: &CryptoKey, plain_text_length: usize) -> Self {
        let plain_text = generate_alpanumeric_string(plain_text_length);

        let signature = key.sign(plain_text.as_bytes());

        Self {
            plain_text,
            signature: signature.into(),
        }
    }

    pub fn assert_is_valid(&self, key: &CryptoKey) -> Result<()> {
        let is_valid = key.verify_signature(self.plain_text.as_bytes(), &self.signature);

        ensure!(is_valid, "Auth token is invalid");

        Ok(())
    }

    pub fn serialize(&self) -> String {
        format!(
            "{}-{}",
            self.plain_text,
            bytes_to_hex_string(&self.signature)
        )
    }

    pub fn parse(value: &str) -> Result<Self> {
        let parts: Vec<&str> = value.splitn(2, '-').collect();
        ensure!(parts.len() == 2, "Invalid input string");

        let plain_text = parts[0].to_string();
        let signature = hex_string_to_bytes(parts[1])?;

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
