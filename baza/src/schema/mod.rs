use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::entities::{DocumentType, ERASED_DOCUMENT_TYPE};

pub use attachment::*;
pub use data_description::*;
pub use data_migration::*;
pub use field::*;

mod attachment;
mod data_description;
mod data_migration;
mod field;

const ERASED_DOCUMENT_DATA_DESCRIPTION: &DataDescription = &DataDescription {
    document_type: ERASED_DOCUMENT_TYPE,
    title_format: "Erased document",
    fields: vec![],
};

#[derive(Serialize, Clone)]
pub struct DataSchema {
    name: String,
    modules: Vec<DataDescription>,
    #[serde(skip)]
    pub(crate) migrations: Arc<DataMigrations>,
}

impl DataSchema {
    #[must_use]
    pub fn new(name: impl Into<String>, modules: Vec<DataDescription>) -> Self {
        Self::with_migrations(name, modules, vec![])
    }

    #[cfg(test)]
    pub fn new_test_schema() -> Self {
        Self::new(
            "test",
            vec![
                DataDescription {
                    document_type: "test_type",
                    title_format: "{test}",
                    fields: vec![
                        Field {
                            name: "test",
                            field_type: FieldType::String {},
                            mandatory: false,
                            readonly: false,
                        },
                        Field {
                            name: "blob",
                            field_type: FieldType::BLOBId {},
                            mandatory: false,
                            readonly: false,
                        },
                        Field {
                            name: "ref",
                            field_type: FieldType::Ref(&["test_type"]),
                            mandatory: false,
                            readonly: false,
                        },
                    ],
                },
                get_attachment_definition(),
            ],
        )
    }

    pub fn with_migrations(
        name: impl Into<String>,
        mut modules: Vec<DataDescription>,
        migrations: DataMigrations,
    ) -> Self {
        modules.push(ERASED_DOCUMENT_DATA_DESCRIPTION.clone());

        DataSchema {
            name: name.into(),
            modules,
            migrations: Arc::new(migrations),
        }
    }

    #[must_use]
    pub fn get_app_name(&self) -> &str {
        &self.name
    }

    pub fn get_data_description(&self, document_type: &DocumentType) -> Result<&DataDescription> {
        self.modules
            .iter()
            .find(|module| document_type.is(module.document_type))
            .ok_or_else(|| {
                let types = self.get_document_types().join(", ");

                anyhow!(
                    "Unknown document type {}, must be one of [{}]",
                    document_type,
                    types
                )
            })
    }

    pub fn iter_fields(
        &self,
        document_type: &DocumentType,
    ) -> Result<impl Iterator<Item = &Field>> {
        let data_description = self.get_data_description(document_type)?;

        Ok(data_description.fields.iter())
    }

    #[must_use]
    pub fn get_document_types(&self) -> Vec<&'static str> {
        self.modules
            .iter()
            .map(|module| module.document_type)
            .collect()
    }

    #[must_use]
    pub fn get_latest_data_version(&self) -> u8 {
        self.migrations.iter().fold(0, |latest_version, migration| {
            migration.get_version().max(latest_version)
        })
    }

    #[must_use]
    pub fn get_min_data_migration_version(&self) -> u8 {
        self.migrations
            .iter()
            .fold(u8::MAX, |latest_version, migration| {
                migration.get_version().min(latest_version)
            })
    }
}
