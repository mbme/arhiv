use std::fmt;

pub enum DbSettings {
    IsPrime,
    DbRevision,
    SchemaVersion,
}

impl fmt::Display for DbSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            DbSettings::IsPrime => "is_prime",
            DbSettings::DbRevision => "db_revision",
            DbSettings::SchemaVersion => "schema_version",
        };

        write!(f, "{}", value)
    }
}
