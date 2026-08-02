use anyhow::{Result, ensure};

use baza_common::{decode_url_safe_base64, new_random_crypto_byte_array, to_url_safe_base64};

const TOKEN_LEN: usize = 32;

pub type Token = [u8; TOKEN_LEN];

#[derive(Clone, PartialEq)]
pub struct AuthToken {
    token: Token,
}

impl AuthToken {
    pub fn generate() -> Self {
        let token: Token = new_random_crypto_byte_array();

        Self { token }
    }

    pub fn serialize(&self) -> String {
        to_url_safe_base64(&self.token)
    }

    pub fn parse(value: &str) -> Result<Self> {
        let data = decode_url_safe_base64(value)?;

        ensure!(
            data.len() == TOKEN_LEN,
            "Wrong AuthToken len: {} instead of {TOKEN_LEN}",
            data.len()
        );

        let token: Token = data.try_into().expect("Invalid AuthToken size");

        Ok(AuthToken { token })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::server::AuthToken;

    #[test]
    fn test_auth_token_parse_serialize() -> Result<()> {
        let token = AuthToken::generate();

        let token_str = token.serialize();

        let parsed_token = AuthToken::parse(&token_str).unwrap();

        assert!(parsed_token == token);

        Ok(())
    }
}
