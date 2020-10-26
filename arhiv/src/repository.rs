use crate::entities::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait DocumentImpl {
    const TYPE: &'static str;

    type Data: Serialize + DeserializeOwned + Default;

    fn new() -> Self;

    fn from(document: Document) -> Self;

    fn into_document(self) -> Document<Self::Data>;
}
