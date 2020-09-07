use rand::distributions::Alphanumeric;
use rand::prelude::*;
use std::env;
use std::fs;
use std::iter;

pub struct TempFile {
    path: String,
}

impl TempFile {
    pub fn new(file_name: &str) -> Self {
        TempFile {
            path: file_in_temp_dir(file_name),
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

pub fn file_in_temp_dir(file_name: &str) -> String {
    let mut path = env::temp_dir();

    path.push(file_name);

    path.to_str()
        .expect("must be able to convert path to string")
        .to_string()
}

pub fn generate_temp_path(prefix: &str, suffix: &str) -> String {
    let name = generate_random_name(5);

    file_in_temp_dir(&format!("{}{}{}", prefix, name, suffix))
}

fn generate_random_name(length: usize) -> String {
    let mut rng = thread_rng();

    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(length)
        .collect()
}
