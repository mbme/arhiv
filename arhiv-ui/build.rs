use std::fs;

// Create dist/bundle.js file if missing, so that include_str!() macro doesn't break the build
fn main() {
    let dir = format!("{}/dist", env!("CARGO_MANIFEST_DIR"));
    fs::create_dir_all(&dir).unwrap();

    let file = format!("{}/bundle.js", dir);
    if !fs::metadata(&file).is_ok() {
        fs::File::create(file).unwrap();
    }
}
