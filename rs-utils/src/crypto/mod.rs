use rand::{rngs::OsRng, RngCore};

mod auth_token;
mod certificate;
pub mod crypto_key;
mod hash;
mod hmac;
mod secret;
mod stream;

pub use auth_token::*;
pub use certificate::*;
pub use hash::*;
pub use hmac::*;
pub use secret::*;
pub use stream::*;

#[must_use]
pub fn new_random_crypto_byte_array<const SIZE: usize>() -> [u8; SIZE] {
    let mut bytes = [0u8; SIZE];

    OsRng.fill_bytes(&mut bytes);

    bytes
}
