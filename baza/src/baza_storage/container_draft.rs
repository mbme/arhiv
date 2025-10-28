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
    pub fn new(writer: W, key: AgeKey, info: &BazaInfo, mut index: DocumentsIndex) -> Result<Self> {
        // sort keys to co-locate similar documents for better compression
        index.sort_by_document_key();

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    use anyhow::Result;
    use rs_utils::{AgeGzReader, ContainerReader, age::AgeKey};
    use serde_json::json;

    use crate::entities::{Document, DocumentKey, Id, new_document};

    #[test]
    fn streams_out_of_order_documents_in_sorted_key_order() -> Result<()> {
        let info = BazaInfo::new_test_info();
        let key = AgeKey::generate_age_x25519_key();

        let doc_a1 = new_document(json!({ "payload": "a1" }))
            .with_id(Id::from("aaaaaaaaaaaaaa"))
            .with_rev(json!({ "a": 1 }));
        let doc_a2 = new_document(json!({ "payload": "a2" }))
            .with_id(Id::from("aaaaaaaaaaaaaa"))
            .with_rev(json!({ "a": 2 }));
        let doc_b1 = new_document(json!({ "payload": "b1" }))
            .with_id(Id::from("bbbbbbbbbbbbbb"))
            .with_rev(json!({ "b": 1 }));

        let expected_keys = [
            DocumentKey::for_document(&doc_a1),
            DocumentKey::for_document(&doc_a2),
            DocumentKey::for_document(&doc_b1),
        ];

        let index = DocumentsIndex::from_document_keys(vec![
            DocumentKey::for_document(&doc_b1),
            DocumentKey::for_document(&doc_a2),
            DocumentKey::for_document(&doc_a1),
        ])?;

        let mut buffer = Vec::new();
        {
            let mut draft = ContainerDraft::new(&mut buffer, key.clone(), &info, index)?;

            draft.push_document(&doc_b1)?;
            draft.push_document(&doc_a2)?;
            draft.push_document(&doc_a1)?;

            draft.finish()?;
        }

        let reader = Cursor::new(buffer);
        let age_reader = AgeGzReader::new(reader, key)?;
        let container_reader = ContainerReader::init(age_reader)?;

        let index_lines: Vec<_> = container_reader
            .get_index()
            .iter()
            .map(|value| value.to_string())
            .collect();

        let expected_index: Vec<_> = std::iter::once("info".to_string())
            .chain(expected_keys.iter().map(DocumentKey::serialize))
            .collect();

        assert_eq!(index_lines, expected_index);

        let mut lines_iter = container_reader.into_lines_iter();

        let (info_key, info_line) = lines_iter.next().expect("info line present")?;
        assert_eq!(info_key, "info");
        let parsed_info: BazaInfo = serde_json::from_str(&info_line)?;
        assert_eq!(parsed_info, info);

        let (first_key, first_line) = lines_iter.next().expect("first document")?;
        let parsed_first: Document = serde_json::from_str(&first_line)?;
        assert_eq!(first_key, expected_keys[0].serialize());
        assert_eq!(parsed_first, doc_a1);

        let (second_key, second_line) = lines_iter.next().expect("second document")?;
        let parsed_second: Document = serde_json::from_str(&second_line)?;
        assert_eq!(second_key, expected_keys[1].serialize());
        assert_eq!(parsed_second, doc_a2);

        let (third_key, third_line) = lines_iter.next().expect("third document")?;
        let parsed_third: Document = serde_json::from_str(&third_line)?;
        assert_eq!(third_key, expected_keys[2].serialize());
        assert_eq!(parsed_third, doc_b1);

        assert!(lines_iter.next().is_none());

        Ok(())
    }

    #[test]
    fn rejects_duplicate_document_push() -> Result<()> {
        let info = BazaInfo::new_test_info();
        let key = AgeKey::generate_age_x25519_key();

        let doc = new_document(json!({ "payload": "dup" }))
            .with_id(Id::from("cccccccccccccc"))
            .with_rev(json!({ "c": 1 }));

        let index = DocumentsIndex::from_document_keys(vec![DocumentKey::for_document(&doc)])?;

        let mut buffer = Vec::new();
        let mut draft = ContainerDraft::new(&mut buffer, key, &info, index)?;

        draft.push_document(&doc)?;

        let err = draft.push_document(&doc).unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("already written") || err_msg.contains("provided multiple times"),
            "unexpected error message: {err_msg}"
        );

        draft.finish().unwrap();

        Ok(())
    }
}
