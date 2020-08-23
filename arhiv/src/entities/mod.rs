use uuid::Uuid;

mod attachment;
mod changeset;
mod changeset_response;
mod document;

pub use attachment::Attachment;
pub use changeset::Changeset;
pub use changeset_response::ChangesetResponse;
pub use document::Document;

pub type Revision = u32;
pub type Id = String;

pub fn gen_id() -> Id {
    Uuid::new_v4().to_hyphenated().to_string()
}
