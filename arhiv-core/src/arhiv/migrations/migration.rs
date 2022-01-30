use anyhow::Result;
use rusqlite::Connection;

use rs_utils::FsTransaction;

pub trait Migration {
    fn get_version(&self) -> u8;

    fn get_schema(&self) -> &str;

    fn apply(&self, conn: &Connection, fs_tx: &mut FsTransaction, data_dir: &str) -> Result<()>;
}
