use crate::FieldValue;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
