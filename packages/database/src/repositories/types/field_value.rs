use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldValue {
    String(String),
    Uuid(Uuid),
    Integer(i64),
    Boolean(bool),
    Null,
}

impl From<String> for FieldValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for FieldValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Uuid> for FieldValue {
    fn from(value: Uuid) -> Self {
        Self::Uuid(value)
    }
}

impl From<i64> for FieldValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<i32> for FieldValue {
    fn from(value: i32) -> Self {
        Self::Integer(value.into())
    }
}

impl From<bool> for FieldValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl<T: Into<FieldValue>> From<Option<T>> for FieldValue {
    fn from(value: Option<T>) -> Self {
        value.map(Into::into).unwrap_or(Self::Null)
    }
}
