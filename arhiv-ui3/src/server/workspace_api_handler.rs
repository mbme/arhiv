use std::{cmp::Ordering, fs, path::Path};

use anyhow::{Context, Result};
use hyper::{body::Bytes, StatusCode};

use arhiv_core::{
    definitions::Attachment,
    entities::{Document, ERASED_DOCUMENT_TYPE},
    markup::MarkupStr,
    schema::DataSchema,
    Arhiv, Filter, ScraperOptions, ValidationError, Validator,
};
use rs_utils::{
    ensure_dir_exists, get_home_dir, http_server::ServerResponse, is_readable, path_to_string,
};

use crate::dto::{
    DirEntry, DocumentBackref, ListDocumentsResult, SaveDocumentErrors, WorkspaceRequest,
    WorkspaceResponse,
};

use super::utils::render_json;

const PAGE_SIZE: u8 = 10;

pub async fn handle_workspace_api_request(arhiv: &Arhiv, body: &Bytes) -> ServerResponse {
    let request: WorkspaceRequest =
        serde_json::from_slice(body).context("failed to parse request")?;

    let response = match request {
        WorkspaceRequest::ListDocuments {
            collection_id,
            document_type,
            query,
            page,
        } => {
            let mut filter = Filter::default()
                .search(query)
                .page_size(PAGE_SIZE)
                .on_page(page)
                .skip_erased(true)
                .recently_updated_first();

            if let Some(collection_id) = collection_id {
                filter = filter.with_collection_ref(collection_id);
            }

            if let Some(document_type) = document_type {
                if document_type == ERASED_DOCUMENT_TYPE {
                    filter = filter.skip_erased(false);
                }

                filter = filter.with_document_type(document_type);
            }

            let schema = arhiv.get_schema();
            let page = arhiv.list_documents(filter)?;

            WorkspaceResponse::ListDocuments {
                has_more: page.has_more,
                documents: documents_into_results(page.items, schema)?,
            }
        }
        WorkspaceRequest::GetStatus {} => {
            let status = arhiv.get_status()?;

            WorkspaceResponse::GetStatus {
                status: status.to_string(),
            }
        }
        WorkspaceRequest::GetDocument { ref id } => {
            let document = arhiv.must_get_document(id)?;

            let schema = arhiv.get_schema();

            let backrefs = arhiv
                .list_documents(Filter::all_backrefs(id))?
                .items
                .into_iter()
                .map(|item| {
                    Ok(DocumentBackref {
                        title: schema.get_title(&item)?,
                        id: item.id,
                        document_type: item.document_type,
                        subtype: item.subtype,
                    })
                })
                .collect::<Result<_>>()?;

            let title = schema.get_title(&document)?;

            WorkspaceResponse::GetDocument {
                id: document.id,
                title,
                document_type: document.document_type,
                subtype: document.subtype,
                updated_at: document.updated_at,
                data: document.data,
                backrefs,
            }
        }
        WorkspaceRequest::ParseMarkup { markup } => {
            let markup: MarkupStr = markup.into();

            let ast = markup.get_ast()?;
            let ast = serde_json::to_value(ast).context("failed to serialize ast")?;

            WorkspaceResponse::ParseMarkup { ast }
        }
        WorkspaceRequest::SaveDocument { id, subtype, data } => {
            let mut document = arhiv.must_get_document(&id)?;

            let prev_data = document.data;

            document.subtype = subtype;
            document.data = data;

            let tx = arhiv.get_tx()?;
            let validation_result = Validator::new(&tx).validate(&document, Some(&prev_data));

            let errors = if let Err(error) = validation_result {
                Some(error.into())
            } else {
                tx.stage_document(&mut document)?;

                None
            };

            tx.commit()?;

            WorkspaceResponse::SaveDocument { errors }
        }
        WorkspaceRequest::CreateDocument {
            document_type,
            subtype,
            data,
        } => {
            let mut document = Document::new_with_data(&document_type, &subtype, data);

            let tx = arhiv.get_tx()?;
            let validation_result = Validator::new(&tx).validate(&document, None);

            let (id, errors) = if let Err(error) = validation_result {
                (None, Some(error.into()))
            } else {
                tx.stage_document(&mut document)?;

                (Some(document.id), None)
            };

            tx.commit()?;

            WorkspaceResponse::CreateDocument { id, errors }
        }
        WorkspaceRequest::EraseDocument { ref id } => {
            let tx = arhiv.get_tx()?;
            tx.erase_document(id)?;
            tx.commit()?;

            WorkspaceResponse::EraseDocument {}
        }
        WorkspaceRequest::ListDir { dir, show_hidden } => {
            let dir = dir.unwrap_or_else(|| get_home_dir().unwrap_or_else(|| "/".to_string()));
            ensure_dir_exists(&dir)?;

            let dir = fs::canonicalize(dir)?;

            let mut entries: Vec<DirEntry> = list_entries(&dir, show_hidden)?;
            sort_entries(&mut entries);

            WorkspaceResponse::ListDir {
                dir: path_to_string(dir)?,
                entries,
            }
        }
        WorkspaceRequest::CreateAttachment { ref file_path } => {
            let mut tx = arhiv.get_tx()?;
            let attachment = Attachment::create(file_path, false, &mut tx)?;
            tx.commit()?;

            WorkspaceResponse::CreateAttachment { id: attachment.id }
        }
        WorkspaceRequest::Scrape { url } => {
            let documents = arhiv
                .scrape(
                    url,
                    ScraperOptions {
                        manual: false,
                        emulate_mobile: false,
                        debug: false,
                    },
                )
                .await?;

            let schema = arhiv.get_schema();

            WorkspaceResponse::Scrape {
                documents: documents_into_results(documents, schema)?,
            }
        }
    };

    let content = serde_json::to_string(&response).context("failed to serialize response")?;

    render_json(StatusCode::OK, content)
}

