mod document;
mod document_data;
mod document_key;
mod document_lock;
mod document_type;
mod id;
mod instance_id;
mod refs;
mod revision;

pub use document::Document;
pub use document_data::DocumentData;
pub use document_key::DocumentKey;
pub use document_lock::{DocumentLock, DocumentLockKey};
pub use document_type::{DocumentType, ERASED_DOCUMENT_TYPE};
pub use id::Id;
pub use instance_id::InstanceId;
pub use refs::Refs;
pub use revision::{LatestRevComputer, Revision, VectorClockOrder};

#[cfg(test)]
pub use document::{new_document, new_empty_document};
