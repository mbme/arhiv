use std::path::Path;

use rocket::{http::ContentType, response::Content};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/public"]
struct PublicAssets;

#[get("/public/<asset>")]
pub fn public_assets(asset: String) -> Option<Content<Vec<u8>>> {
    let data: Vec<u8> = PublicAssets::get(&asset)?.into();

    let content_type = Path::new(&asset)
        .extension()
        .map(|ext| ContentType::from_extension(ext.to_str()?))
        .flatten()
        .unwrap_or(ContentType::Binary);

    Some(Content(content_type, data))
}
