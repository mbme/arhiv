use std::io::Write;

use anyhow::Result;

use rs_utils::{AgeGzWriter, ContainerWriter, age::AgeKey};

use crate::entities::{Document, DocumentKey};

use super::{BazaInfo, DocumentsIndex};

pub struct ContainerDraft<W: Write> {
    key: AgeKey,
    info: BazaInfo,
    writer: Option<W>,
    entries: Vec<(DocumentKey, String)>,
}

impl<W: Write> ContainerDraft<W> {
    pub fn new(writer: W, key: AgeKey, info: BazaInfo, capacity: usize) -> Self {
        ContainerDraft {
            key,
            info,
            writer: Some(writer),
            entries: Vec::with_capacity(capacity),
        }
    }

    pub fn push_document(&mut self, document: &Document) -> Result<()> {
        let serialized = serde_json::to_string(document)?;

        self.entries
            .push((DocumentKey::for_document(document), serialized));

        Ok(())
    }

    pub fn push_serialized(&mut self, key: DocumentKey, line: String) -> Result<()> {
        self.entries.push((key, line));
        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        let writer = self.writer.take().expect("writer must be available");
        let age_writer = AgeGzWriter::new(writer, self.key)?;
        let mut container_writer = ContainerWriter::new(age_writer);

        let index =
            DocumentsIndex::from_document_keys(self.entries.iter().map(|(key, _)| key.clone()));
        container_writer.write_index(&index)?;

        let info_line = serde_json::to_string(&self.info)?;
        container_writer.write_line(&info_line)?;

        for (_key, line) in self.entries {
            container_writer.write_line(&line)?;
        }

        let age_writer = container_writer.finish()?;
        age_writer.finish()?;

        Ok(())
    }
}
