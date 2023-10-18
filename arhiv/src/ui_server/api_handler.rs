use std::{cmp::Ordering, fs, path::Path};

use anyhow::{Context, Result};

use baza::{
    entities::{Document, DocumentClass, ERASED_DOCUMENT_TYPE},
    markup::MarkupStr,
    schema::{create_attachment, DataSchema},
    validator::{ValidationError, Validator},
    DocumentExpert, Filter,
};
use rs_utils::{
    decode_base64, ensure_dir_exists, get_home_dir, get_symlink_target_path, is_readable,
    path_to_string, TempFile,
};
use scraper::ScraperOptions;

use crate::{
    dto::{
        APIRequest, APIResponse, DirEntry, DocumentBackref, ListDocumentsResult, SaveDocumentErrors,
    },
    Arhiv,
};

const PAGE_SIZE: u8 = 10;

pub async fn handle_api_request(arhiv: &Arhiv, request: APIRequest) -> Result<APIResponse> {
    let response = match request {
        APIRequest::ListDocuments {
            document_types,
            query,
            page,
        } => {
            let mut filter = Filter::default()
                .search(query)
                .page_size(PAGE_SIZE)
                .on_page(page)
                .skip_erased(true)
                .recently_updated_first();

            if !document_types.is_empty() {
                if document_types.contains(&ERASED_DOCUMENT_TYPE.into()) {
                    filter = filter.skip_erased(false);
                }

                filter = filter.with_document_types(document_types);
            }

            let schema = arhiv.baza.get_schema();
            let page = arhiv.baza.get_connection()?.list_documents(&filter)?;

            APIResponse::ListDocuments {
                has_more: page.has_more,
                documents: documents_into_results(page.items, schema)?,
            }
        }
        APIRequest::GetDocuments { ids } => {
            let conn = arhiv.baza.get_connection()?;

            let schema = arhiv.baza.get_schema();

            let documents = ids
                .iter()
                .map(|id| conn.must_get_document(id))
                .collect::<Result<Vec<_>>>()?;

            APIResponse::GetDocuments {
                documents: documents_into_results(documents, schema)?,
            }
        }
        APIRequest::GetStatus {} => {
            let status = arhiv.get_status().await?;

            APIResponse::GetStatus {
                status: status.to_string(),
            }
        }
        APIRequest::GetDocument { ref id } => {
            let conn = arhiv.baza.get_connection()?;
            let document = conn.must_get_document(id)?;

            let schema = arhiv.baza.get_schema();
            let document_expert = DocumentExpert::new(schema);

            let backrefs = conn
                .list_document_backrefs(id)?
                .into_iter()
                .map(|item| {
                    Ok(DocumentBackref {
                        title: document_expert.get_title(&item.class, &item.data)?,
                        id: item.id,
                        document_type: item.class.document_type,
                        subtype: item.class.subtype,
                    })
                })
                .collect::<Result<_>>()?;

            let collections = conn
                .list_document_collections(id)?
                .into_iter()
                .map(|item| {
                    Ok(DocumentBackref {
                        title: document_expert.get_title(&item.class, &item.data)?,
                        id: item.id,
                        document_type: item.class.document_type,
                        subtype: item.class.subtype,
                    })
                })
                .collect::<Result<_>>()?;

            let title = document_expert.get_title(&document.class, &document.data)?;

            APIResponse::GetDocument {
                id: document.id,
                title,
                document_type: document.class.document_type,
                subtype: document.class.subtype,
                updated_at: document.updated_at,
                data: document.data,
                backrefs,
                collections,
            }
        }
        APIRequest::ParseMarkup { markup } => {
            let markup: MarkupStr = markup.into();

            let ast = markup.get_ast()?;
            let ast = serde_json::to_value(ast).context("failed to serialize ast")?;

            APIResponse::ParseMarkup { ast }
        }
        APIRequest::SaveDocument {
            id,
            subtype,
            data,
            collections,
        } => {
            let mut tx = arhiv.baza.get_tx()?;

            let mut document = tx.must_get_document(&id)?;

            let prev_data = document.data;

            document.class.set_subtype(subtype);
            document.data = data;

            let validator = Validator::new(&tx);
            let validation_result = validator.validate(&document, Some(&prev_data));

            let errors = if let Err(error) = validation_result {
                Some(error.into())
            } else {
                tx.stage_document(&mut document)?;

                None
            };

            let schema = arhiv.baza.get_schema();
            let document_expert = DocumentExpert::new(schema);

            let mut old_collections = tx.list_document_collections(&id)?;

            for old_collection in &mut old_collections {
                if !collections.contains(&old_collection.id) {
                    document_expert.remove_document_from_collection(&document, old_collection)?;
                    tx.stage_document(old_collection)?;
                }
            }

            let old_collections_ids = old_collections
                .iter()
                .map(|collection| &collection.id)
                .collect::<Vec<_>>();

            for collection_id in collections {
                if !old_collections_ids.contains(&&collection_id) {
                    let mut collection = tx.must_get_document(&collection_id)?;
                    document_expert.add_document_to_collection(&document, &mut collection)?;
                    tx.stage_document(&mut collection)?;
                }
            }

            if errors.is_none() {
                tx.commit()?;
            }

            APIResponse::SaveDocument { errors }
        }
        APIRequest::CreateDocument {
            document_type,
            subtype,
            data,
            collections,
        } => {
            let document_type = DocumentClass::new(document_type, subtype);
            let mut document = Document::new_with_data(document_type, data);

            let mut tx = arhiv.baza.get_tx()?;

            let validator = Validator::new(&tx);
            let validation_result = validator.validate(&document, None);

            let (id, errors) = if let Err(error) = validation_result {
                (None, Some(error.into()))
            } else {
                tx.stage_document(&mut document)?;

                (Some(document.id.clone()), None)
            };

            let schema = arhiv.baza.get_schema();
            let document_expert = DocumentExpert::new(schema);

            for collection_id in collections {
                let mut collection = tx.must_get_document(&collection_id)?;
                document_expert.add_document_to_collection(&document, &mut collection)?;
                tx.stage_document(&mut collection)?;
            }

            if errors.is_none() {
                tx.commit()?;
            }

            APIResponse::CreateDocument { id, errors }
        }
        APIRequest::EraseDocument { ref id } => {
            let tx = arhiv.baza.get_tx()?;
            tx.erase_document(id)?;
            tx.commit()?;

            APIResponse::EraseDocument {}
        }
        APIRequest::ListDir { dir, show_hidden } => {
            let dir = dir.unwrap_or_else(|| get_home_dir().unwrap_or_else(|| "/".to_string()));
            ensure_dir_exists(&dir)?;

            let dir =
                fs::canonicalize(&dir).context(format!("failed to canonicalize path '{dir}'"))?;

            let mut entries: Vec<DirEntry> = list_entries(&dir, show_hidden).context(format!(
                "failed to list entries in a dir '{}'",
                dir.display()
            ))?;
            sort_entries(&mut entries);

            APIResponse::ListDir {
                dir: path_to_string(dir)?,
                entries,
            }
        }
        APIRequest::CreateAttachment {
            ref file_path,
            move_file,
        } => {
            let mut tx = arhiv.baza.get_tx()?;
            let attachment = create_attachment(&mut tx, file_path, move_file, None)?;
            tx.commit()?;

            APIResponse::CreateAttachment { id: attachment.id }
        }
        APIRequest::UploadFile {
            ref base64_data,
            file_name,
        } => {
            let temp_file = TempFile::new();
            temp_file.create_file()?;
            temp_file
                .write(&decode_base64(base64_data)?)
                .context("failed to write data into temp file")?;

            let mut tx = arhiv.baza.get_tx()?;

            let attachment = create_attachment(&mut tx, &temp_file.path, true, Some(file_name))?;

            tx.commit()?;

            APIResponse::UploadFile { id: attachment.id }
        }
        APIRequest::Scrape { url } => {
            let documents = arhiv
                .scrape(
                    url,
                    ScraperOptions {
                        manual: false,
                        emulate_mobile: false,
                        debug: false,
                        screenshot_file: None,
                    },
                )
                .await?;

            let schema = arhiv.baza.get_schema();

            APIResponse::Scrape {
                documents: documents_into_results(documents, schema)?,
            }
        }
        APIRequest::CommitOrSync {} => {
            let has_staged_documents = arhiv.baza.get_connection()?.has_staged_documents()?;

            if has_staged_documents {
                let mut tx = arhiv.baza.get_tx()?;

                tx.commit_staged_documents()?;

                tx.commit()?;
            } else {
                arhiv.sync().await?;
            }

            APIResponse::CommitOrSync {}
        }
        APIRequest::GetIsModified {} => {
            let has_staged_documents = arhiv.baza.get_connection()?.has_staged_documents()?;

            APIResponse::GetIsModified {
                is_modified: has_staged_documents,
            }
        }
    };

    Ok(response)
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

        let metadata = fs::metadata(&path).context("failed to read path metadata")?;

        result.push(DirEntry::Dir {
            name: "..".to_string(),
            path,
            is_readable: is_readable(&metadata),
        });
    }

    for entry in fs::read_dir(dir).context("failed to read directory entries")? {
        let entry = entry.context("failed to read an entry")?;

        let name = path_to_string(entry.file_name())?;

        // skip hidden files
        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let path = path_to_string(entry.path())?;

        let file_type = entry.file_type()?;
        let metadata =
            fs::symlink_metadata(&path).context(format!("failed to read metadata for '{path}'"))?;

        if metadata.is_dir() {
            result.push(DirEntry::Dir {
                name,
                path,
                is_readable: is_readable(&metadata),
            });
            continue;
        }

        if file_type.is_symlink() {
            let links_to = get_symlink_target_path(&path)?;
            let metadata = fs::metadata(&path).ok();

            let is_readable = metadata.as_ref().map_or(false, is_readable);

            let size = metadata.and_then(|metadata| metadata.is_file().then_some(metadata.len()));

            result.push(DirEntry::Symlink {
                name,
                path,
                is_readable,
                links_to,
                size,
            });
            continue;
        }

        result.push(DirEntry::File {
            name,
            path,
            is_readable: is_readable(&metadata),
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
    let document_expert = DocumentExpert::new(schema);

    documents
        .into_iter()
        .map(|item| {
            Ok(ListDocumentsResult {
                title: document_expert.get_title(&item.class, &item.data)?,
                id: item.id,
                document_type: item.class.document_type,
                subtype: item.class.subtype,
                updated_at: item.updated_at,
            })
        })
        .collect::<Result<_>>()
}
