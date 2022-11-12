mod v1;
mod v2;

use baza::schema::DataMigrations;

use self::v1::DataSchema1;
use self::v2::DataSchema2;

pub(crate) fn get_data_migrations() -> DataMigrations {
    vec![
        Box::new(DataSchema1), //
        Box::new(DataSchema2),
    ]
}
