use std::{collections::HashMap, io::Write, time::Instant};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use rs_utils::{
    AgeGzReader, AgeGzWriter, age::AgeKey, create_file_reader, create_file_writer, log,
};

use crate::{
    BazaInfo,
    entities::{DocumentKey, Id, InstanceId, Refs},
};

use super::DocumentHead;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BazaStateFile {
    pub instance_id: InstanceId,
    pub info: BazaInfo,
    pub documents: HashMap<Id, DocumentHead>,
    pub refs: HashMap<DocumentKey, Refs>,

    #[serde(skip)]
    pub modified: bool,
}

impl BazaStateFile {
    pub fn new(instance_id: InstanceId, info: BazaInfo) -> Self {
        BazaStateFile {
            info,
            documents: HashMap::new(),
            refs: HashMap::new(),
            instance_id,
            modified: true,
        }
    }

    pub fn read(file: &str, key: AgeKey) -> Result<Self> {
        log::debug!("Reading state from file {file}");

        let start_time = Instant::now();

        let reader = create_file_reader(file)?;
        let agegz_reader = AgeGzReader::new(reader, key)?;

        let file: BazaStateFile =
            serde_json::from_reader(agegz_reader).context("Failed to parse BazaStateFile")?;

        let duration = start_time.elapsed();
        log::info!("Read state from file in {:?}", duration);

        Ok(file)
    }

    pub fn write(&self, file: &str, key: AgeKey) -> Result<()> {
        log::debug!("Writing state to file {file}");

        let start_time = Instant::now();

        let writer = create_file_writer(file, true)?;

        let mut agegz_writer = AgeGzWriter::new(writer, key)?;

        serde_json::to_writer(&mut agegz_writer, &self)
            .context("Failed to serialize BazaStateFile")?;

        let mut writer = agegz_writer.finish()?;
        writer.flush()?;

        let duration = start_time.elapsed();
        log::info!("Wrote state to file in {:?}", duration);

        Ok(())
    }
}
