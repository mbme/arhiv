/// format: CONFIDENTIAL-1[key-salt:32b][nonce:12b][chacha20-encrypted-data][sha256-hash:32b]
/// 1. password -> read salt & derive key using Argon2id v19 (m=19456 (19 MiB), t=2, p=1)
/// 2. crypto key -> read salt & derive subkey using HKDF-sha256
use core::str;
use std::io::{self, Read, Seek, SeekFrom, Write};

use anyhow::{anyhow, ensure, Context, Result};

use crate::{
    crypto::crypto_key::{CryptoKey, Salt, SALT_SIZE},
    new_random_crypto_byte_array, ReaderWithHeader, ReaderWithTrailer, SecretBytes, Sha256Hash,
    Sha256HashingReader, Sha256HashingWriter, SHA256_HASH_SIZE,
};

use super::crypto_stream::{CryptoStreamReader, CryptoStreamWriter, CHACHA_NONCE_SIZE};

pub const CONFIDENTIAL1_MAGIC_STRING: &str = "CONFIDENTIAL-1";
pub const CONFIDENTIAL1_MAGIC_STRING_LEN: usize = CONFIDENTIAL1_MAGIC_STRING.len();

pub const CONFIDENTIAL1_HEADER_SIZE: usize =
    CONFIDENTIAL1_MAGIC_STRING_LEN + SALT_SIZE + CHACHA_NONCE_SIZE;

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

    pub fn new_random_key() -> Self {
        Self::new(CryptoKey::new_random_key())
    }

    pub fn new(key: CryptoKey) -> Self {
        Confidential1Key::Key(key)
    }

    pub fn from_key_and_salt(key: &CryptoKey, salt: Salt) -> Result<Self> {
        let subkey = CryptoKey::derive_subkey(key.get(), salt)?;

        Ok(Self::new(subkey))
    }
}

pub struct Confidential1Reader<R: Read> {
    inner: CryptoStreamReader<
        ReaderWithHeader<CONFIDENTIAL1_HEADER_SIZE, ReaderWithTrailer<SHA256_HASH_SIZE, R>>,
    >,
}

impl<R: Read> Confidential1Reader<R> {
    pub fn new(reader: R, confidential1_key: &Confidential1Key) -> Result<Self> {
        let reader = ReaderWithTrailer::new(reader);
        let (reader, header) = ReaderWithHeader::new(reader)?;

        let magic_string = str::from_utf8(&header[0..CONFIDENTIAL1_MAGIC_STRING_LEN])
            .context("Failed to interpret magic string bytes as a UTF-8 string")?;

        ensure!(
            magic_string == CONFIDENTIAL1_MAGIC_STRING,
            "Invalid magic string: {magic_string}"
        );

        let mut salt = [0; SALT_SIZE];
        let salt_start = CONFIDENTIAL1_MAGIC_STRING_LEN;
        let salt_end = salt_start + SALT_SIZE;
        salt.copy_from_slice(&header[salt_start..salt_end]);

        let mut nonce = [0; CHACHA_NONCE_SIZE];
        let nonce_start = salt_end;
        let nonce_end = nonce_start + CHACHA_NONCE_SIZE;
        nonce.copy_from_slice(&header[nonce_start..nonce_end]);

        let key = confidential1_key.get_crypto_key(salt)?;

        let crypto_reader = CryptoStreamReader::new_chacha20(reader, key.get(), &nonce);

        Ok(Self {
            inner: crypto_reader,
        })
    }

    pub fn get_hash(&self) -> Option<&Sha256Hash> {
        self.inner.get_inner().get_inner().get_trailer()
    }
}

impl<R: Read> Read for Confidential1Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.inner.read(buf)?;

        Ok(bytes_read)
    }
}

impl<R: Read + Seek> Seek for Confidential1Reader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

pub struct Confidential1AuthReader<R: Read> {
    inner: Sha256HashingReader<Confidential1Reader<R>>,
}

