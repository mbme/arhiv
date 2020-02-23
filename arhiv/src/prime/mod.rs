pub struct ArhivPrime {
    root_dir: String,
}

impl ArhivPrime {
    pub fn create(root_dir: &str) -> ArhivPrime {
        ArhivPrime {
            root_dir: root_dir.to_string(),
        }
    }
}
