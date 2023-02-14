use std::ops::Deref;

use serde_json::Value;

use rs_utils::generate_temp_path;

use crate::{
    entities::{Document, DocumentClass},
    schema::DataSchema,
    Baza,
};

mod validation;

pub struct TestBaza {
    baza: Baza,
}

impl TestBaza {
    pub fn create(schema: DataSchema) -> Self {
        let temp_dir = generate_temp_path("TempBaza", "");
        let baza = Baza::create(temp_dir, schema, vec![]).expect("must create baza");

        TestBaza { baza }
    }
}

// Remove temporary Baza in tests
impl Drop for TestBaza {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.baza.get_path_manager().root_dir)
            .expect("must be able to remove baza");
    }
}

impl Deref for TestBaza {
    type Target = Baza;

    fn deref(&self) -> &Self::Target {
        &self.baza
    }
}

pub fn new_document(value: Value) -> Document {
    Document::new_with_data(
        DocumentClass::new("test_type", ""),
        value.try_into().unwrap(),
    )
}
