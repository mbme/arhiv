#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod markup;
mod notes;

use arhiv::entities::*;
pub use markup::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub use notes::*;

pub trait DocumentImpl {
    const TYPE: &'static str;

    type Data: Serialize + DeserializeOwned + Default;

    fn new() -> Self;

    fn from(document: Document) -> Self;

    fn into_document(self) -> Document<Self::Data>;
}
