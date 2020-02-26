use crate::entities::*;

pub struct PathFinder {
    pub root_path: String,
}

impl PathFinder {
    pub fn new(root_path: String) -> PathFinder {
        PathFinder { root_path }
    }

    pub fn get_state_file(&self) -> String {
        format!("{}/arhiv-state.json", self.root_path)
    }

    pub fn get_documents_directory(&self) -> String {
        format!("{}/documents", self.root_path)
    }

    pub fn get_attachments_directory(&self) -> String {
        format!("{}/attachments", self.root_path)
    }

    pub fn get_attachments_data_directory(&self) -> String {
        format!("{}/attachments-data", self.root_path)
    }

    pub fn get_document_path(&self, id: &Id) -> String {
        format!("{}/{}.json", self.get_documents_directory(), id)
    }

    pub fn get_attachment_path(&self, id: &Id) -> String {
        format!("{}/{}.json", self.get_attachments_directory(), id)
    }

    pub fn get_attachment_data_path(&self, id: &Id) -> String {
        format!("{}/{}", self.get_attachments_data_directory(), id)
    }
}
