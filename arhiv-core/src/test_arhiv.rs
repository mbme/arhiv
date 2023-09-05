use std::{ops::Deref, sync::Arc};

use baza::schema::{DataDescription, DataSchema, Field, FieldType};
use rs_utils::generate_temp_path;

use crate::{Arhiv, Config};

pub struct TestArhiv(pub Arc<Arhiv>);

impl TestArhiv {
    fn new(config: Config, schema: DataSchema) -> Self {
        let arhiv = Arhiv::create_with_options(config, schema, vec![])
            .expect("must be able to create temp arhiv");

        TestArhiv(Arc::new(arhiv))
    }

    #[must_use]
    pub fn new_prime_with_schema(schema: DataSchema) -> Self {
        let config = Config {
            arhiv_root: generate_temp_path("TempArhiv", ""),
            ..Config::default()
        };

        TestArhiv::new(config, schema)
    }

    #[must_use]
    pub fn new_prime() -> Self {
        TestArhiv::new_prime_with_schema(DataSchema::new(
            "test",
            vec![DataDescription {
                document_type: "test_type",
                fields: vec![
                    Field {
                        name: "blob",
                        field_type: FieldType::BLOBId {},
                        mandatory: false,
                        readonly: false,
                        for_subtypes: None,
                    },
                    Field {
                        name: "test",
                        field_type: FieldType::String {},
                        mandatory: false,
                        readonly: false,
                        for_subtypes: None,
                    },
                ],
                subtypes: None,
            }],
        ))
    }
}

// Remove temporary Arhiv in tests
impl Drop for TestArhiv {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0.get_config().arhiv_root)
            .expect("must be able to remove arhiv");
    }
}

impl Deref for TestArhiv {
    type Target = Arhiv;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
