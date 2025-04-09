use std::{collections::HashMap, io::Write};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use rs_utils::{age::AgeKey, create_file_reader, create_file_writer, AgeGzReader, AgeGzWriter};

use crate::{
    baza2::BazaInfo,
    entities::{DocumentKey, DocumentLock, Id, InstanceId, Refs},
};

use super::DocumentHead;

pub type Locks = HashMap<Id, DocumentLock>;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BazaStateFile {
    pub instance_id: InstanceId,
    pub info: BazaInfo,
    pub documents: HashMap<Id, DocumentHead>,
    pub locks: Locks,
    pub refs: HashMap<DocumentKey, Refs>,
}

impl BazaStateFile {
    pub fn read(file: &str, key: AgeKey) -> Result<Self> {
        let reader = create_file_reader(file)?;
        let agegz_reader = AgeGzReader::new(reader, key)?;

        let file: BazaStateFile =
            serde_json::from_reader(agegz_reader).context("Failed to parse BazaStateFile")?;

        Ok(file)
    }

    pub fn write(&self, file: &str, key: AgeKey) -> Result<()> {
        let writer = create_file_writer(file, true)?;

        let mut agegz_writer = AgeGzWriter::new(writer, key)?;

        serde_json::to_writer(&mut agegz_writer, &self)
            .context("Failed to serialize BazaStateFile")?;

        let mut writer = agegz_writer.finish()?;
        writer.flush()?;

        Ok(())
    }
}
