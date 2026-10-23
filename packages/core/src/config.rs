use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) toolchain: Toolchain,
}

#[derive(Deserialize)]
pub(crate) struct Toolchain {
    pub(crate) language: Language,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Language {
    Ts,
}

impl Language {
    pub(crate) fn dockerfile(&self) -> &'static str {
        match self {
            Self::Ts => "scriptc",
        }
    }
}
