use std::{
    io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write},
    iter,
    str::FromStr,
};

use age::{
    armor::{ArmoredReader, ArmoredWriter, Format},
    scrypt,
    secrecy::{ExposeSecret, SecretString},
    stream::{StreamReader, StreamWriter},
    x25519, Decryptor, Encryptor, Identity, Recipient,
};
use anyhow::{anyhow, ensure, Context, Result};

use crate::{create_file_reader, create_file_writer, read_all};

use super::SecretBytes;

#[derive(Clone)]
pub enum AgeKey {
    Password(SecretString),
    Key(x25519::Identity),
}

impl AgeKey {
    pub const MIN_PASSWORD_LEN: usize = 8;

    pub fn from_password(password: SecretString) -> Result<Self> {
        ensure!(
            password.expose_secret().len() >= Self::MIN_PASSWORD_LEN,
            "password must consist of at least {} bytes",
            Self::MIN_PASSWORD_LEN
        );

        Ok(AgeKey::Password(password))
    }

    pub fn from_age_x25519_key(key: SecretString) -> Result<Self> {
        let identity = x25519::Identity::from_str(key.expose_secret())
            .map_err(|err| anyhow!("Failed to parse x25519 key: {err}"))?;

        Ok(AgeKey::Key(identity))
    }

    pub fn generate_age_x25519_key() -> Self {
        let identity = x25519::Identity::generate();

        AgeKey::Key(identity)
    }

    pub fn serialize(&self) -> SecretString {
        match self {
            AgeKey::Password(password) => password.clone(),
            AgeKey::Key(identity) => identity.to_string(),
        }
    }

    fn into_identity(self) -> Box<dyn Identity> {
        match self {
            AgeKey::Password(password) => {
                #[cfg(test)]
                {
                    let mut identity = scrypt::Identity::new(password);

                    identity.set_max_work_factor(1);

                    Box::new(identity)
                }

                #[cfg(not(test))]
                {
                    let identity = scrypt::Identity::new(password);

                    Box::new(identity)
                }
            }
            AgeKey::Key(identity) => Box::new(identity),
        }
    }

    fn into_recipient(self) -> Box<dyn Recipient> {
        match self {
            AgeKey::Password(password) => {
                #[cfg(test)]
                {
                    let mut recipient = scrypt::Recipient::new(password);
                    recipient.set_work_factor(1);

                    Box::new(recipient)
                }

                #[cfg(not(test))]
                {
                    let recipient = scrypt::Recipient::new(password);

                    Box::new(recipient)
                }
            }
            AgeKey::Key(identity) => Box::new(identity.to_public()),
        }
    }
}

pub struct AgeReader<R: Read> {
    inner: StreamReader<R>,
}

impl<R: Read> AgeReader<R> {
    pub fn new(reader: R, key: AgeKey) -> Result<Self> {
        let decryptor = Decryptor::new(reader)?;

        Self::create(decryptor, key)
    }

    pub fn new_armored(reader: R, key: AgeKey) -> Result<AgeReader<ArmoredReader<BufReader<R>>>> {
        let decryptor = Decryptor::new(ArmoredReader::new(reader))?;

        AgeReader::create(decryptor, key)
    }

    fn create(decryptor: Decryptor<R>, key: AgeKey) -> Result<Self> {
        let reader = decryptor
            .decrypt(iter::once(key.into_identity().as_ref()))
            .context("Failed to decrypt")?;

        Ok(Self { inner: reader })
    }
}

