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
    // TODO make const fn
    let chars: Vec<char> = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
        .chars()
        .collect();

    // see https://zelark.github.io/nano-id-cc/
    nanoid::nanoid!(14, &chars)
}
