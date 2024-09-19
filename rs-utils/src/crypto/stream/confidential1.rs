/// format: CONFIDENTIAL-1[key-salt:32b][nonce:19b]([chunk:64KB][tag:16b])*[last-chunk:<=64KB][last-tag:16b]
/// 1. password -> read salt & derive key
/// 2. crypto key -> read salt & derive subkey
use core::str;
use std::io::{BufRead, Read, Write};

use anyhow::{ensure, Context, Result};

use crate::{
    crypto::crypto_key::{CryptoKey, Salt, SALT_SIZE},
    new_random_crypto_byte_array, SecretBytes,
};

use super::crypto_stream::{
    get_encrypted_stream_size, CryptoStreamReader, CryptoStreamWriter, X_CHACHA_NONCE_SIZE,
};

pub const CONFIDENTIAL1_MAGIC_STRING: &str = "CONFIDENTIAL-1";
pub const CONFIDENTIAL1_CHUNK_SIZE: usize = 64 * 1024; // 64KB like in AGE crypto format

pub const CONFIDENTIAL1_HEADER_SIZE: usize =
    CONFIDENTIAL1_MAGIC_STRING.as_bytes().len() + SALT_SIZE + X_CHACHA_NONCE_SIZE;

pub enum Confidential1Key {
    Password(SecretBytes),
    Key(CryptoKey),
}

impl Confidential1Key {
    pub fn get_crypto_key(&self, salt: Salt) -> Result<CryptoKey> {
        match self {
            Confidential1Key::Password(password) => {
                CryptoKey::derive_from_password_with_argon2(password, salt)
            }
            Confidential1Key::Key(key) => CryptoKey::derive_subkey(key.get(), salt),
        }
    }
}

pub fn create_confidential1_reader(
    mut reader: impl Read,
    confidential1_key: &Confidential1Key,
) -> Result<impl BufRead> {
    let mut magic_string = [0; CONFIDENTIAL1_MAGIC_STRING.as_bytes().len()];
    reader
        .read_exact(&mut magic_string)
        .context("Failed to read magic string bytes")?;

    let magic_string = str::from_utf8(&magic_string)
        .context("Failed to interpret magic string bytes as a UTF-8 string")?;

    ensure!(
        magic_string == CONFIDENTIAL1_MAGIC_STRING,
        "Invalid magic string: {magic_string}"
    );

    let mut salt = [0; SALT_SIZE];
    reader
        .read_exact(&mut salt)
        .context("Failed to read salt")?;

    let mut nonce = [0; X_CHACHA_NONCE_SIZE];
    reader
        .read_exact(&mut nonce)
        .context("failed to read nonce")?;

    let key = confidential1_key.get_crypto_key(salt)?;

    let confidential_reader = CryptoStreamReader::new_xchacha20poly1305(
        reader,
        key.get(),
        &nonce,
        CONFIDENTIAL1_CHUNK_SIZE,
    );

    Ok(confidential_reader)
}

pub fn create_confidential1_writer<W: Write>(
    mut writer: W,
    confidential1_key: &Confidential1Key,
) -> Result<CryptoStreamWriter<W>> {
    let salt = CryptoKey::random_salt();
    let key = confidential1_key.get_crypto_key(salt)?;
    let nonce = new_random_crypto_byte_array();

    writer
        .write_all(CONFIDENTIAL1_MAGIC_STRING.as_bytes())
        .context("Failed to write magic string")?;

    writer
        .write_all(key.get_salt())
        .context("Failed to write salt")?;

    writer.write_all(&nonce).context("Failed to write nonce")?;

    let writer = CryptoStreamWriter::new_xchacha20poly1305(
        writer,
        key.get(),
        &nonce,
        CONFIDENTIAL1_CHUNK_SIZE,
    );

    Ok(writer)
}

pub fn get_confidential1_stream_size(data_size: usize) -> usize {
    CONFIDENTIAL1_HEADER_SIZE + get_encrypted_stream_size(data_size, CONFIDENTIAL1_CHUNK_SIZE)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::Result;

    use crate::{generate_alpanumeric_string, new_random_crypto_byte_array};

    use super::*;

    #[test]
    fn test_write_read() -> Result<()> {
        let chunks_count = 2;
        let data = generate_alpanumeric_string(chunks_count * CONFIDENTIAL1_CHUNK_SIZE);
        let key = Confidential1Key::Key(CryptoKey::new(
            new_random_crypto_byte_array(),
            CryptoKey::random_salt(),
        ));

        let encrypted = {
            let mut writer = create_confidential1_writer(Vec::new(), &key)?;
            writer.write_all(data.as_bytes())?;
            writer.finish()?
        };
        let expected_len = get_confidential1_stream_size(data.len());

        assert_eq!(encrypted.len(), expected_len);

        let mut reader = create_confidential1_reader(Cursor::new(encrypted), &key)?;
        let mut decrypted = String::new();
        reader.read_to_string(&mut decrypted)?;

        assert_eq!(decrypted, data);

        Ok(())
    }
}
