use std::{cmp::Ordering, fs, path::Path};

use anyhow::{Context, Result, anyhow, bail, ensure};
use serde::Serialize;

use baza::{
    DocumentExpert, Filter, StagingError, ValidationError,
    entities::{Document, DocumentType},
    markup::MarkupStr,
    schema::DataSchema,
};
use rs_utils::{
    Timestamp, ensure_dir_exists, get_crate_version, get_symlink_target_path,
    image::generate_qrcode_svg, is_readable, log, path_to_string, remove_file_if_exists,
    render_template, to_base64,
};

use crate::ui::dto::{
    APIRequest, APIResponse, DirEntry, DocumentBackref, GetDocumentsResult, ListDocumentsResult,
    SaveDocumentErrors,
};

use super::ServerContext;

pub async fn handle_api_request(ctx: &ServerContext, request: APIRequest) -> Result<APIResponse> {
    let arhiv = &ctx.arhiv;

    let response = match request {
        APIRequest::ListDocuments {
            document_types,
            query,
            page,
            only_conflicts,
        } => {
            let document_types = document_types.into_iter().map(DocumentType::new).collect();
            let filter = Filter {
                query,
                document_types,
                page,
                only_conflicts,
            };

            let document_expert = arhiv.baza.get_document_expert();

            let baza = arhiv.baza.open()?;
            let page = baza.list_documents(&filter)?;

            let documents = page
                .items
                .into_iter()
                .map(|head| {
                    let doc = head.get_single_document();

                    let asset_id = document_expert.get_cover_asset_id(doc)?;

                    Ok(ListDocumentsResult {
                        title: document_expert.get_title(&doc.document_type, &doc.data)?,
                        cover: asset_id,
                        id: doc.id.clone(),
                        document_type: doc.document_type.clone().into(),
                        updated_at: doc.updated_at,
                        data: doc.data.clone(),
                        has_conflict: head.is_conflict(),
                    })
                })
                .collect::<Result<_>>()?;

            APIResponse::ListDocuments {
                has_more: page.has_more,
                documents,
                total: page.total,
            }
        }
        APIRequest::GetDocuments {
            ids,
            ignore_missing,
        } => {
            let ignore_missing = ignore_missing.unwrap_or_default();

            let baza = arhiv.baza.open()?;

            let schema = arhiv.baza.get_schema();

            let mut documents = Vec::with_capacity(ids.len());
            for id in &ids {
                let head = baza.get_document(id);

                if let Some(head) = head {
                    documents.push(head.get_single_document());
                } else if !ignore_missing {
                    bail!("Can't find document {id}");
                }
            }

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
            let head = baza.get_document(id).context("Document is missing")?;
            let document = head.get_single_document();
            let snapshots_count = head.get_snapshots_count();

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
                snapshots_count,
                has_conflict: head.is_conflict(),
                is_staged: head.is_staged(),
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
            let mut document = {
                let baza = arhiv.baza.open()?;

                let mut document = baza.must_get_document(&id)?.clone();

                document.data = data;

                document
            };

            let document_expert = arhiv.baza.get_document_expert();
            document_expert
                .prepare_assets(&mut document, &arhiv.baza)
                .await?;

            let mut baza = arhiv.baza.open_mut()?;
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
            let mut document = Document::new_with_data(document_type, data);
            let id = document.id.clone();

            let document_expert = arhiv.baza.get_document_expert();
            document_expert
                .prepare_assets(&mut document, &arhiv.baza)
                .await?;

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

            let committed_ids = baza.commit()?;

            ctx.img_cache.remove_stale_files(&baza)?;

            APIResponse::Commit { committed_ids }
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
            arhiv.create(password)?;
            ctx.img_cache.init(&arhiv.baza).await?;

            APIResponse::CreateArhiv {}
        }
        APIRequest::LockArhiv {} => {
            arhiv.lock()?;
            ctx.img_cache.clear().await;

            APIResponse::LockArhiv {}
        }
        APIRequest::UnlockArhiv { password } => {
            if let Some(password) = password {
                arhiv.unlock(password)?;
            } else {
                let unlocked = arhiv.unlock_using_keyring()?;
                ensure!(unlocked, "Failed to unlock Arhiv: no password in Keyring");
            }

            ctx.img_cache.init(&arhiv.baza).await?;

            APIResponse::UnlockArhiv {}
        }
        APIRequest::ImportKey {
            encrypted_key,
            password,
        } => {
            let was_locked = arhiv.baza.is_locked();

            arhiv.baza.import_key(encrypted_key, password.clone())?;

            if was_locked {
                arhiv.unlock(password)?;
                ctx.img_cache.init(&arhiv.baza).await?;
            }

            APIResponse::ImportKey {}
        }
        APIRequest::ExportKey {
            password,
            export_password,
        } => {
            let key = arhiv.baza.export_key(password, export_password)?;
            let qrcode_svg_data = generate_qrcode_svg(key.as_bytes())?;
            let qrcode_svg_base64 = to_base64(&qrcode_svg_data);

            let date = Timestamp::now()
                .format_time("[month repr:short] [day padding:space] [year]")
                .expect("must be valid format");

            let html_page = get_export_key_html_page(&PageProps {
                arhiv_version: get_crate_version().to_string(),
                key: &key,
                qrcode_svg_base64: &qrcode_svg_base64,
                date,
            })?;

            APIResponse::ExportKey {
                key,
                qrcode_svg_base64,
                html_page,
            }
        }
        APIRequest::CountConflicts {} => {
            let conflicts_count = arhiv.baza.open()?.iter_conflicts().count();

            APIResponse::CountConflicts { conflicts_count }
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

#[derive(Serialize)]
struct PageProps<'a> {
    arhiv_version: String,
    key: &'a str,
    qrcode_svg_base64: &'a str,
    date: String,
}

fn get_export_key_html_page(props: &PageProps) -> Result<String> {
    let source = include_str!("./export-key.html");
    let props = serde_json::to_value(props).context("Failed to serialize PageProps")?;

    render_template(source, &props).map_err(|err| anyhow!("failed to render export-key: {err}"))
}
