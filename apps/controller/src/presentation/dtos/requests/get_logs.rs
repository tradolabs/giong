use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub(in crate::presentation) struct GetLogsRequestDto {
    #[serde(default)]
    pub from_lines: usize,
}
