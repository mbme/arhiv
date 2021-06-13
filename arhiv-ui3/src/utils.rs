use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

// see https://doc.rust-lang.org/std/hash/index.html#examples
pub fn get_file_hash(name: impl Hash, data: impl Hash) -> u64 {
    let mut s = DefaultHasher::new();

    name.hash(&mut s);
    data.hash(&mut s);

    s.finish()
}