impl<R: BufRead> AgeReader<R> {
    pub fn new_buffered(reader: R, key: AgeKey) -> Result<Self> {
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

pub struct AgeWriter<W: Write> {
    inner: StreamWriter<ArmoredWriter<W>>,
}

impl<W: Write> AgeWriter<W> {
    pub fn new(writer: W, key: AgeKey) -> Result<Self> {
        AgeWriter::create(writer, key, false)
    }

    pub fn new_armored(writer: W, key: AgeKey) -> Result<Self> {
        AgeWriter::create(writer, key, true)
    }

    fn create(writer: W, key: AgeKey, armored: bool) -> Result<Self> {
        let writer = ArmoredWriter::wrap_output(
            writer,
            if armored {
                Format::AsciiArmor
            } else {
                Format::Binary
            },
        )?;

        let encryptor = Encryptor::with_recipients(iter::once(key.into_recipient().as_ref()))?;

        let inner = encryptor.wrap_output(writer)?;

        Ok(Self { inner })
    }

    pub fn finish(self) -> Result<W> {
        let armored_writer = self.inner.finish()?;

        let inner_writer = armored_writer.finish()?;

        Ok(inner_writer)
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

pub fn read_and_decrypt_file(file_path: &str, key: AgeKey, armored: bool) -> Result<SecretBytes> {
    let reader = create_file_reader(file_path)?;

    let data = if armored {
        let age_reader = AgeReader::new_armored(reader, key)?;

        read_all(age_reader)?
    } else {
        let age_reader = AgeReader::new_buffered(reader, key)?;

        read_all(age_reader)?
    };

    let data = SecretBytes::new(data);

    Ok(data)
}

pub fn encrypt_and_write_file(
    file_path: &str,
    key: AgeKey,
    data: SecretBytes,
    armored: bool,
) -> Result<()> {
    let writer = create_file_writer(file_path, false)?;

    let mut age_writer = if armored {
        AgeWriter::new_armored(writer, key)?
    } else {
        AgeWriter::new(writer, key)?
    };

    age_writer.write_all(data.expose_secret())?;
    age_writer.finish()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{generate_alpanumeric_string, read_all_as_string};

    use super::*;

    #[test]
    fn test_write_read() {
        let data = generate_alpanumeric_string(100 * 1024);
        let key = AgeKey::generate_age_x25519_key();

        let encrypted = {
            let mut writer = AgeWriter::new(Vec::new(), key.clone()).unwrap();
            writer.write_all(data.as_bytes()).unwrap();
            writer.finish().unwrap()
        };

        let decrypted = {
            let reader = AgeReader::new(Cursor::new(encrypted), key).unwrap();

            read_all_as_string(reader).unwrap()
        };

        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_write_read_armored() {
        let data = generate_alpanumeric_string(100 * 1024);
        let key = AgeKey::generate_age_x25519_key();

        let encrypted = {
            let mut writer = AgeWriter::new_armored(Vec::new(), key.clone()).unwrap();
            writer.write_all(data.as_bytes()).unwrap();
            writer.finish().unwrap()
        };

        let decrypted = {
            let reader = AgeReader::new_armored(Cursor::new(encrypted), key).unwrap();

            read_all_as_string(reader).unwrap()
        };

        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_write_read_with_password() {
        let data = generate_alpanumeric_string(100 * 1024);
        let key = AgeKey::from_password("test1234".into()).unwrap();

        let encrypted = {
            let mut writer = AgeWriter::new(Vec::new(), key.clone()).unwrap();
            writer.write_all(data.as_bytes()).unwrap();
            writer.finish().unwrap()
        };

        let decrypted = {
            let reader = AgeReader::new(Cursor::new(encrypted), key).unwrap();

            read_all_as_string(reader).unwrap()
        };

        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_seek() {
        let data = generate_alpanumeric_string(100);
        let key = AgeKey::generate_age_x25519_key();

        let encrypted = {
            let mut writer = AgeWriter::new(Vec::new(), key.clone()).unwrap();
            writer.write_all(data.as_bytes()).unwrap();
            writer.finish().unwrap()
        };

        let decrypted = {
            let mut reader = AgeReader::new(Cursor::new(encrypted), key).unwrap();
            reader.seek(SeekFrom::Start(50)).unwrap();

            read_all(reader).unwrap()
        };

        assert_eq!(&decrypted, &data.as_bytes()[50..]);
    }

    #[test]
    fn test_authentication() {
        let data = generate_alpanumeric_string(100);
        let key = AgeKey::generate_age_x25519_key();

        let mut encrypted = {
            let mut writer = AgeWriter::new(Vec::new(), key.clone()).unwrap();
            writer.write_all(data.as_bytes()).unwrap();
            writer.finish().unwrap()
        };

        // corrupt data
        let len = encrypted.len();
        encrypted[len - 50] ^= 2;

        {
            let mut reader = AgeReader::new(encrypted.as_slice(), key).unwrap();
            let decrypted = read_all_as_string(&mut reader);
            assert!(decrypted.is_err());
        }
    }
}
