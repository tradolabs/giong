use crate::adapters::base::StorageAdapter;
use crate::error::Error;
use s3::{Bucket, Region, creds::Credentials, error::S3Error};
use std::path::Path;
use url::Url;

pub(crate) struct S3 {
    bucket: Box<Bucket>,
}

impl S3 {
    pub(crate) fn new(storage_url: &str) -> Result<Self, Error> {
        let storage_url =
            Url::parse(storage_url).map_err(|_| Error::Configuration("invalid S3 URL".into()))?;

        let host = storage_url
            .host_str()
            .ok_or(Error::Configuration("missing S3 endpoint host".into()))?;
        let bucket_name = Some(storage_url.path().trim_matches('/'))
            .filter(|name| !name.is_empty())
            .ok_or(Error::Configuration("missing S3 bucket name".into()))?;
        let access_key = Some(storage_url.username())
            .filter(|v| !v.is_empty())
            .map(str::to_owned)
            .or_else(|| std::env::var("AWS_ACCESS_KEY_ID").ok())
            .ok_or(Error::Configuration("missing S3 access key".into()))?;
        let secret_key = storage_url
            .password()
            .map(str::to_owned)
            .or_else(|| std::env::var("AWS_SECRET_ACCESS_KEY").ok())
            .ok_or(Error::Configuration("missing S3 secret key".into()))?;

        let region = Region::Custom { region: "auto".into(), endpoint: format!("https://{host}") };
        let credentials = Credentials::new(Some(&access_key), Some(&secret_key), None, None, None)
            .map_err(|_| Error::Configuration("invalid S3 credentials".into()))?;
        let bucket = Bucket::new(bucket_name, region, credentials)
            .map_err(|_| Error::Configuration("invalid S3 bucket configuration".into()))?
            .with_path_style();

        Ok(Self { bucket })
    }
}

#[async_trait::async_trait]
impl StorageAdapter for S3 {
    async fn upload(&self, path: &Path, key: &str, content_type: &str) -> Result<(), Error> {
        let mut file = tokio::fs::File::open(path).await?;

        self.bucket
            .put_object_stream_with_content_type(&mut file, key, content_type)
            .await
            .map_err(|err| Error::BadRequest(err.to_string()))?;

        Ok(())
    }

    async fn download(&self, key: &str) -> Result<Option<Vec<u8>>, Error> {
        match self.bucket.get_object(key).await {
            Ok(response) => Ok(Some(response.to_vec())),
            Err(S3Error::HttpFailWithBody(404, _)) => Ok(None),
            Err(error) => Err(Error::BadRequest(error.to_string())),
        }
    }
}
