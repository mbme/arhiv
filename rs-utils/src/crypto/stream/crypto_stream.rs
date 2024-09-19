use std::io::{BufRead, Read, Write};

use anyhow::{anyhow, Result};
use chacha20poly1305::{
    aead::stream::{DecryptorBE32, EncryptorBE32},
    KeyInit, XChaCha20Poly1305,
};

pub const KEY_SIZE: usize = 32;
pub type Key = [u8; KEY_SIZE];

/// The last 5 bytes of the nonce are used for a 32 bits counter, and a 1-byte "last block" flag,
/// so we only need to generate 19 (or in general nonce_size - 5) bytes of random data.
pub const X_CHACHA_NONCE_SIZE: usize = /* 24 - 5 */ 19;
pub type XChaChaNonce = [u8; X_CHACHA_NONCE_SIZE];

/// 16b https://docs.rs/chacha20poly1305/latest/chacha20poly1305/type.Tag.html
pub const X_CHACHA_CHUNK_TAG_SIZE: usize = 16;

pub fn get_encrypted_chunks_count(data_size: usize, chunk_size: usize) -> usize {
    (data_size as f64 / chunk_size as f64).ceil() as usize
}

pub fn get_encrypted_stream_size(data_size: usize, chunk_size: usize) -> usize {
    data_size + get_encrypted_chunks_count(data_size, chunk_size) * X_CHACHA_CHUNK_TAG_SIZE
}

pub enum Encryptor {
    XChaCha20Poly1305(EncryptorBE32<XChaCha20Poly1305>),
}

impl Encryptor {
    pub fn new_xchacha20poly1305(key: &Key, nonce: &XChaChaNonce) -> Self {
        let aead = XChaCha20Poly1305::new(key.into());
        let encryptor = EncryptorBE32::from_aead(aead, nonce.into());

        Self::XChaCha20Poly1305(encryptor)
    }

    pub fn encrypt_next(&mut self, chunk: &[u8]) -> std::io::Result<Vec<u8>> {
        match self {
            Encryptor::XChaCha20Poly1305(encryptor) => encryptor
                .encrypt_next(chunk)
                .map_err(|err| std::io::Error::other(anyhow!("Failed to encrypt chunk: {err}"))),
        }
    }

    pub fn encrypt_last(self, chunk: &[u8]) -> std::io::Result<Vec<u8>> {
        match self {
            Encryptor::XChaCha20Poly1305(encryptor) => {
                encryptor.encrypt_last(chunk).map_err(|err| {
                    std::io::Error::other(anyhow!("Failed to encrypt last chunk: {err}"))
                })
            }
        }
    }
}

pub enum Decryptor {
    XChaCha20Poly1305(Option<DecryptorBE32<XChaCha20Poly1305>>),
}

impl Decryptor {
    pub fn new_xchacha20poly1305(key: &Key, nonce: &XChaChaNonce) -> Self {
        let aead = XChaCha20Poly1305::new(key.into());
        let decryptor = DecryptorBE32::from_aead(aead, nonce.into());

        Self::XChaCha20Poly1305(Some(decryptor))
    }

    pub fn decrypt_next(&mut self, chunk: &[u8]) -> std::io::Result<Vec<u8>> {
        match self {
            Decryptor::XChaCha20Poly1305(decryptor) => {
                if let Some(decryptor) = decryptor {
                    decryptor.decrypt_next(chunk).map_err(|err| {
                        std::io::Error::other(anyhow!("Failed to decrypt chunk: {err}"))
                    })
                } else {
                    Err(std::io::Error::other(anyhow!("Decryptor not available")))
                }
            }
        }
    }

    pub fn decrypt_last(&mut self, chunk: &[u8]) -> std::io::Result<Vec<u8>> {
        match self {
            Decryptor::XChaCha20Poly1305(decryptor) => {
                if let Some(decryptor) = decryptor.take() {
                    decryptor.decrypt_last(chunk).map_err(|err| {
                        std::io::Error::other(anyhow!("Failed to decrypt last chunk: {err}"))
                    })
                } else {
                    Err(std::io::Error::other(anyhow!("Decryptor not available")))
                }
            }
        }
    }
}

/// The original “Rogaway-flavored” STREAM as described in the paper "Online Authenticated-Encryption and its Nonce-Reuse Misuse-Resistance".
/// Uses a 32-bit big endian counter and 1-byte “last block” flag stored as the last 5-bytes of the AEAD nonce.
pub struct CryptoStreamWriter<InnerWrite: Write> {
    inner: InnerWrite,
    encryptor: Encryptor,
    buffer: Vec<u8>,
    chunk_size: usize,
}

impl<InnerWrite: Write> CryptoStreamWriter<InnerWrite> {
    pub fn new(inner: InnerWrite, encryptor: Encryptor, chunk_size: usize) -> Self {
        assert!(chunk_size > 0, "chunk size must not be 0");

        Self {
            inner,
            encryptor,
            buffer: Vec::with_capacity(chunk_size),
            chunk_size,
        }
    }

