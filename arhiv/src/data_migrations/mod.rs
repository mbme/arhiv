mod v3;
mod v4;
mod v5;
mod v6;

use baza::schema::DataMigrations;

use self::v3::DataSchema3;
use self::v4::DataSchema4;
use self::v5::DataSchema5;
use self::v6::DataSchema6;

pub(crate) fn get_data_migrations() -> DataMigrations {
    vec![
        Box::new(DataSchema3), //
        Box::new(DataSchema4),
        Box::new(DataSchema5),
        Box::new(DataSchema6),
    ]
}
