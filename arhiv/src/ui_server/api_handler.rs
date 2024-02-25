use std::{cmp::Ordering, fs, path::Path};

use anyhow::{bail, Context, Result};

use baza::{
    entities::{Document, DocumentType, ERASED_DOCUMENT_TYPE},
    markup::MarkupStr,
    schema::{create_attachment, Attachment, DataSchema},
    validator::{ValidationError, Validator},
    DocumentExpert, Filter,
};
use rs_utils::{
    decode_base64, ensure_dir_exists, get_symlink_target_path, is_readable, path_to_string,
    TempFile,
};

use crate::{
    dto::{
        APIRequest, APIResponse, DirEntry, DocumentBackref, GetDocumentsResult,
        ListDocumentsResult, SaveDocumentErrors,
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
            let document_expert = DocumentExpert::new(schema);

            let conn = arhiv.baza.get_connection()?;
            let page = conn.list_documents(&filter)?;

            let documents = page
                .items
                .into_iter()
                .map(|item| {
                    let attachment_id = document_expert.get_cover_attachment_id(&item)?;

                    let cover = if let Some(ref attachment_id) = attachment_id {
                        let attachment: Attachment =
                            conn.must_get_document(attachment_id)?.convert()?;

                        Some(attachment.data.blob)
                    } else {
                        None
                    };

                    Ok(ListDocumentsResult {
                        title: document_expert.get_title(&item.document_type, &item.data)?,
                        cover,
                        id: item.id,
                        document_type: item.document_type.into(),
                        updated_at: item.updated_at,
                        data: item.data,
                    })
                })
                .collect::<Result<_>>()?;

            APIResponse::ListDocuments {
                has_more: page.has_more,
                documents,
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
                        title: document_expert.get_title(&item.document_type, &item.data)?,
                        id: item.id,
                        document_type: item.document_type.into(),
                    })
                })
                .collect::<Result<_>>()?;

            let collections = conn
                .list_document_collections(id)?
                .into_iter()
                .map(|item| {
                    Ok(DocumentBackref {
                        title: document_expert.get_title(&item.document_type, &item.data)?,
                        id: item.id,
                        document_type: item.document_type.into(),
                    })
                })
                .collect::<Result<_>>()?;

            let title = document_expert.get_title(&document.document_type, &document.data)?;

            let refs = document_expert.extract_refs(&document.document_type, &document.data)?;

            APIResponse::GetDocument {
                id: document.id,
                title,
                document_type: document.document_type.into(),
                updated_at: document.updated_at,
                data: document.data,
                backrefs,
                collections,
                refs: refs.get_all_document_refs(),
            }
        }
        APIRequest::ParseMarkup { markup } => {
            let markup: MarkupStr = markup.into();

            let ast = markup.get_ast()?;
            let ast = serde_json::to_value(ast).context("failed to serialize ast")?;

            APIResponse::ParseMarkup { ast }
        }
        APIRequest::SaveDocument {
            lock_key,
            id,
            data,
            collections,
        } => {
            let mut tx = arhiv.baza.get_tx()?;

            let mut document = tx.must_get_document(&id)?;

            let prev_data = document.data;

            document.data = data;

            let validator = Validator::new(&tx);
            let validation_result = validator.validate(&document, Some(&prev_data));

            let errors = if let Err(error) = validation_result {
                Some(error.into())
            } else {
                tx.stage_document(&mut document, Some(lock_key))?;

                None
            };

            if errors.is_none() {
                tx.update_document_collections(&document, &collections)?;

                tx.commit()?;
            }

            APIResponse::SaveDocument { errors }
        }
        APIRequest::CreateDocument {
            document_type,
            data,
            collections,
        } => {
            let document_type = DocumentType::new(document_type);
            let mut document = Document::new_with_data(document_type, data);

            let mut tx = arhiv.baza.get_tx()?;

            let validator = Validator::new(&tx);
            let validation_result = validator.validate(&document, None);

            let (id, errors) = if let Err(error) = validation_result {
                (None, Some(error.into()))
            } else {
                tx.stage_document(&mut document, None)?;

                (Some(document.id.clone()), None)
            };

            if errors.is_none() {
                tx.update_document_collections(&document, &collections)?;

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
            let dir = dir.unwrap_or_else(|| arhiv.get_file_browser_root_dir().to_string());
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
        #[cfg(feature = "scraper")]
        APIRequest::Scrape { url } => {
            use scraper::ScraperOptions;
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
        APIRequest::Commit {} => {
            let mut tx = arhiv.baza.get_tx()?;

            tx.commit_staged_documents()?;

            tx.commit()?;

            APIResponse::Commit {}
        }
        APIRequest::Sync {} => {
            arhiv.sync().await?;

            APIResponse::Sync {}
        }
        APIRequest::GetSaveState {} => {
            let conn = arhiv.baza.get_connection()?;

            let has_locks = !conn.list_document_locks()?.is_empty();
            let has_staged_documents = conn.has_staged_documents()?;
            let has_agents = arhiv.has_sync_agents();

            APIResponse::GetSaveState {
                can_commit: has_staged_documents && !has_locks,
                can_sync: !has_staged_documents && !has_locks && has_agents,
            }
        }
        APIRequest::LockDocument { id } => {
            let mut tx = arhiv.baza.get_tx()?;
            let lock = tx.lock_document(&id, "UI editor lock".to_string())?;
            tx.commit()?;

            APIResponse::LockDocument {
                lock_key: lock.get_key().clone(),
            }
        }
        APIRequest::UnlockDocument { id, lock_key } => {
            let mut tx = arhiv.baza.get_tx()?;
            tx.unlock_document(&id, &lock_key)?;
            tx.commit()?;

            APIResponse::UnlockDocument {}
        }
        APIRequest::ReorderCollectionRefs {
            collection_id,
            id,
            new_pos,
        } => {
            let mut tx = arhiv.baza.get_tx()?;
            if tx.is_document_locked(&collection_id)? {
                bail!("Collection {collection_id} is locked")
            }

            let mut collection = tx.must_get_document(&collection_id)?;
            let document = tx.must_get_document(&id)?;
            let document_expert = arhiv.baza.get_document_expert();

            document_expert.reorder_refs(&mut collection, &document, new_pos)?;

            tx.stage_document(&mut collection, None)?;

            tx.commit()?;

            APIResponse::ReorderCollectionRefs {}
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
) -> Result<Vec<GetDocumentsResult>> {
    let document_expert = DocumentExpert::new(schema);

    documents
        .into_iter()
        .map(|item| {
            Ok(GetDocumentsResult {
                title: document_expert.get_title(&item.document_type, &item.data)?,
                id: item.id,
                document_type: item.document_type.into(),
                updated_at: item.updated_at,
                data: item.data,
            })
        })
        .collect::<Result<_>>()
}
