use std::{cmp::Ordering, collections::HashMap, io::Write};

use anyhow::{Result, anyhow, bail, ensure};

use rs_utils::{AgeGzWriter, ContainerWriter, age::AgeKey};

use crate::entities::{Document, DocumentKey};

use super::{BazaInfo, DocumentsIndex};

pub struct ContainerDraft<W: Write> {
    writer: Option<ContainerWriter<AgeGzWriter<W>>>,
    index: DocumentsIndex,
    pending: HashMap<DocumentKey, String>,
    next_expected: usize,
}

impl<W: Write> ContainerDraft<W> {
    pub fn new(writer: W, key: AgeKey, info: &BazaInfo, index: DocumentsIndex) -> Result<Self> {
        let lines_index = index.to_lines_index();
        let age_writer = AgeGzWriter::new(writer, key)?;
        let mut container_writer = ContainerWriter::new(age_writer);

        container_writer.write_index(&lines_index)?;

        // Baza info is the first line
        container_writer.write_line(&serde_json::to_string(info)?)?;

        Ok(ContainerDraft {
            writer: Some(container_writer),
            index,
            pending: HashMap::new(),
            next_expected: 0,
        })
    }

    pub fn push_document(&mut self, document: &Document) -> Result<()> {
        let serialized = serde_json::to_string(document)?;
        let key = DocumentKey::for_document(document);

        self.push_serialized(key, serialized)
    }

    pub fn push_serialized(&mut self, key: DocumentKey, line: String) -> Result<()> {
        let position = self
            .index
            .position_of(&key)
            .ok_or_else(|| anyhow!("document {} not found in index", key.serialize()))?;

        let key_serialized = key.serialize();

        match position.cmp(&self.next_expected) {
            Ordering::Less => bail!("document {} already written", key_serialized),
            Ordering::Equal => {
                self.write_line(&line)?;
                self.next_expected += 1;
                self.flush_pending()?;
            }
            Ordering::Greater => {
                if self.pending.insert(key, line).is_some() {
                    bail!("document {} provided multiple times", key_serialized);
                }
            }
        }

        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        ensure!(
            self.next_expected == self.index.len(),
            "expected {} documents, got {}",
            self.index.len(),
            self.next_expected
        );
        ensure!(
            self.pending.is_empty(),
            "there are {} pending documents left to flush",
            self.pending.len()
        );

        let writer = self.writer.take().expect("writer must be available");
        let age_writer = writer.finish()?;
        age_writer.finish()?;
        Ok(())
    }

    fn flush_pending(&mut self) -> Result<()> {
        while let Some(next_key) = self.index.key_at(self.next_expected) {
            if let Some(line) = self.pending.remove(next_key) {
                self.write_line(&line)?;
                self.next_expected += 1;
            } else {
                break;
            }
        }

        Ok(())
    }

    fn write_line(&mut self, line: &str) -> Result<()> {
        let writer = self
            .writer
            .as_mut()
            .expect("writer must be available during writes");
        writer.write_line(line)?;
        Ok(())
    }
}
