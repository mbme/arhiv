use super::Id;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentSource {
    pub id: Id,
    pub file_path: String,
    pub filename: String,
    pub copy: bool,
}

impl AttachmentSource {
    pub fn new<S: Into<String>>(file_path: S) -> AttachmentSource {
        let file_path = file_path.into();

        let filename = Path::new(&file_path)
            .file_name()
            .expect("file must have name")
            .to_str()
            .expect("file name must be valid string");

        AttachmentSource {
            id: Id::new(),
            filename: filename.to_string(),
            file_path,
            copy: false,
        }
    }

    pub fn new_from_path_buf(file_path: &PathBuf) -> AttachmentSource {
        Self::new(file_path.to_str().expect("Path must be valid string"))
    }
}

impl fmt::Display for AttachmentSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[AttachmentSource {} \"{}\"]", self.id, self.file_path)
    }
}
