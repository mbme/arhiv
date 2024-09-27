use std::io::{Read, Seek, Write};

use anyhow::anyhow;
use chacha20::{
    cipher::{KeyIvInit, StreamCipher, StreamCipherSeek},
    ChaCha20,
};

pub const KEY_SIZE: usize = 32;
pub type Key = [u8; KEY_SIZE];

pub const CHACHA_NONCE_SIZE: usize = 12;
pub type ChaChaNonce = [u8; CHACHA_NONCE_SIZE];

pub enum Encryptor {
    ChaCha20 { cipher: ChaCha20 },
}

impl Encryptor {
    pub fn new_chacha20(key: &Key, nonce: &ChaChaNonce) -> Self {
        let cipher = ChaCha20::new(key.into(), nonce.into());

        Self::ChaCha20 { cipher }
    }

    pub fn encrypt_next(&mut self, chunk: &mut [u8]) -> std::io::Result<()> {
        match self {
            Encryptor::ChaCha20 { cipher } => cipher
                .try_apply_keystream(chunk)
                .map_err(|err| std::io::Error::other(anyhow!("Failed to encrypt chunk: {err}"))),
        }
    }
}

pub enum Decryptor {
    ChaCha20 { cipher: ChaCha20 },
}

impl Decryptor {
    pub fn new_chacha20(key: &Key, nonce: &ChaChaNonce) -> Self {
        let cipher = ChaCha20::new(key.into(), nonce.into());

        Self::ChaCha20 { cipher }
    }

    pub fn decrypt_next(&mut self, chunk: &mut [u8]) -> std::io::Result<()> {
        match self {
            Decryptor::ChaCha20 { cipher } => {
                cipher.try_apply_keystream(chunk).map_err(|err| {
                    std::io::Error::other(anyhow!("Failed to decrypt chunk: {err}"))
                })?;

                Ok(())
            }
        }
    }

    pub fn seek_to(&mut self, offset: u64) -> std::io::Result<()> {
        match self {
            Decryptor::ChaCha20 { cipher } => cipher
                .try_seek(offset)
                .map_err(|err| std::io::Error::other(anyhow!("Failed to seek: {err}"))),
        }
    }
}

pub struct CryptoStreamWriter<InnerWrite: Write> {
    inner: InnerWrite,
    encryptor: Encryptor,
}

impl<InnerWrite: Write> CryptoStreamWriter<InnerWrite> {
    pub fn new(inner: InnerWrite, encryptor: Encryptor) -> Self {
        Self { inner, encryptor }
    }

    pub fn new_chacha20(inner: InnerWrite, key: &Key, nonce: &ChaChaNonce) -> Self {
        let encryptor = Encryptor::new_chacha20(key, nonce);

        Self::new(inner, encryptor)
    }

    pub fn into_inner(self) -> InnerWrite {
        self.inner
    }
}

impl<InnerWrite: Write> Write for CryptoStreamWriter<InnerWrite> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut chunk: Vec<u8> = buf.into();
        self.encryptor.encrypt_next(&mut chunk)?;

        self.inner.write_all(&chunk)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

pub struct CryptoStreamReader<InnerRead: Read> {
    inner: InnerRead,
    decryptor: Decryptor,
}

impl<InnerRead: Read> CryptoStreamReader<InnerRead> {
    pub fn new(inner: InnerRead, decryptor: Decryptor) -> Self {
        Self { inner, decryptor }
    }

    pub fn new_chacha20(inner: InnerRead, key: &Key, nonce: &ChaChaNonce) -> Self {
        let decryptor = Decryptor::new_chacha20(key, nonce);

        Self::new(inner, decryptor)
    }

    pub fn get_inner(&self) -> &InnerRead {
        &self.inner
    }
}

impl<InnerRead: Read> Read for CryptoStreamReader<InnerRead> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.inner.read(buf)?;

        if bytes_read > 0 {
            self.decryptor.decrypt_next(&mut buf[0..bytes_read])?;
        }

        Ok(bytes_read)
    }
}

impl<InnerRead: Read + Seek> Seek for CryptoStreamReader<InnerRead> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let offset = self.inner.seek(pos)?;

        self.decryptor.seek_to(offset)?;

        Ok(offset)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::Result;

    use crate::{generate_bytes, new_random_crypto_byte_array, read_all_as_string};

    use super::*;

    #[test]
    fn test_write_read() -> Result<()> {
        let original_data = generate_bytes(1000);

        let key = [0; KEY_SIZE];
        let nonce = new_random_crypto_byte_array();

        let encrypted_data = {
            let mut writer = CryptoStreamWriter::new_chacha20(Vec::new(), &key, &nonce);

            for chunk in original_data.chunks(100) {
                writer.write_all(chunk)?;
            }

            writer.into_inner()
        };

        let encrypted_size = original_data.len();
        assert_eq!(encrypted_data.len(), encrypted_size);

        let decrypted_data = {
            let mut reader =
                CryptoStreamReader::new_chacha20(Cursor::new(encrypted_data), &key, &nonce);

            let mut data = Vec::new();

            loop {
                let mut buffer = [0; 10];
                let bytes_read = reader.read(&mut buffer)?;

                if bytes_read > 0 {
                    data.extend_from_slice(&buffer[0..bytes_read]);
                } else {
                    break;
                }
            }

            data
        };

        assert_eq!(decrypted_data.len(), original_data.len());
        assert_eq!(decrypted_data, original_data);

        Ok(())
    }

    #[test]
    fn test_seek() -> Result<()> {
        let data = r#"test value
ok go"#;

        let key = [0; KEY_SIZE];
        let nonce = new_random_crypto_byte_array();
        let encrypted_data = {
            let mut writer = CryptoStreamWriter::new_chacha20(Vec::new(), &key, &nonce);
            writer.write_all(data.as_bytes())?;

            writer.into_inner()
        };

        let encrypted_data = Cursor::new(encrypted_data);

        let mut reader = CryptoStreamReader::new_chacha20(encrypted_data, &key, &nonce);

        reader.seek(std::io::SeekFrom::Start(11))?;

        let line = read_all_as_string(reader)?;
        assert_eq!(line, "ok go");

        Ok(())
    }
}
