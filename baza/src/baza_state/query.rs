use anyhow::Result;

use crate::entities::DocumentType;

use super::{BazaState, DocumentHead};

#[derive(Default)]
pub struct Filter {
    pub document_types: Vec<DocumentType>,
    pub query: String,
    pub page: u8,
    pub only_conflicts: bool,
}

impl Filter {
    pub fn should_show_document(&self, head: &DocumentHead) -> bool {
        self.should_show_document_type(head.get_type())
            && self.should_show_if_conflict(head.is_conflict())
    }

    fn should_show_document_type(&self, document_type: &DocumentType) -> bool {
        // we should ignore erased documents unless explicitly included in document_types
        if document_type.is_erased() {
            return self.document_types.contains(&DocumentType::erased());
        }

        if self.document_types.is_empty() {
            return true;
        }

        self.document_types.contains(document_type)
    }

    fn should_show_if_conflict(&self, is_conflict: bool) -> bool {
        if self.only_conflicts {
            is_conflict
        } else {
            true
        }
    }
}

#[derive(Debug)]
pub struct ListPage<'d> {
    pub items: Vec<&'d DocumentHead>,
    pub has_more: bool,
    pub total: usize,
}

const PAGE_SIZE: usize = 10;

impl BazaState {
    pub fn list_documents(&self, filter: &Filter) -> Result<ListPage<'_>> {
        let page_start = (filter.page as usize) * PAGE_SIZE;

        if filter.query.trim().is_empty() {
            let mut filtered_documents = self
                .iter_documents()
                .filter(|head| filter.should_show_document(head))
                .collect::<Vec<_>>();

            // sort by modification time
            filtered_documents.sort_by(|a, b| b.get_updated_at().cmp(a.get_updated_at()));

            let page_end = page_start + PAGE_SIZE;
            let paginated_documents =
                &filtered_documents[page_start..filtered_documents.len().min(page_end)];

            Ok(ListPage {
                items: paginated_documents.to_vec(),
                has_more: page_end < filtered_documents.len(),
                total: filtered_documents.len(),
            })
        } else {
            let results = self
                .search
                .search(&filter.query)
                .map(|id| {
                    self.get_document(&id)
                        .expect("Document returned by search engine must exist")
                })
                .filter(|doc| filter.should_show_document(doc))
                .collect::<Vec<_>>();

            let total = results.len();

            let has_more = (page_start + PAGE_SIZE) < total;

            let items = results
                .into_iter()
                .skip(page_start)
                .take(PAGE_SIZE)
                .collect::<Vec<_>>();

            Ok(ListPage {
                items,
                has_more,
                total,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::entities::{DocumentType, Id, new_document, new_empty_document};

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
            assert!(result.items[0].get_updated_at() >= result.items[1].get_updated_at());
            assert!(!result.has_more);
            assert_eq!(result.total, 2);
        }

        // Check if query "Val" returns 2 documents
        {
            let filter = Filter {
                query: "Val".to_string(),
                ..Default::default()
            };

            let result = state.list_documents(&filter).unwrap();
            assert_eq!(result.items.len(), 2);
            assert!(result.items[0].get_updated_at() >= result.items[1].get_updated_at());
            assert!(!result.has_more);
            assert_eq!(result.total, 2);
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
            assert_eq!(result.total, 1);
        }

        // Check if querying for erased document returns 1 erased document
        {
            let filter = Filter {
                document_types: vec![DocumentType::erased()],
                ..Default::default()
            };

            let result = state.list_documents(&filter).unwrap();
            assert_eq!(result.items.len(), 1);
            assert!(result.items[0].get_type().is_erased());
            assert!(!result.has_more);
            assert_eq!(result.total, 1);
        }

        // Add more documents to test pagination
        state.insert_snapshots(
            (0..PAGE_SIZE)
                .map(|_| new_document(json!({ "test": "value" })).with_rev(json!({ "a": 1 })))
                .collect(),
        );

        // Check if pagination works when no query
        {
            let filter = Filter {
                page: 0,
                ..Default::default()
            };

            let result = state.list_documents(&filter).unwrap();
            assert_eq!(result.items.len(), PAGE_SIZE);
            assert!(result.has_more);
            assert_eq!(result.total, 12);

            let filter = Filter {
                page: 1,
                ..Default::default()
            };

            let result = state.list_documents(&filter).unwrap();
            assert_eq!(result.items.len(), 2); // Remaining documents
            assert!(!result.has_more);
            assert_eq!(result.total, 12);
        }

        // Check if pagination works with query
        {
            let filter = Filter {
                page: 0,
                query: "val".to_string(),
                ..Default::default()
            };

            let result = state.list_documents(&filter).unwrap();
            assert_eq!(result.items.len(), PAGE_SIZE);
            assert!(result.has_more);
            assert_eq!(result.total, 12);

            let filter = Filter {
                page: 1,
                query: "val".to_string(),
                ..Default::default()
            };

            let result = state.list_documents(&filter).unwrap();
            assert_eq!(result.items.len(), 2); // Remaining documents
            assert!(!result.has_more);
            assert_eq!(result.total, 12);
        }
    }

    #[test]
    fn test_list_conflics() {
        let mut state = BazaState::new_test_state();

        let doc = new_document(json!({"test": "value"})).with_rev(json!({"r": 1}));
        let conflict1 = new_empty_document().with_rev(json!({"a": 1}));
        let conflict2 = conflict1.clone().with_rev(json!({"b": 1}));
        state.insert_snapshots(vec![doc.clone(), conflict1.clone(), conflict2.clone()]);

        // Default should include both normal and conflict documents
        {
            let result = state.list_documents(&Default::default()).unwrap();
            assert_eq!(result.items.len(), 2);
        }

        // Filter only_conflicts should return only the conflict document
        {
            let filter = Filter {
                only_conflicts: true,
                ..Default::default()
            };
            let result = state.list_documents(&filter).unwrap();
            assert_eq!(result.items.len(), 1);
            assert!(result.items[0].is_conflict());
        }
    }
}
