use super::FieldValue;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operator {
    Eq(FieldValue),
    In(Vec<FieldValue>),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Condition {
    pub(crate) filters: Vec<(String, Operator)>,
}

impl Condition {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn eq(mut self, field: impl Into<String>, value: impl Into<FieldValue>) -> Self {
        self.filters
            .push((field.into(), Operator::Eq(value.into())));
        self
    }

    pub fn is_in<V: Into<FieldValue>>(
        mut self,
        field: impl Into<String>,
        values: impl IntoIterator<Item = V>,
    ) -> Self {
        self.filters.push((
            field.into(),
            Operator::In(values.into_iter().map(Into::into).collect()),
        ));
        self
    }
}
