use std::{
    collections::HashMap,
    io::{BufRead, Write},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use rs_utils::{age::AgeKey, AgeGzReader, AgeGzWriter};

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
    pub fn read(reader: impl BufRead, key: AgeKey) -> Result<Self> {
        let agegz_reader = AgeGzReader::new(reader, key)?;

        let file: BazaStateFile =
            serde_json::from_reader(agegz_reader).context("Failed to parse BazaStateFile")?;

        Ok(file)
    }

    pub fn write(&self, writer: impl Write, key: AgeKey) -> Result<()> {
        let mut agegz_writer = AgeGzWriter::new(writer, key)?;

        serde_json::to_writer(&mut agegz_writer, &self)
            .context("Failed to serialize BazaStateFile")?;

        agegz_writer.finish()?;

        Ok(())
    }
}
