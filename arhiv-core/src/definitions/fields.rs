use crate::schema::FieldType;

pub const LANGUAGES: &[&str] = &[
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
];

pub const LANGUAGE_FIELD: FieldType = FieldType::Enum(LANGUAGES);

pub const RATINGS: &[&str] = &[
    "Very Bad", //
    "Bad",      //
    "Average",  //
    "Fine",     //
    "Good",     //
    "Great",
];

pub const RATING_FIELD: FieldType = FieldType::Enum(RATINGS);
