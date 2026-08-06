use anyhow::Result;
use similar::{Algorithm, TextDiff};

use crate::entities::Document;

/// Unified diff between two canonical document data renderings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentDataDiff {
    pub unified_diff: String,
    pub has_changes: bool,
}

/// Builds a unified diff of two documents' JSON data using stable pretty JSON.
pub fn diff_document_data(
    left_label: &str,
    left: &Document,
    right_label: &str,
    right: &Document,
) -> Result<DocumentDataDiff> {
    let left_data = canonical_document_data(left)?;
    let right_data = canonical_document_data(right)?;
    let has_changes = left_data != right_data;

    let diff = TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .diff_lines(&left_data, &right_data);

    let unified_diff = diff
        .unified_diff()
        .header(left_label, right_label)
        .to_string();

    Ok(DocumentDataDiff {
        unified_diff,
        has_changes,
    })
}

fn canonical_document_data(document: &Document) -> Result<String> {
    let mut data = serde_json::to_string_pretty(&document.data)?;
    data.push('\n');

    Ok(data)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serde_json::json;

    use crate::entities::{Document, DocumentType};

    use super::diff_document_data;

    #[test]
    fn test_diff_document_data_uses_pretty_json() -> Result<()> {
        let left = Document::new(DocumentType::new("note"))
            .with_data(json!({"title": "Old", "body": "same"}));
        let right = Document::new(DocumentType::new("note"))
            .with_data(json!({"title": "New", "body": "same"}));

        let diff = diff_document_data("left", &left, "right", &right)?;

        assert!(diff.has_changes);
        assert!(diff.unified_diff.contains("--- left"));
        assert!(diff.unified_diff.contains("+++ right"));
        assert!(diff.unified_diff.contains("-  \"title\": \"Old\""));
        assert!(diff.unified_diff.contains("+  \"title\": \"New\""));

        Ok(())
    }

    #[test]
    fn test_diff_document_data_reports_unchanged_data() -> Result<()> {
        let left = Document::new(DocumentType::new("note")).with_data(json!({"title": "Same"}));
        let right = Document::new(DocumentType::new("note")).with_data(json!({"title": "Same"}));

        let diff = diff_document_data("left", &left, "right", &right)?;

        assert!(!diff.has_changes);

        Ok(())
    }
}
