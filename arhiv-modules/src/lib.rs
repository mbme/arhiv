#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod markup;
mod notes;

use std::collections::HashSet;

use arhiv::entities::*;
pub use markup::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub use notes::*;

pub trait DocumentImpl {
    const TYPE: &'static str;

    type Data: Serialize + DeserializeOwned + Default;

    fn new() -> Self;

    fn from_document(document: Document) -> Self;

    fn into_document(self) -> Document<Self::Data>;

    fn extract_refs(&self) -> HashSet<Id>;
}