impl<R: Read> Confidential1AuthReader<R> {
    pub fn new(reader: R, confidential1_key: &Confidential1Key) -> Result<Self> {
        let c1_reader = Confidential1Reader::new(reader, confidential1_key)?;
        let hashing_reader = Sha256HashingReader::new(c1_reader);

        Ok(Self {
            inner: hashing_reader,
        })
    }

    pub fn is_authentic(&self) -> Option<bool> {
        let computed_hash = self.inner.get_hash()?;
        let trailer_hash = self.inner.get_inner().get_hash()?;

        Some(computed_hash == trailer_hash)
    }
}

impl<R: Read> Read for Confidential1AuthReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.inner.read(buf)?;

        if bytes_read == 0 && !self.is_authentic().unwrap_or_default() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                anyhow!("Stream hash doesn't match"),
            ));
        }

        Ok(bytes_read)
    }
}

pub struct Confidential1Writer<W: Write> {
    inner: Sha256HashingWriter<CryptoStreamWriter<W>>,
}

impl<W: Write> Confidential1Writer<W> {
    pub fn new(mut writer: W, confidential1_key: &Confidential1Key) -> Result<Self> {
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

        let crypto_writer = CryptoStreamWriter::new_chacha20(writer, key.get(), &nonce);

        let hashing_writer = Sha256HashingWriter::new(crypto_writer);

        Ok(Self {
            inner: hashing_writer,
        })
    }

    pub fn finish(self) -> Result<W> {
        let (hash, crypto_writer) = self.inner.finish();

        let mut writer = crypto_writer.into_inner();

        writer.write_all(&hash).context("Failed to write hash")?;

        writer.flush()?;

        Ok(writer)
    }
}

impl<W: Write> Write for Confidential1Writer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

pub fn get_confidential1_stream_size(data_size: usize) -> usize {
    CONFIDENTIAL1_HEADER_SIZE + data_size + SHA256_HASH_SIZE
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::Result;

    use crate::{generate_alpanumeric_string, read_all_as_string};

    use super::*;

    #[test]
    fn test_write_read() -> Result<()> {
        let data = generate_alpanumeric_string(100 * 1024);
        let key = Confidential1Key::new_random_key();

        let encrypted = {
            let mut writer = Confidential1Writer::new(Vec::new(), &key)?;
            writer.write_all(data.as_bytes())?;
            writer.finish()?
        };
        let expected_len = get_confidential1_stream_size(data.len());

        assert_eq!(encrypted.len(), expected_len);

        let reader = Confidential1Reader::new(Cursor::new(encrypted), &key)?;
        let decrypted = read_all_as_string(reader)?;

        assert_eq!(decrypted, data);

        Ok(())
    }

    #[test]
    fn test_seek() -> Result<()> {
        let data = generate_alpanumeric_string(100);
        let key = Confidential1Key::new_random_key();

        let encrypted = {
            let mut writer = Confidential1Writer::new(Vec::new(), &key)?;
            writer.write_all(data.as_bytes())?;
            writer.finish()?
        };

        let mut reader = Confidential1Reader::new(Cursor::new(encrypted), &key)?;
        reader.seek(SeekFrom::Start(50))?;

        let mut decrypted = Vec::new();
        reader.read_to_end(&mut decrypted)?;

        assert_eq!(&decrypted, &data.as_bytes()[50..]);

        Ok(())
    }

    #[test]
    fn test_authentication() -> Result<()> {
        let data = generate_alpanumeric_string(100);
        let key = Confidential1Key::new_random_key();

        let mut encrypted = {
            let mut writer = Confidential1Writer::new(Vec::new(), &key)?;
            writer.write_all(data.as_bytes())?;
            writer.finish()?
        };

        {
            let mut reader = Confidential1AuthReader::new(encrypted.as_slice(), &key)?;
            let decrypted = read_all_as_string(&mut reader)?;

            assert_eq!(decrypted, data);
            assert_eq!(reader.is_authentic(), Some(true));
        }

        {
            // corrupt data
            encrypted[20] ^= 2;

            let mut reader = Confidential1AuthReader::new(encrypted.as_slice(), &key)?;
            let decrypted = read_all_as_string(&mut reader);
            assert!(decrypted.is_err());
        }

        Ok(())
    }
}
