use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    fs,
    io::{BufReader, Write},
};

use anyhow::{ensure, Context, Result};
use serde::Deserialize;
use tokio::{sync::RwLock, time::Instant};

use baza::{entities::Id, schema::ASSET_TYPE, Baza, BazaManager};
use rs_utils::{
    create_dir_if_not_exist, create_file_reader, create_file_writer, file_exists, format_bytes,
    get_file_name, image::scale_image_file, list_files, log, read_all, Timestamp,
};

#[derive(Deserialize, Clone)]
pub struct ImageParams {
    pub max_w: Option<u32>,
    pub max_h: Option<u32>,
}

impl ImageParams {
    pub fn is_empty(&self) -> bool {
        self.max_w.is_none() && self.max_h.is_none()
    }
}

impl Display for ImageParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}x{}",
            self.max_w
                .map_or("_".to_string(), |max_w| max_w.to_string()),
            self.max_h
                .map_or("_".to_string(), |max_h| max_h.to_string()),
        )
    }
}

impl Debug for ImageParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

const CACHE_SIZE_LIMIT: usize = 30 * 1024 * 1024; // 30Mb

#[derive(Clone)]
struct ImageInfo {
    access_time: Timestamp,
    size: usize,
}

pub struct ScaledImagesCache {
    root_dir: String,
    usage_info: RwLock<HashMap<String, ImageInfo>>,
}

impl ScaledImagesCache {
    pub fn new(root_dir: String) -> Self {
        ScaledImagesCache {
            root_dir,
            usage_info: Default::default(),
        }
    }

    pub async fn init(&self, baza_manager: &BazaManager) -> Result<()> {
        log::info!("Initializing Scaled images cache");

        ensure!(
            baza_manager.is_unlocked(),
            "Baza must be unlocked to init image cache service"
        );

        create_dir_if_not_exist(&self.root_dir)?;

        {
            let baza = baza_manager.open()?;
            self.remove_stale_files(&baza)?;
        }

        self.init_usage_info().await?;

        Ok(())
    }

    pub async fn clear(&self) {
        let mut usage_info = self.usage_info.write().await;
        usage_info.clear();
    }

    async fn init_usage_info(&self) -> Result<()> {
        let mut usage_info = self.usage_info.write().await;

        let files = list_files(&self.root_dir)?;
        for file_path in &files {
            let metadata = fs::metadata(file_path)?;
            let file_name = get_file_name(file_path).to_string();
            let size = metadata.len() as usize;

            let access_time = metadata
                .accessed()
                .or_else(|_| metadata.created())
                .map_or(Timestamp::MIN, |time| time.into());

            usage_info.insert(file_name, ImageInfo { access_time, size });
        }

        let total_size: usize = usage_info.values().map(|info| info.size).sum();

        log::info!(
            "Found {} cache files ({} total)",
            files.len(),
            format_bytes(total_size as u64)
        );

        Ok(())
    }

    // on open & on commit
    pub fn remove_stale_files(&self, baza: &Baza) -> Result<()> {
        log::debug!("Checking for stale cache files in {}", self.root_dir);

        let mut files_removed = 0;
        for file_path in list_files(&self.root_dir)? {
            let file_name = get_file_name(&file_path);

            if let Some(asset_id) = file_name.split('.').next() {
                let asset_id: Id = asset_id.into();

                let is_valid_asset = baza
                    .get_document(&asset_id)
                    .is_some_and(|head| head.get_type() == &ASSET_TYPE);

                if !is_valid_asset {
                    log::info!("Removing stale cache file: {file_name}");
                    fs::remove_file(&file_path)?;
                    files_removed += 1;
                }
            } else {
                log::warn!("Unexpected file in cache dir: {file_name}; removing");
                fs::remove_file(&file_path)?;
                files_removed += 1;
            }
        }

        if files_removed > 0 {
            log::info!("Removed {files_removed} cache files");
        }

        Ok(())
    }

    async fn remove_files_if_necessary(&self) -> Result<()> {
        let total_size: usize = {
            let usage_info = self.usage_info.read().await;

            usage_info.values().map(|info| info.size).sum()
        };

        if total_size <= CACHE_SIZE_LIMIT {
            return Ok(());
        }
        log::debug!("Cache size is too big: {}", format_bytes(total_size as u64));

        let mut usage_info = self.usage_info.write().await;
        let mut sorted_files: Vec<_> = usage_info.iter().collect();
        sorted_files.sort_by_key(|&(_, info)| info.access_time);
        let sorted_files: Vec<_> = sorted_files
            .iter()
            .map(|(cache_file_name, _)| (*cache_file_name).clone())
            .collect();

        let mut current_size = total_size;
        for cache_file_name in sorted_files {
            if current_size <= CACHE_SIZE_LIMIT {
                break;
            }

            let cache_file_path = format!("{}/{cache_file_name}", self.root_dir);

            let info = usage_info
                .remove(&cache_file_name)
                .expect("Info must be present");
            current_size -= info.size;

            log::debug!(
                "Removing cached file: {cache_file_name} {}",
                format_bytes(info.size as u64)
            );

            fs::remove_file(&cache_file_path)?;
        }

        Ok(())
    }

    async fn scale_image(
        &self,
        asset_id: &Id,
        params: ImageParams,
        baza_manager: &BazaManager,
    ) -> Result<Vec<u8>> {
        log::info!("Scaling image {asset_id} to {params}");

        let buf_reader = {
            let baza = baza_manager.open()?;
            let blob = baza.get_asset_data(asset_id)?;
            BufReader::new(blob)
        };

        let start_time = Instant::now();

        let (send, recv) = tokio::sync::oneshot::channel();
        rayon::spawn_fifo(move || {
            let result = scale_image_file(buf_reader, params.max_w, params.max_h);

            send.send(result)
                .expect("Failed to send back scaled image from the thread pool");
        });

        let duration = start_time.elapsed();
        log::info!("Scaled image {asset_id} to {params} in {:?}", duration);

        recv.await
            .context("Failed to receive result from the thread pool")?
    }

    pub async fn get_image(
        &self,
        asset_id: &Id,
        params: ImageParams,
        baza_manager: &BazaManager,
    ) -> Result<Vec<u8>> {
        let cache_file_name = &format!("{asset_id}.{params}.webp.age");
        let cache_file_path = format!("{}/{cache_file_name}", self.root_dir);

        let data = if file_exists(&cache_file_path)? {
            log::debug!("Found cached image file for {asset_id} {params}");

            let reader = create_file_reader(&cache_file_path)?;
            let decrypted_reader = baza_manager.decrypt(reader)?;

            read_all(decrypted_reader)?
        } else {
            let data = self
                .scale_image(asset_id, params.clone(), baza_manager)
                .await?;

            let writer = create_file_writer(&cache_file_path, false)?;
            let mut encrypted_writer = baza_manager.encrypt(writer)?;

            encrypted_writer
                .write_all(&data)
                .context("Failed to write cache file")?;
            encrypted_writer.finish()?;

            log::debug!("Wrote cached image file to disk for {asset_id} {params}");

            data
        };

        // update cache hit info
        {
            let mut usage_info = self.usage_info.write().await;
            usage_info.insert(
                cache_file_name.clone(),
                ImageInfo {
                    access_time: Timestamp::now(),
                    size: data.len(),
                },
            );
        }

        self.remove_files_if_necessary().await?;

        Ok(data)
    }
}