    pub fn new_xchacha20poly1305(
        inner: InnerWrite,
        key: &Key,
        nonce: &XChaChaNonce,
        chunk_size: usize,
    ) -> Self {
        let encryptor = Encryptor::new_xchacha20poly1305(key, nonce);

        Self::new(inner, encryptor, chunk_size)
    }

    fn encrypt_and_write_chunks(&mut self) -> std::io::Result<()> {
        // leave last chunk for the .finish() call
        while self.buffer.len() > self.chunk_size {
            let chunk = &self.buffer[0..self.chunk_size];

            let encrypted_chunk = self.encryptor.encrypt_next(chunk)?;

            self.inner.write_all(&encrypted_chunk)?;

            // remove the chunk from the buffer
            self.buffer.drain(0..self.chunk_size);
        }

        Ok(())
    }

    /// NOTE: finish() **MUST** be called to correctly complete the encryption
    pub fn finish(mut self) -> Result<InnerWrite> {
        self.encrypt_and_write_chunks()?;

        let encrypted_chunk = self.encryptor.encrypt_last(self.buffer.as_slice())?;

        self.inner.write_all(&encrypted_chunk)?;
        self.inner.flush()?;

        Ok(self.inner)
    }
}

impl<InnerWrite: Write> Write for CryptoStreamWriter<InnerWrite> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        self.encrypt_and_write_chunks()?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

pub struct CryptoStreamReader<InnerRead: Read> {
    inner: InnerRead,
    decryptor: Decryptor,
    encrypted_buffer: Vec<u8>,
    decrypted_buffer: Vec<u8>,
    is_finished: bool,
    chunk_size: usize,
    encrypted_chunk_size: usize,
}

impl<InnerRead: Read> CryptoStreamReader<InnerRead> {
    pub fn new(inner: InnerRead, decryptor: Decryptor, chunk_size: usize) -> Self {
        assert!(chunk_size > 0, "chunk size must not be 0");

        let encrypted_chunk_size = chunk_size + X_CHACHA_CHUNK_TAG_SIZE;

        Self {
            inner,
            decryptor,
            encrypted_buffer: Vec::with_capacity(encrypted_chunk_size * 2),
            decrypted_buffer: Vec::with_capacity(encrypted_chunk_size * 2),
            is_finished: false,
            chunk_size,
            encrypted_chunk_size,
        }
    }

    pub fn new_xchacha20poly1305(
        inner: InnerRead,
        key: &Key,
        nonce: &XChaChaNonce,
        chunk_size: usize,
    ) -> Self {
        let decryptor = Decryptor::new_xchacha20poly1305(key, nonce);

        Self::new(inner, decryptor, chunk_size)
    }

    fn feed_decrypted_data(&mut self, buf: &mut [u8]) -> usize {
        if self.decrypted_buffer.is_empty() {
            return 0;
        }

        let len = buf.len().min(self.decrypted_buffer.len());

        buf[0..len].copy_from_slice(&self.decrypted_buffer[0..len]);

        self.decrypted_buffer.drain(0..len);

        len
    }

    fn fill_encrypted_buffer(
        &mut self,
        desired_chunks_count: Option<usize>,
    ) -> std::io::Result<()> {
        if self.is_finished {
            return Ok(());
        }

        // read at least 2 chunks or more to be able to determine the last chunk
        let buffer_size =
            std::cmp::max(2, desired_chunks_count.unwrap_or_default()) * self.encrypted_chunk_size;

        let mut chunk = vec![0u8; buffer_size];

        while self.encrypted_buffer.len() < buffer_size {
            let read_bytes = self.inner.read(&mut chunk)?;

            self.encrypted_buffer
                .extend_from_slice(&chunk[0..read_bytes]);

            if read_bytes == 0 {
                self.is_finished = true;
                break;
            }
        }

        Ok(())
    }

    fn decrypt_buffered_data(&mut self) -> std::io::Result<()> {
        loop {
            if self.encrypted_buffer.is_empty() {
                return Ok(());
            }

            let has_many_chunks = self.encrypted_buffer.len() > self.encrypted_chunk_size;
            let is_last_chunk = self.is_finished && !has_many_chunks;

            // not enough data
            if !(has_many_chunks || self.is_finished) {
                return Ok(());
            }

            if is_last_chunk {
                let decrypted_chunk = self
                    .decryptor
                    .decrypt_last(self.encrypted_buffer.as_slice())?;

                self.decrypted_buffer.extend_from_slice(&decrypted_chunk);
                self.encrypted_buffer.clear();

                return Ok(());
            }

            let encrypted_chunk = &self.encrypted_buffer[0..self.encrypted_chunk_size];

            let decrypted_chunk = self.decryptor.decrypt_next(encrypted_chunk)?;

            // remove the chunk from the buffer
            self.encrypted_buffer.drain(0..self.encrypted_chunk_size);

            self.decrypted_buffer.extend_from_slice(&decrypted_chunk);
        }
    }
}

