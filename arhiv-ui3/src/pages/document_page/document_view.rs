use anyhow::Result;

use arhiv_core::{entities::Document, schema::Collection, Arhiv};
use rs_utils::server::Url;

use crate::components::{Catalog, DocumentDataViewer};

pub fn render_document_view(document: &Document, arhiv: &Arhiv, url: Url) -> Result<String> {
    let data_description = arhiv
        .get_schema()
        .get_data_description(&document.document_type)?;

    let mut content = DocumentDataViewer::new(document).render(arhiv)?;

    if let Collection::Type {
        document_type: item_type,
        field: _,
    } = data_description.collection_of
    {
        let catalog = Catalog::new(url)
            .with_type(item_type)
            .in_collection(&document.id)
            .render(arhiv)?;

        content.push_str("\n\n");
        content.push_str(&catalog);
    };

    Ok(content)
}
