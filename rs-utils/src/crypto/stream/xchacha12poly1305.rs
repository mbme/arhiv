use std::io::{BufRead, Read, Write};

use anyhow::{anyhow, Result};
use chacha20poly1305::{
    aead::stream::{DecryptorBE32, EncryptorBE32},
    KeyInit, XChaCha12Poly1305,
};
use rand::{rngs::OsRng, RngCore};

// key derivation salt size is 32b
// chunk size is 64kb; chunk tag size is 16b
// v1[salt][nonce]...([chunk][tag])[last chunk][last tag]

pub const KEY_SIZE: usize = 32;

/// The last 5 bytes of the nonce are used for a 32 bits counter, and a 1-byte "last block" flag,
/// so we only need to generate 19 (or in general nonce_size - 5) bytes of random data.
const NONCE_SIZE: usize = 19;

/// 16b https://docs.rs/chacha20poly1305/latest/chacha20poly1305/type.Tag.html
const CHUNK_TAG_SIZE: usize = 16;

/// The original “Rogaway-flavored” STREAM as described in the paper "Online Authenticated-Encryption and its Nonce-Reuse Misuse-Resistance".
/// Uses a 32-bit big endian counter and 1-byte “last block” flag stored as the last 5-bytes of the AEAD nonce.
pub struct XChaCha12Poly1305Writer<InnerWrite: Write> {
    inner: InnerWrite,
    encryptor: EncryptorBE32<XChaCha12Poly1305>,
    buffer: Vec<u8>,
    chunk_size: usize,
}

impl<InnerWrite: Write> XChaCha12Poly1305Writer<InnerWrite> {
    #[must_use]
    pub fn generate_nonce() -> [u8; NONCE_SIZE] {
        let mut nonce = [0u8; 19];
        // We can safely generate random nonce for XChaCha (ChaCha with eXtended nonce)
        OsRng.fill_bytes(&mut nonce);

        nonce
    }

    pub fn new(
        inner: InnerWrite,
        key: &[u8; KEY_SIZE],
        nonce: &[u8; NONCE_SIZE],
        chunk_size: usize,
    ) -> Self {
        assert!(chunk_size > 0, "chunk size must not be 0");

        let aead = XChaCha12Poly1305::new(key.into());
        let encryptor = EncryptorBE32::from_aead(aead, nonce.into());

        Self {
            inner,
            encryptor,
            buffer: Vec::with_capacity(chunk_size),
            chunk_size,
        }
    }

    fn encrypt_and_write_chunks(&mut self) -> std::io::Result<()> {
        // leave last chunk for the .finalize() call
        while self.buffer.len() > self.chunk_size {
            let chunk = self.buffer.drain(0..self.chunk_size).collect::<Vec<_>>();

            let encrypted_chunk = self
                .encryptor
                .encrypt_next(chunk.as_slice())
                .map_err(|err| std::io::Error::other(anyhow!("Failed to encrypt chunk: {err}")))?;

            self.inner.write_all(&encrypted_chunk)?;
        }

        Ok(())
    }

    /// NOTE: finalize **MUST** be called to correctly complete the encryption
    pub fn finalize(mut self) -> Result<InnerWrite> {
        self.encrypt_and_write_chunks()?;

        let encrypted_chunk = self
            .encryptor
            .encrypt_last(self.buffer.as_slice())
            .map_err(|err| std::io::Error::other(anyhow!("Failed to encrypt last chunk: {err}")))?;

        self.inner.write_all(&encrypted_chunk)?;
        self.inner.flush()?;

        Ok(self.inner)
    }
}

impl<InnerWrite: Write> Write for XChaCha12Poly1305Writer<InnerWrite> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        self.encrypt_and_write_chunks()?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

pub struct XChaCha12Poly1305Reader<InnerRead: Read> {
    inner: InnerRead,
    decryptor: Option<DecryptorBE32<XChaCha12Poly1305>>,
    encrypted_buffer: Vec<u8>,
    decrypted_buffer: Vec<u8>,
    is_finished: bool,
    encrypted_chunk_size: usize,
}