impl<InnerRead: Read> Read for CryptoStreamReader<InnerRead> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.decrypted_buffer.len() >= buf.len() {
            let len = self.feed_decrypted_data(buf);
            return Ok(len);
        }

        // attempt to read enough chunks to fill the whole buf
        let desired_chunks_count =
            (buf.len() - self.decrypted_buffer.len()).div_ceil(self.chunk_size);
        self.fill_encrypted_buffer(Some(desired_chunks_count))?;

        self.decrypt_buffered_data()?;

        let len = self.feed_decrypted_data(buf);

        Ok(len)
    }
}

impl<InnerRead: Read> BufRead for CryptoStreamReader<InnerRead> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.fill_encrypted_buffer(None)?;
        self.decrypt_buffered_data()?;

        Ok(&self.decrypted_buffer)
    }

    fn consume(&mut self, amt: usize) {
        self.decrypted_buffer.drain(0..amt);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::Result;

    use crate::{generate_alpanumeric_string, new_random_crypto_byte_array};

    use super::*;

    fn encrypt_decrypt(original_data: &[u8], chunk_size: usize) -> Result<()> {
        let key = [0; KEY_SIZE];
        let nonce = new_random_crypto_byte_array();

        let encrypted_data = {
            let data = Cursor::new(Vec::new());

            let mut writer =
                CryptoStreamWriter::new_xchacha20poly1305(data, &key, &nonce, chunk_size);

            for chunk in original_data.chunks(100) {
                writer.write_all(chunk)?;
            }

            let mut inner = writer.finish()?;

            inner.set_position(0);

            inner
        };

        let encrypted_size = get_encrypted_stream_size(original_data.len(), chunk_size);
        assert_eq!(encrypted_data.get_ref().len(), encrypted_size);

        let decrypted_data = {
            let mut reader =
                CryptoStreamReader::new_xchacha20poly1305(encrypted_data, &key, &nonce, chunk_size);

            let mut data = Vec::new();

            loop {
                let mut buffer = [0; 100];
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
    fn test_write_read() -> Result<()> {
        {
            let chunk_size = 20;
            let chunks_count = 10;
            let original_data = generate_alpanumeric_string(chunks_count * chunk_size).into_bytes();

            encrypt_decrypt(&original_data, chunk_size)?;
        }

        {
            let chunk_size = 20;
            let chunks_count = 10;
            let original_data =
                generate_alpanumeric_string(chunks_count * chunk_size + chunk_size / 2)
                    .into_bytes();

            encrypt_decrypt(&original_data, chunk_size)?;
        }

        {
            let chunk_size = 20;
            let chunks_count = 1;
            let original_data = generate_alpanumeric_string(chunks_count * chunk_size).into_bytes();

            encrypt_decrypt(&original_data, chunk_size)?;
        }

        Ok(())
    }

    #[test]
    fn test_buf_read() -> Result<()> {
        let data = r#"test value
ok go"#;

        let key = [0; KEY_SIZE];
        let nonce = new_random_crypto_byte_array();
        let chunk_size = 5;
        let encrypted_data = {
            let mut writer =
                CryptoStreamWriter::new_xchacha20poly1305(Vec::new(), &key, &nonce, chunk_size);
            writer.write_all(data.as_bytes())?;

            writer.finish()?
        };

        let mut reader = CryptoStreamReader::new_xchacha20poly1305(
            encrypted_data.as_slice(),
            &key,
            &nonce,
            chunk_size,
        );

        let mut line = String::new();
        reader.read_line(&mut line)?;

        assert_eq!(line, "test value\n");

        line.clear();

        reader.read_line(&mut line)?;
        assert_eq!(line, "ok go");

        let bytes_read = reader.read_line(&mut line)?;
        assert_eq!(bytes_read, 0);

        Ok(())
    }

    #[test]
    fn test_buf_read_long() -> Result<()> {
        const CHUNK_SIZE: usize = 10;
        let data = generate_alpanumeric_string(CHUNK_SIZE * 1000);

        let key = [0; KEY_SIZE];
        let nonce = new_random_crypto_byte_array();
        let encrypted_data = {
            let mut writer =
                CryptoStreamWriter::new_xchacha20poly1305(Vec::new(), &key, &nonce, CHUNK_SIZE);
            writer.write_all(data.as_bytes())?;

            writer.finish()?
        };

        let mut reader = CryptoStreamReader::new_xchacha20poly1305(
            encrypted_data.as_slice(),
            &key,
            &nonce,
            CHUNK_SIZE,
        );

        let mut line = String::new();
        reader.read_line(&mut line)?;

        assert_eq!(line, data);

        Ok(())
    }
}
