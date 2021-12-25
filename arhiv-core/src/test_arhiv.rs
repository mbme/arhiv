use std::{ops::Deref, sync::Arc};

use rs_utils::generate_temp_path;

use crate::{
    definitions::get_standard_schema,
    schema::{Collection, DataDescription, Field, FieldType},
    Arhiv, Config,
};

pub struct TestArhiv(pub Arc<Arhiv>);

impl TestArhiv {
    fn new(config: Config, prime: bool, mut modules: Vec<DataDescription>) -> Self {
        let mut schema = get_standard_schema();

        schema.with_modules(&mut modules);

        let arhiv = Arhiv::create(config, schema, "test-arhiv".to_string(), prime)
            .expect("must be able to create temp arhiv");

        TestArhiv(Arc::new(arhiv))
    }

    #[must_use]
    pub fn new_prime_with_schema(modules: Vec<DataDescription>) -> Self {
        let config = Config {
            arhiv_root: generate_temp_path("TempArhiv", ""),
            ..Config::default()
        };

        TestArhiv::new(config, true, modules)
    }

    #[must_use]
    pub fn new_prime() -> Self {
        TestArhiv::new_prime_with_schema(vec![DataDescription {
            document_type: "test_type",
            collection_of: Collection::None,
            fields: vec![
                Field {
                    name: "blob",
                    field_type: FieldType::BLOBId,
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "test",
                    field_type: FieldType::String {},
                    mandatory: false,
                    readonly: false,
                },
            ],
        }])
    }

    #[must_use]
    pub fn new_replica(port: u16) -> Self {
        let config = Config {
            arhiv_root: generate_temp_path("TempArhiv", ""),
            prime_url: format!("http://localhost:{}", port),
            ..Config::default()
        };

        TestArhiv::new(
            config,
            false,
            vec![DataDescription {
                document_type: "test_type",
                collection_of: Collection::None,
                fields: vec![
                    Field {
                        name: "blob",
                        field_type: FieldType::BLOBId,
                        mandatory: false,
                        readonly: false,
                    },
                    Field {
                        name: "test",
                        field_type: FieldType::String {},
                        mandatory: false,
                        readonly: false,
                    },
                ],
            }],
        )
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