impl From<ValidationError> for SaveDocumentErrors {
    fn from(val: ValidationError) -> Self {
        match val {
            ValidationError::DocumentError { errors } => SaveDocumentErrors {
                document_errors: errors,
                ..Default::default()
            },
            ValidationError::FieldError { errors } => SaveDocumentErrors {
                field_errors: errors,
                ..Default::default()
            },
        }
    }
}

fn list_entries(dir: &Path, show_hidden: bool) -> Result<Vec<DirEntry>> {
    let mut result = vec![];

    if let Some(parent) = dir.parent() {
        let path = path_to_string(parent)?;

        let metadata = fs::metadata(&path)?;

        result.push(DirEntry::Dir {
            name: "..".to_string(),
            path,
            is_readable: is_readable(&metadata),
        });
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;

        let name = path_to_string(entry.file_name())?;

        // skip hidden files
        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let path = path_to_string(entry.path())?;

        let file_type = entry.file_type()?;
        let metadata = fs::metadata(&path)?;

        let is_readable = is_readable(&metadata);

        if metadata.is_dir() {
            result.push(DirEntry::Dir {
                name,
                path,
                is_readable,
            });
            continue;
        }

        if file_type.is_symlink() {
            let link_path = fs::canonicalize(&path)?;
            let link_path = path_to_string(link_path)?;

            let size = metadata.is_file().then_some(metadata.len());

            result.push(DirEntry::Symlink {
                name,
                path,
                is_readable,
                links_to: link_path,
                size,
            });
            continue;
        }

        result.push(DirEntry::File {
            name,
            path,
            is_readable,
            size: metadata.len(),
        });
    }

    Ok(result)
}

fn sort_entries(entries: &mut [DirEntry]) {
    entries.sort_by(|a, b| {
        match (
            matches!(a, DirEntry::Dir { .. }),
            matches!(b, DirEntry::Dir { .. }),
        ) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.get_name().cmp(b.get_name()),
        }
    });
}

fn documents_into_results(
    documents: Vec<Document>,
    schema: &DataSchema,
) -> Result<Vec<ListDocumentsResult>> {
    documents
        .into_iter()
        .map(|item| {
            Ok(ListDocumentsResult {
                title: schema.get_title(&item)?,
                id: item.id,
                document_type: item.document_type,
                subtype: item.subtype,
                updated_at: item.updated_at,
            })
        })
        .collect::<Result<_>>()
}
