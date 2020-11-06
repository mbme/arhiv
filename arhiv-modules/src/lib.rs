#![deny(clippy::all)]
#![deny(clippy::pedantic)]

pub mod generator;
mod markup;
mod note;
mod project;
mod task;

use std::collections::HashSet;

use arhiv::entities::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub use markup::*;

pub use note::*;
pub use project::*;
pub use task::*;

pub trait DocumentImpl {
    const TYPE: &'static str;

    type Data: Serialize + DeserializeOwned;

    fn from_document(document: Document) -> Self;

    fn into_document(self) -> Document<Self::Data>;

    fn extract_refs(&self) -> HashSet<Id>;
}
