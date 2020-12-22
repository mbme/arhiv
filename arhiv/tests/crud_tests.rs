use anyhow::*;
use arhiv::DocumentFilter;
use serde_json::json;
pub use utils::*;

mod utils;

#[test]
fn test_crud() -> Result<()> {
    let arhiv = new_prime();

    let original_data = json!({ "test": "test" });

    // CREATE
    let id = {
        let document = new_document(original_data.clone());
        arhiv.stage_document(document.clone(), vec![])?;
        assert_eq!(
            arhiv.list_documents(DocumentFilter::default())?.items.len(),
            1
        );

        document.id
    };

    // READ
    {
        let other_document = arhiv.get_document(&id)?.unwrap();

        assert_eq!(other_document.data, original_data);
        assert_eq!(other_document.rev.is_staged(), true);
    }

    // UPDATE
    {
        let mut other_document = arhiv.get_document(&id)?.unwrap();
        other_document.data = json!({ "test": "1" });
        arhiv.stage_document(other_document.clone(), vec![])?;

        assert_eq!(arhiv.get_document(&id)?.unwrap().data, other_document.data);
    }

    // DELETE
    {
        assert_eq!(
            arhiv.list_documents(DocumentFilter::default())?.items.len(),
            1
        );
        let mut other_document = arhiv.get_document(&id)?.unwrap();
        other_document.archived = true;
        arhiv.stage_document(other_document, vec![])?;

        assert_eq!(arhiv.get_document(&id)?.unwrap().archived, true);
        assert_eq!(
            arhiv.list_documents(DocumentFilter::default())?.items.len(),
            0
        );
    }

    Ok(())
}
