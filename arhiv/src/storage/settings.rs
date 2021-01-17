use std::fmt;

pub enum DbSettings {
    ArhivId,
    IsPrime,
    SchemaVersion,
    DbRevision,
}

impl fmt::Display for DbSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            DbSettings::ArhivId => "arhiv_id",
            DbSettings::IsPrime => "is_prime",
            DbSettings::DbRevision => "db_revision",
            DbSettings::SchemaVersion => "schema_version",
        };

        write!(f, "{}", value)
    }
}
