use form_urlencoded::Serializer;

pub mod server;

pub fn get_mime_from_path(path: impl AsRef<str>) -> String {
    mime_guess::from_path(path.as_ref())
        .first_or_octet_stream()
        .to_string()
}

pub fn query_builder<'s>() -> Serializer<'s, String> {
    Serializer::new(String::new())
}
