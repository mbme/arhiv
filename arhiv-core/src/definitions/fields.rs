use crate::schema::FieldType;

pub fn language_field() -> FieldType {
    FieldType::Enum(vec![
        "Ukrainian",
        "English",
        "French",
        "German",
        "Polish",
        "Spanish",
        "Portuguese",
        "Russian",
        "Chinese",
        "Hindi",
        "Bengali",
        "Japanese",
        "Korean",
        "Turkish",
        "Arabic",
    ])
}

pub fn rating_field() -> FieldType {
    FieldType::Enum(vec!["Very Bad", "Bad", "Average", "Fine", "Good", "Great"])
}
