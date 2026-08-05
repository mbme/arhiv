mod download;
mod file_name_expert;

use anyhow::{Result, bail};
use baza::{
    BazaManager,
    entities::{Document, Id},
    schema::{Field, FieldType},
};
use baza_common::log;

use crate::{
    assets::download::Download,
    server::http::{is_http_url, is_image_url, parse_url},
};

/// Replaces supported remote asset URLs in asset reference fields with local asset ids.
///
/// Network fetching belongs to the application layer; `baza` receives completed
/// local files and stores them as encrypted asset blobs.
pub async fn materialize_asset_urls(
    baza_manager: &BazaManager,
    document: &mut Document,
) -> Result<()> {
    let document_expert = baza_manager.get_document_expert();
    let fields = document_expert.asset_ref_fields(&document.document_type)?;

    for field in fields {
        materialize_asset_field(baza_manager, document, field).await?;
    }

    Ok(())
}

async fn materialize_asset_field(
    baza_manager: &BazaManager,
    document: &mut Document,
    field: &Field,
) -> Result<()> {
    match field.field_type {
        FieldType::Ref(_) => {
            let Some(value) = document.data.get_str(field.name) else {
                return Ok(());
            };

            let Some(asset_id) = materialize_asset_url(baza_manager, field.name, value).await?
            else {
                return Ok(());
            };

            log::info!(
                "Materialized remote asset URL in field {} as asset {}",
                field.name,
                asset_id
            );
            document.data.set(field.name, asset_id);
        }

        FieldType::RefList(_) => {
            let mut values = document
                .data
                .get_ref_list(field.name)?
                .unwrap_or_default()
                .into_iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>();

            for value in &mut values {
                let Some(asset_id) = materialize_asset_url(baza_manager, field.name, value).await?
                else {
                    continue;
                };

                log::info!(
                    "Materialized remote asset URL in field {} as asset {}",
                    field.name,
                    asset_id
                );
                *value = asset_id.to_string();
            }

            document.data.set(field.name, values);
        }

        _ => unreachable!("only ref fields might reference assets"),
    }

    Ok(())
}

async fn materialize_asset_url(
    baza_manager: &BazaManager,
    field_name: &str,
    value: &str,
) -> Result<Option<Id>> {
    let url = if let Ok(url) = parse_url(value) {
        url
    } else {
        return Ok(None);
    };

    if !is_http_url(&url) {
        return Ok(None);
    }

    if !is_image_url(&url) {
        log::warn!(
            "Rejected non-image remote asset URL in field {}: {}",
            field_name,
            value
        );
        bail!("Only image asset URLs are supported, got '{value}'");
    }

    log::info!(
        "Downloading remote asset URL from field {}: {}",
        field_name,
        value
    );

    let download_result = Download::new_in_dir(value, baza_manager.get_downloads_dir())?
        .start()
        .await?;

    let mut baza = baza_manager.open_mut()?;
    let asset = baza.create_asset_with_filename(
        &download_result.file_path,
        download_result.original_file_name.clone(),
    )?;

    log::info!(
        "Created asset {} from remote URL in field {} with filename {}",
        asset.id,
        field_name,
        download_result.original_file_name
    );

    Ok(Some(asset.id))
}
