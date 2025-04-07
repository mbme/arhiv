use anyhow::Result;
use serde::Serialize;

use crate::entities::{Document, DocumentType};

use super::BazaState;

#[derive(Default)]
pub struct Filter {
    pub document_types: Vec<DocumentType>,
    pub query: String,
    pub page: u8,
}

impl Filter {
    pub fn should_show_document(&self, doc: &Document) -> bool {
        // we should ignore erased documents unless explicitly included in document_types
        if doc.is_erased() {
            return self.document_types.contains(&DocumentType::erased());
        }

        if self.document_types.is_empty() {
            return true;
        }

        self.document_types.contains(&doc.document_type)
    }
}

#[derive(Debug, Serialize)]
pub struct ListPage<'d> {
    pub items: Vec<&'d Document>,
    pub has_more: bool,
}

const PAGE_SIZE: usize = 10;

impl BazaState {
    pub fn list_documents(&self, filter: &Filter) -> Result<ListPage> {
        let page_start = (filter.page as usize) * PAGE_SIZE;

        if filter.query.trim().is_empty() {
            let mut filtered_documents: Vec<&Document> = self
                .iter_documents()
                .map(|head| head.get_single_document())
                .filter(|doc| filter.should_show_document(doc))
                .collect();

            // sort by modification time
            filtered_documents.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

            let page_end = page_start + PAGE_SIZE;
            let paginated_documents =
                &filtered_documents[page_start..filtered_documents.len().min(page_end)];

            Ok(ListPage {
                items: paginated_documents.to_vec(),
                has_more: page_end < filtered_documents.len(),
            })
        } else {
            let mut items = self
                .search
                .search(&filter.query)
                .map(|id| {
                    self.must_get_document(&id)
                        .expect("Document returned by search engine must exist")
                })
                .filter(|doc| filter.should_show_document(doc))
                .skip(page_start)
                .take(PAGE_SIZE + 1)
                .collect::<Vec<_>>();

            let has_more = items.len() > PAGE_SIZE;
            if has_more {
                items.remove(PAGE_SIZE);
            }

            Ok(ListPage { items, has_more })
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::entities::{new_document, new_empty_document, DocumentType, Id};

    use super::*;

    #[test]
    fn test_list_documents() {
        let mut state = BazaState::new_test_state();

        let mut erased = new_empty_document().with_rev(json!({ "a": 1 }));
        erased.erase();
        let doc1 = new_document(json!({ "test": "value" })).with_rev(json!({ "a": 1 }));
        let doc2 = doc1
            .clone()
            .with_id(Id::new())
            .with_data(json!({ "test": "other value" }));
        state.insert_snapshots(vec![doc1, doc2, erased]);

        // Check if querying doesn't return erased documents by default
        {
            let result = state.list_documents(&Default::default()).unwrap();
            assert_eq!(result.items.len(), 2);
            assert!(result.items[0].updated_at >= result.items[1].updated_at);
            assert!(!result.has_more);
        }

        // Check if query "Val" returns 2 documents
        {
            let filter = Filter {
                query: "Val".to_string(),
                ..Default::default()
            };

            let result = state.list_documents(&filter).unwrap();
            assert_eq!(result.items.len(), 2);
            assert!(result.items[0].updated_at >= result.items[1].updated_at);
            assert!(!result.has_more);
        }

        // Check if query "oth" returns 1 document
        {
            let filter = Filter {
                query: "oth".to_string(),
                ..Default::default()
            };

            let result = state.list_documents(&filter).unwrap();
            assert_eq!(result.items.len(), 1);
            assert!(!result.has_more);
        }

        // Check if querying for erased document returns 1 erased document
        {
            let filter = Filter {
                document_types: vec![DocumentType::erased()],
                ..Default::default()
            };

            let result = state.list_documents(&filter).unwrap();
            assert_eq!(result.items.len(), 1);
            assert_eq!(result.items[0].document_type, DocumentType::erased());
            assert!(!result.has_more);
        }

        // Add more documents to test pagination
        for _ in 0..PAGE_SIZE {
            state
                .insert_snapshot(new_empty_document().with_rev(json!({ "a": 1 })))
                .unwrap();
        }

        // Check if pagination works
        {
            let filter = Filter {
                page: 0,
                ..Default::default()
            };

            let result = state.list_documents(&filter).unwrap();
            assert_eq!(result.items.len(), PAGE_SIZE);
            assert!(result.has_more);

            let filter = Filter {
                page: 1,
                ..Default::default()
            };

            let result = state.list_documents(&filter).unwrap();
            assert_eq!(result.items.len(), 2); // Remaining documents
            assert!(!result.has_more);
        }
    }
}
