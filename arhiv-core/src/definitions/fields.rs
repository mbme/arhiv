use crate::schema::FieldType;

pub fn language_field() -> FieldType {
    FieldType::Enum(vec!["Ukrainian", "English", "Russian"])
}

pub fn rating_field() -> FieldType {
    FieldType::Enum(vec!["Very Bad", "Bad", "Average", "Fine", "Good", "Great"])
}
