use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AttachmentLocation {
    Url(String),
    File(String),
}
