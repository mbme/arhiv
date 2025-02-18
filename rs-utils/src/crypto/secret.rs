use secrecy::{SecretBox, SecretSlice};

use crate::new_random_crypto_byte_array;

pub use secrecy::{ExposeSecret, SecretString};

pub struct SecretByteArray<const SIZE: usize>(SecretBox<[u8; SIZE]>);

impl<const SIZE: usize> SecretByteArray<SIZE> {
    pub fn new(bytes: [u8; SIZE]) -> Self {
        SecretByteArray(SecretBox::new(Box::new(bytes)))
    }

    pub fn new_random() -> Self {
        SecretByteArray::new(new_random_crypto_byte_array())
    }

    pub fn len(&self) -> usize {
        self.0.expose_secret().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<const SIZE: usize> ExposeSecret<[u8; SIZE]> for SecretByteArray<SIZE> {
    fn expose_secret(&self) -> &[u8; SIZE] {
        self.0.expose_secret()
    }
}

impl<const SIZE: usize> Clone for SecretByteArray<SIZE> {
    fn clone(&self) -> Self {
        Self::new(*self.0.expose_secret())
    }
}

#[derive(Clone)]
pub struct SecretBytes(SecretSlice<u8>);

impl SecretBytes {
    pub fn new(value: Vec<u8>) -> Self {
        Self(SecretSlice::new(value.into_boxed_slice()))
    }

    pub fn len(&self) -> usize {
        self.0.expose_secret().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl ExposeSecret<[u8]> for SecretBytes {
    fn expose_secret(&self) -> &[u8] {
        self.0.expose_secret()
    }
}
