use crate::error::Error;

impl From<sea_orm::DbErr> for Error {
    fn from(error: sea_orm::DbErr) -> Self {
        Self::Orm(error.to_string())
    }
}
