use super::AttachmentData;
use crate::entities::Hash;

pub struct BlobManager {
    data_dir: String,
}

impl BlobManager {
    pub fn new<S: Into<String>>(data_dir: S) -> Self {
        BlobManager {
            data_dir: data_dir.into(),
        }
    }

    fn get_attachment_data_path(&self, hash: &Hash) -> String {
        format!("{}/{}", &self.data_dir, hash)
    }

    pub fn get_attachment_data(&self, hash: Hash) -> AttachmentData {
        let path = self.get_attachment_data_path(&hash);

        AttachmentData::new(hash, path)
    }
}
