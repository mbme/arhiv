use rand::distributions::Alphanumeric;
use rand::prelude::*;
use std::env;
use std::fs;
use std::iter;

pub struct TempFile {
    path: String,
}

impl TempFile {
    pub fn new(prefix: &str) -> Self {
        TempFile {
            path: generate_temp_path(prefix),
        }
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        fs::remove_file(&self.path).expect("failed to remove file");
    }
}

pub fn generate_temp_path(prefix: &str) -> String {
    let mut path = env::temp_dir();

    let name = generate_random_name(5);
    path.push(format!("{}-{}", prefix, name));

    path.to_str()
        .expect("must be able to convert path to string")
        .to_string()
}

fn generate_random_name(length: usize) -> String {
    let mut rng = thread_rng();

    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(length)
        .collect()
}
