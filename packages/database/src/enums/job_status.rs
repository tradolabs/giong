use crate::repositories::types::field_value::FieldValue;
use sea_orm::entity::prelude::{DeriveActiveEnum, EnumIter, StringLen};

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)", rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Interrupted,
}

impl JobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Interrupted => "interrupted",
        }
    }
}

impl From<JobStatus> for FieldValue {
    fn from(value: JobStatus) -> Self {
        value.as_str().into()
    }
}
