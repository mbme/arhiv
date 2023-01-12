mod v1;
mod v2;
mod v3;

use baza::schema::DataMigrations;

use self::v1::DataSchema1;
use self::v2::DataSchema2;
use self::v3::DataSchema3;

pub(crate) fn get_data_migrations() -> DataMigrations {
    vec![
        Box::new(DataSchema1), //
        Box::new(DataSchema2),
        Box::new(DataSchema3),
    ]
}
