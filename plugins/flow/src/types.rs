use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FlowProperties {
    pub(crate) source: String,
    #[serde(default = "default_path")]
    pub(crate) path: String,
}

fn default_path() -> String {
    "examples".into()
}
