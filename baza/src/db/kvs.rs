use std::{fmt::Display, marker::PhantomData};

use anyhow::{anyhow, ensure, Context, Result};
use rusqlite::OptionalExtension;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use super::BazaConnection;

impl BazaConnection {
    pub fn kvs_get<T: Serialize + DeserializeOwned>(&self, key: &KvsKey) -> Result<Option<T>> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT value FROM kvs WHERE key = ?1")?;

        let key = key.to_string();

        let value: Option<String> = stmt
            .query_row([&key], |row| row.get(0))
            .optional()
            .context(anyhow!("failed to read kvs {key}"))?;

        if let Some(value) = value {
            serde_json::from_str(&value).context(anyhow!("failed to parse kvs value for {key}"))
        } else {
            Ok(None)
        }
    }

    pub fn kvs_set<T: Serialize + DeserializeOwned>(&self, key: &KvsKey, value: &T) -> Result<()> {
        let key = key.to_string();

        let value =
            serde_json::to_string(value).context(anyhow!("failed to serialize kvs {key}"))?;

        self.get_connection()
            .execute(
                "INSERT OR REPLACE INTO kvs(key, value) VALUES (?, ?)",
                [&key, &value],
            )
            .context(anyhow!("failed to save kvs {key}"))?;

        Ok(())
    }

    pub fn kvs_delete(&self, key: &KvsKey) -> Result<bool> {
        let key = key.to_string();

        let rows_count = self
            .get_connection()
            .execute("DELETE FROM kvs WHERE key = ?1", [&key])
            .context(anyhow!("failed to save kvs {key}"))?;

        ensure!(rows_count <= 1, "deleted {rows_count} rows from kvs {key}");

        Ok(rows_count == 1)
    }

    pub fn kvs_list(&self, namespace: Option<&str>) -> Result<Vec<KvsEntry>> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT key, value FROM kvs")?;

        let rows = stmt
            .query_and_then([], |row| -> Result<KvsEntry> {
                let key_str = row.get::<_, String>(0).expect("must read 0 index");
                let value = row.get::<_, String>(1).expect("must read 0 index");

                let key = KvsKey::parse(&key_str)?;
                Ok(KvsEntry(
                    key,
                    serde_json::from_str(&value)
                        .context(anyhow!("failed to parse kvs value for {key_str}"))?,
                ))
            })
            .context("failed to list kvs entries")?;

        let entries = rows.collect::<Result<Vec<_>>>()?;

        // TODO optimize
        let entries = entries
            .into_iter()
            .filter(|KvsEntry(key, _value)| {
                if let Some(namespace) = namespace {
                    key.namespace == namespace
                } else {
                    true
                }
            })
            .collect();

        Ok(entries)
    }

    pub fn kvs_const_get<T: Serialize + DeserializeOwned>(
        &self,
        key: &KvsConstKey<T>,
    ) -> Result<Option<T>> {
        self.kvs_get(&key.into())
    }

    pub fn kvs_const_must_get<T: Serialize + DeserializeOwned>(
        &self,
        key: &KvsConstKey<T>,
    ) -> Result<T> {
        self.kvs_const_get(key)
            .transpose()
            .context(anyhow!("kvs {key} is missing"))?
    }

    pub fn kvs_const_set<T: Serialize + DeserializeOwned>(
        &self,
        key: &KvsConstKey<T>,
        value: &T,
    ) -> Result<()> {
        self.kvs_set(&key.into(), value)
    }
}

#[derive(Debug)]
pub struct KvsKey {
    pub namespace: String,
    pub key: String,
}

impl ToString for KvsKey {
    fn to_string(&self) -> String {
        serde_json::to_string(&vec![&self.namespace, &self.key])
            .expect("failed to serialize kvs key")
    }
}

impl KvsKey {
    pub fn new(namespace: impl Into<String>, key: impl Into<String>) -> Self {
        KvsKey {
            namespace: namespace.into(),
            key: key.into(),
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        let (namespace, key): (String, String) =
            serde_json::from_str(value).context("failed to parse kvs key")?;

        Ok(KvsKey { namespace, key })
    }
}

pub struct KvsConstKey<T: Serialize + DeserializeOwned> {
    pub namespace: &'static str,
    pub key: &'static str,
    _return_value_type_holder: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> Display for KvsConstKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}/{}]", self.namespace, self.key)
    }
}

impl<T: Serialize + DeserializeOwned> From<&KvsConstKey<T>> for KvsKey {
    fn from(value: &KvsConstKey<T>) -> Self {
        KvsKey {
            namespace: value.namespace.to_string(),
            key: value.key.to_string(),
        }
    }
}

impl<T: Serialize + DeserializeOwned> KvsConstKey<T> {
    pub const fn new(namespace: &'static str, key: &'static str) -> Self {
        KvsConstKey {
            namespace,
            key,
            _return_value_type_holder: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct KvsEntry(pub KvsKey, pub Value);
