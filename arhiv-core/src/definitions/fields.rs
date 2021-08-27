use crate::schema::FieldType;

pub fn language_field() -> FieldType {
    FieldType::Enum(vec![
        "Ukrainian",
        //
        "English",
        "French",
        "German",
        "Polish",
        "Russian",
        "Czech",
        "Spanish",
        "Portuguese",
        "Italian",
        "Greek",
        "Latin",
        //
        "Chinese",
        "Hindi",
        "Bengali",
        "Japanese",
        "Korean",
        //
        "Turkish",
        "Arabic",
    ])
}

pub fn rating_field() -> FieldType {
    FieldType::Enum(vec!["Very Bad", "Bad", "Average", "Fine", "Good", "Great"])
}
