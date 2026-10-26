#[derive(Clone)]
pub struct Config {
    pub database_url: Option<String>,
    pub storage_url: Option<String>,
    pub jwt_secret: Option<String>,
    pub encryption_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let value = |name| std::env::var(name).ok().filter(|value| !value.is_empty());
        Self {
            database_url: value("GIONG_DATABASE_URL"),
            storage_url: value("GIONG_STORAGE_URL"),
            jwt_secret: value("GIONG_JWT_SECRET"),
            encryption_key: value("GIONG_ENCRYPTION_KEY"),
        }
    }
}
