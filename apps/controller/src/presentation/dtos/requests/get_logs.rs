use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::presentation) struct GetLogsRequestDto {
    #[serde(default)]
    pub from_lines: usize,
}
