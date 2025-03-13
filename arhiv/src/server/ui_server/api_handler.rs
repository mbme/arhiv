use std::{cmp::Ordering, fs, path::Path};

use anyhow::{bail, Context, Result};

use baza::{
    baza2::{Filter, StagingError, ValidationError},
    entities::{Document, DocumentType},
    markup::MarkupStr,
    schema::DataSchema,
    DocumentExpert,
};
use rs_utils::{
    ensure_dir_exists, get_symlink_target_path, is_readable, log, path_to_string,
    remove_file_if_exists,
};

use crate::{
    dto::{
        APIRequest, APIResponse, DirEntry, DocumentBackref, GetDocumentsResult,
        ListDocumentsResult, SaveDocumentErrors,
    },
    Arhiv,
};

pub async fn handle_api_request(arhiv: &Arhiv, request: APIRequest) -> Result<APIResponse> {
    let response = match request {
        APIRequest::ListDocuments {
            document_types,
            query,
            page,
        } => {
            let document_types = document_types.into_iter().map(DocumentType::new).collect();
            let filter = Filter {
                query,
                document_types,
                page,
            };

            let document_expert = arhiv.baza.get_document_expert();

            let baza = arhiv.baza.open()?;
            let page = baza.list_documents(&filter)?;

            let documents = page
                .items
                .into_iter()
                .map(|item| {
                    let asset_id = document_expert.get_cover_asset_id(item)?;

                    Ok(ListDocumentsResult {
                        title: document_expert.get_title(&item.document_type, &item.data)?,
                        cover: asset_id,
                        id: item.id.clone(),
                        document_type: item.document_type.clone().into(),
                        updated_at: item.updated_at,
                        data: item.data.clone(),
                    })
                })
                .collect::<Result<_>>()?;

            APIResponse::ListDocuments {
                has_more: page.has_more,
                documents,
            }
        }
        APIRequest::GetDocuments { ids } => {
            let baza = arhiv.baza.open()?;

            let schema = arhiv.baza.get_schema();

            let documents = ids
                .iter()
                .map(|id| baza.must_get_document(id))
                .collect::<Result<Vec<_>>>()?;

            APIResponse::GetDocuments {
                documents: documents_into_results(documents, schema)?,
            }
        }
        APIRequest::GetStatus {} => {
            let status = arhiv.get_status()?;

            APIResponse::GetStatus {
                status: status.to_string(),
            }
        }
        APIRequest::GetDocument { ref id } => {
            let baza = arhiv.baza.open()?;
            let document = baza.must_get_document(id)?;

            let document_expert = arhiv.baza.get_document_expert();

            let backrefs = baza
                .find_document_backrefs(id)
                .into_iter()
                .map(|id| {
                    let item = baza.must_get_document(&id)?;

                    Ok(DocumentBackref {
                        title: document_expert.get_title(&item.document_type, &item.data)?,
                        id: item.id.clone(),
                        document_type: item.document_type.clone().into(),
                    })
                })
                .collect::<Result<_>>()?;

            let collections = baza
                .find_document_collections(id)
                .into_iter()
                .map(|id| {
                    let item = baza.must_get_document(&id)?;

                    Ok(DocumentBackref {
                        title: document_expert.get_title(&item.document_type, &item.data)?,
                        id: item.id.clone(),
                        document_type: item.document_type.clone().into(),
                    })
                })
                .collect::<Result<_>>()?;

            let title = document_expert.get_title(&document.document_type, &document.data)?;

            let refs = document_expert.extract_refs(&document.document_type, &document.data)?;

            APIResponse::GetDocument {
                id: document.id.clone(),
                title,
                document_type: document.document_type.clone().into(),
                updated_at: document.updated_at,
                data: document.data.clone(),
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
            let mut baza = arhiv.baza.open_mut()?;

            let mut document = baza.must_get_document(&id)?.clone();

            document.data = data;

            if let Err(err) = baza.stage_document(document, &Some(lock_key)) {
                match err {
                    StagingError::Validation(validation_error) => APIResponse::SaveDocument {
                        errors: Some(validation_error.into()),
                    },
                    StagingError::Other(error) => return Err(error),
                }
            } else {
                baza.update_document_collections(&id, &collections)?;
                baza.save_changes()?;

                APIResponse::SaveDocument { errors: None }
            }
        }
        APIRequest::CreateDocument {
            document_type,
            data,
            collections,
        } => {
            let document_type = DocumentType::new(document_type);
            let document = Document::new_with_data(document_type, data);
            let id = document.id.clone();

            let mut baza = arhiv.baza.open_mut()?;

            if let Err(err) = baza.stage_document(document, &None) {
                match err {
                    StagingError::Validation(validation_error) => APIResponse::CreateDocument {
                        id: None,
                        errors: Some(validation_error.into()),
                    },
                    StagingError::Other(error) => return Err(error),
                }
            } else {
                baza.update_document_collections(&id, &collections)?;

                baza.save_changes()?;

                APIResponse::CreateDocument {
                    id: Some(id.clone()),
                    errors: None,
                }
            }
        }
        APIRequest::EraseDocument { ref id } => {
            let mut baza = arhiv.baza.open_mut()?;
            baza.erase_document(id)?;
            baza.save_changes()?;

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
                dir: path_to_string(dir),
                entries,
            }
        }
        APIRequest::CreateAsset {
            ref file_path,
            remove_file,
        } => {
            let mut baza = arhiv.baza.open_mut()?;
            let asset = baza.create_asset(file_path)?;
            baza.save_changes()?;

            if remove_file {
                log::debug!("CreateAsset: removing original file {file_path}");
                remove_file_if_exists(file_path)?;
            }

            APIResponse::CreateAsset { id: asset.id }
        }
        APIRequest::Commit {} => {
            let mut baza = arhiv.baza.open_mut()?;

            baza.commit()?;
            arhiv.img_cache.remove_stale_files()?;

            APIResponse::Commit {}
        }
        APIRequest::LockDocument { id } => {
            let mut baza = arhiv.baza.open_mut()?;

            let lock = baza.lock_document(&id, "UI editor lock".to_string())?;
            let lock_key = lock.get_key().clone();
            baza.save_changes()?;

            APIResponse::LockDocument { lock_key }
        }
        APIRequest::UnlockDocument {
            id,
            lock_key,
            force_unlock,
        } => {
            let mut baza = arhiv.baza.open_mut()?;

            if force_unlock.unwrap_or_default() {
                baza.unlock_document_without_key(&id)?;
            } else if let Some(lock_key) = lock_key {
                baza.unlock_document(&id, &lock_key)?;
            } else {
                bail!("Can't unlock document {id} without a key");
            }
            baza.save_changes()?;

            APIResponse::UnlockDocument {}
        }
        APIRequest::ReorderCollectionRefs {
            collection_id,
            id,
            new_pos,
        } => {
            let mut baza = arhiv.baza.open_mut()?;
            if baza.is_document_locked(&collection_id) {
                bail!("Collection {collection_id} is locked")
            }

            let mut collection = baza.must_get_document(&collection_id)?.clone();
            let document = baza.must_get_document(&id)?;
            let document_expert = arhiv.baza.get_document_expert();

            document_expert.reorder_refs(&mut collection, document, new_pos)?;

            baza.stage_document(collection, &None)?;

            baza.save_changes()?;

            APIResponse::ReorderCollectionRefs {}
        }
        APIRequest::CreateArhiv { password } => {
            if arhiv.baza.storage_exists()? {
                bail!("Arhiv already exists");
            }

            log::info!("Creating new Arhiv");

            arhiv.baza.create(password.clone())?;
            Arhiv::save_password_to_keyring(password)?;
            arhiv.img_cache.init().await?;

            APIResponse::CreateArhiv {}
        }
        APIRequest::LockArhiv {} => {
            log::info!("Locking Arhiv");

            arhiv.baza.lock()?;
            Arhiv::erase_password_from_keyring()?;
            arhiv.img_cache.clear().await;

            APIResponse::LockArhiv {}
        }
        APIRequest::UnlockArhiv { password } => {
            log::info!("Unlocking Arhiv");

            arhiv.baza.unlock(password.clone())?;
            Arhiv::save_password_to_keyring(password)?;
            arhiv.img_cache.init().await?;

            APIResponse::UnlockArhiv {}
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
        let path = path_to_string(parent);

        let metadata = fs::metadata(&path).context("failed to read path metadata")?;

        result.push(DirEntry::Dir {
            name: "..".to_string(),
            path,
            is_readable: is_readable(&metadata),
        });
    }

    for entry in fs::read_dir(dir).context("failed to read directory entries")? {
        let entry = entry.context("failed to read an entry")?;

        let name = path_to_string(entry.file_name());

        // skip hidden files
        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let path = path_to_string(entry.path());

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

            let is_readable = metadata.as_ref().is_some_and(is_readable);

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
    documents: Vec<&Document>,
    schema: &DataSchema,
) -> Result<Vec<GetDocumentsResult>> {
    let document_expert = DocumentExpert::new(schema);

    documents
        .into_iter()
        .map(|item| {
            Ok(GetDocumentsResult {
                title: document_expert.get_title(&item.document_type, &item.data)?,
                id: item.id.clone(),
                document_type: item.document_type.clone().into(),
                updated_at: item.updated_at,
                data: item.data.clone(),
            })
        })
        .collect::<Result<_>>()
}