impl<InnerRead: Read> XChaCha12Poly1305Reader<InnerRead> {
    pub fn new(
        inner: InnerRead,
        key: &[u8; KEY_SIZE],
        nonce: &[u8; NONCE_SIZE],
        chunk_size: usize,
    ) -> Self {
        assert!(chunk_size > 0, "chunk size must not be 0");

        let aead = XChaCha12Poly1305::new(key.into());
        let decryptor = DecryptorBE32::from_aead(aead, nonce.into());

        let encrypted_chunk_size = chunk_size + CHUNK_TAG_SIZE;
        Self {
            inner,
            decryptor: Some(decryptor),
            encrypted_buffer: Vec::with_capacity(encrypted_chunk_size * 2),
            decrypted_buffer: Vec::with_capacity(encrypted_chunk_size * 2),
            is_finished: false,
            encrypted_chunk_size,
        }
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

    fn fill_encrypted_buffer(&mut self) -> std::io::Result<()> {
        if self.is_finished {
            return Ok(());
        }

        let buffer_size = self.encrypted_chunk_size * 2;
        let mut chunk = vec![0u8; buffer_size];

        while self.encrypted_buffer.len() < buffer_size {
            // TODO read directly into encrypted_buffer
            let read_bytes = self.inner.read(&mut chunk)?;

            if read_bytes == 0 {
                self.is_finished = true;
                break;
            }

            self.encrypted_buffer
                .extend_from_slice(&chunk[0..read_bytes]);
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
                let last_decryptor = self
                    .decryptor
                    .take()
                    .expect("decryptor must be available for last chunk");

                let decrypted_chunk = last_decryptor
                    .decrypt_last(self.encrypted_buffer.as_slice())
                    .map_err(|err| {
                        std::io::Error::other(anyhow!("Failed to decrypt last chunk: {err}"))
                    })?;

                self.decrypted_buffer.extend_from_slice(&decrypted_chunk);
                self.encrypted_buffer.clear();

                return Ok(());
            }

            let encrypted_chunk = self
                .encrypted_buffer
                .drain(0..self.encrypted_chunk_size)
                .collect::<Vec<_>>();

            let decryptor = self
                .decryptor
                .as_mut()
                .expect("decryptor must be available");

            let decrypted_chunk = decryptor
                .decrypt_next(encrypted_chunk.as_slice())
                .map_err(|err| std::io::Error::other(anyhow!("Failed to decrypt chunk: {err}")))?;

            self.decrypted_buffer.extend_from_slice(&decrypted_chunk);
        }

        // TODO benchmark this
        // let chunk = self
        //     .encrypted_buffer
        //     .split_off(X_CHACHA_12_POLY_1305_ENCRYPTED_CHUNK_SIZE);
        // let chunk = std::mem::replace(&mut self.encrypted_buffer, chunk);
    }
}

impl<InnerRead: Read> Read for XChaCha12Poly1305Reader<InnerRead> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.decrypted_buffer.len() >= buf.len() {
            let len = self.feed_decrypted_data(buf);
            return Ok(len);
        }

        // TODO try to read (buf.len() / chunk_size).ceil() chunks?
        self.fill_encrypted_buffer()?;
        self.decrypt_buffered_data()?;

        let len = self.feed_decrypted_data(buf);

        Ok(len)
    }
}

impl<InnerRead: Read> BufRead for XChaCha12Poly1305Reader<InnerRead> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.fill_encrypted_buffer()?;
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

    use crate::{generate_alpanumeric_string, new_random_byte_array};

    use super::*;

    fn encrypt_decrypt(original_data: &[u8], chunk_size: usize) -> Result<()> {
        let chunks_count = (original_data.len() as f64 / chunk_size as f64).ceil() as usize;

        let key = [0; KEY_SIZE];
        let nonce = new_random_byte_array();

        let encrypted_data = {
            let data = Cursor::new(Vec::new());

            let mut writer = XChaCha12Poly1305Writer::new(data, &key, &nonce, chunk_size);

            for chunk in original_data.chunks(100) {
                writer.write_all(chunk)?;
            }

            let mut inner = writer.finalize()?;

            inner.set_position(0);

            inner
        };

        let overhead = encrypted_data.get_ref().len() - original_data.len();
        assert_eq!(overhead, chunks_count * CHUNK_TAG_SIZE);

        let decrypted_data = {
            let mut reader = XChaCha12Poly1305Reader::new(encrypted_data, &key, &nonce, chunk_size);

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
        let nonce = new_random_byte_array();
        let chunk_size = 5;
        let encrypted_data = {
            let mut writer = XChaCha12Poly1305Writer::new(Vec::new(), &key, &nonce, chunk_size);
            writer.write_all(data.as_bytes())?;

            writer.finalize()?
        };

        let mut reader =
            XChaCha12Poly1305Reader::new(encrypted_data.as_slice(), &key, &nonce, chunk_size);

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
}
