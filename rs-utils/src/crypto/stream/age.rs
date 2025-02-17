use std::{
    io::{self, BufRead, Read, Seek, SeekFrom, Write},
    iter,
    str::FromStr,
};

use age::{
    scrypt,
    secrecy::{ExposeSecret, SecretString},
    stream::{StreamReader, StreamWriter},
    x25519, Decryptor, Encryptor, Identity, Recipient,
};
use anyhow::{anyhow, ensure, Context, Result};

pub const MIN_PASSWORD_LEN: usize = 8;

pub enum PrivateKey {
    Password(scrypt::Identity),
    Key(x25519::Identity),
}

impl PrivateKey {
    pub fn from_password(password: crate::SecretString) -> Result<Self> {
        ensure!(
            password.len() >= MIN_PASSWORD_LEN,
            "password must consist of at least {MIN_PASSWORD_LEN} bytes",
        );

        let password = SecretString::from(password.as_str());

        let identity = scrypt::Identity::new(password);

        Ok(PrivateKey::Password(identity))
    }

    pub fn from_age_x25519_key(key: crate::SecretString) -> Result<Self> {
        let identity = x25519::Identity::from_str(key.as_str())
            .map_err(|err| anyhow!("Failed to parse x25519 identity: {err}"))?;

        Ok(PrivateKey::Key(identity))
    }

    pub fn generate_age_x25519_key() -> crate::SecretString {
        let identity = x25519::Identity::generate();

        identity.to_string().expose_secret().to_string().into()
    }

    #[cfg(test)]
    pub fn new_random_key() -> Self {
        let key = Self::generate_age_x25519_key();

        Self::from_age_x25519_key(key).expect("key is valid")
    }

    fn get_identity(&self) -> &dyn Identity {
        match self {
            PrivateKey::Password(identity) => identity,
            PrivateKey::Key(identity) => identity,
        }
    }
}

pub struct AgeReader<R: Read> {
    inner: StreamReader<R>,
}

impl<R: Read> AgeReader<R> {
    pub fn new(reader: R, key: PrivateKey) -> Result<Self> {
        let decryptor = Decryptor::new(reader)?;

        Self::create(decryptor, key)
    }

    fn create(decryptor: Decryptor<R>, key: PrivateKey) -> Result<Self> {
        let reader = decryptor
            .decrypt(iter::once(key.get_identity()))
            .context("Failed to decrypt")?;

        Ok(Self { inner: reader })
    }
}

impl<R: BufRead> AgeReader<R> {
    pub fn new_buffered(reader: R, key: PrivateKey) -> Result<Self> {
        let decryptor = Decryptor::new_buffered(reader)?;

        Self::create(decryptor, key)
    }
}

impl<R: Read> Read for AgeReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Read + Seek> Seek for AgeReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

pub enum PublicKey {
    Password(scrypt::Recipient),
    Key(x25519::Recipient),
}

impl PublicKey {
    pub fn from_password(password: crate::SecretString) -> Result<Self> {
        ensure!(
            password.len() >= MIN_PASSWORD_LEN,
            "password must consist of at least {MIN_PASSWORD_LEN} bytes",
        );
        let password = SecretString::from(password.as_str());

        let recipient = scrypt::Recipient::new(password);

        Ok(PublicKey::Password(recipient))
    }

    pub fn from_age_x25519_key(key: crate::SecretString) -> Result<Self> {
        let identity = x25519::Identity::from_str(key.as_str())
            .map_err(|err| anyhow!("Failed to parse x25519 identity: {err}"))?;

        let recipient = identity.to_public();

        Ok(PublicKey::Key(recipient))
    }

    fn get_recipient(&self) -> &dyn Recipient {
        match self {
            PublicKey::Password(recipient) => recipient,
            PublicKey::Key(recipient) => recipient,
        }
    }
}

pub struct AgeWriter<W: Write> {
    inner: StreamWriter<W>,
}

impl<W: Write> AgeWriter<W> {
    pub fn new(writer: W, key: PublicKey) -> Result<Self> {
        let encryptor = Encryptor::with_recipients(iter::once(key.get_recipient()))?;

        let inner = encryptor.wrap_output(writer)?;

        Ok(Self { inner })
    }

    pub fn finish(self) -> Result<W> {
        let mut writer = self.inner.finish()?;
        writer.flush()?;

        Ok(writer)
    }
}

impl<W: Write> Write for AgeWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{generate_alpanumeric_string, read_all_as_string};

    use super::*;

    #[test]
    fn test_write_read() {
        let data = generate_alpanumeric_string(100 * 1024);
        let key = PrivateKey::generate_age_x25519_key();

        let encrypted = {
            let key = PublicKey::from_age_x25519_key(key.duplicate()).unwrap();
            let mut writer = AgeWriter::new(Vec::new(), key).unwrap();
            writer.write_all(data.as_bytes()).unwrap();
            writer.finish().unwrap()
        };

        let decrypted = {
            let key = PrivateKey::from_age_x25519_key(key.duplicate()).unwrap();
            let reader = AgeReader::new(Cursor::new(encrypted), key).unwrap();

            read_all_as_string(reader).unwrap()
        };

        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_seek() {
        let data = generate_alpanumeric_string(100);
        let key = PrivateKey::generate_age_x25519_key();

        let encrypted = {
            let key = PublicKey::from_age_x25519_key(key.duplicate()).unwrap();
            let mut writer = AgeWriter::new(Vec::new(), key).unwrap();
            writer.write_all(data.as_bytes()).unwrap();
            writer.finish().unwrap()
        };

        let decrypted = {
            let key = PrivateKey::from_age_x25519_key(key.duplicate()).unwrap();
            let mut reader = AgeReader::new(Cursor::new(encrypted), key).unwrap();
            reader.seek(SeekFrom::Start(50)).unwrap();

            let mut decrypted = Vec::new();
            reader.read_to_end(&mut decrypted).unwrap();

            decrypted
        };

        assert_eq!(&decrypted, &data.as_bytes()[50..]);
    }

    #[test]
    fn test_authentication() {
        let data = generate_alpanumeric_string(100);
        let key = PrivateKey::generate_age_x25519_key();

        let mut encrypted = {
            let key = PublicKey::from_age_x25519_key(key.duplicate()).unwrap();
            let mut writer = AgeWriter::new(Vec::new(), key).unwrap();
            writer.write_all(data.as_bytes()).unwrap();
            writer.finish().unwrap()
        };

        // corrupt data
        let len = encrypted.len();
        encrypted[len - 50] ^= 2;

        {
            let key = PrivateKey::from_age_x25519_key(key.duplicate()).unwrap();
            let mut reader = AgeReader::new(encrypted.as_slice(), key).unwrap();
            let decrypted = read_all_as_string(&mut reader);
            assert!(decrypted.is_err());
        }
    }
}
