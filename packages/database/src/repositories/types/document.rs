use std::collections::BTreeMap;

use super::field_value::FieldValue;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Document {
    pub(crate) fields: BTreeMap<String, FieldValue>,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(mut self, field: impl Into<String>, value: impl Into<FieldValue>) -> Self {
        self.fields.insert(field.into(), value.into());
        self
    }
}
