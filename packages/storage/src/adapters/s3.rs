use crate::StorageAdapter;
use anyhow::{Context, Result, ensure};
use s3::{Bucket, Region, creds::Credentials};
use std::path::Path;
use url::Url;

pub(crate) struct S3 {
    bucket: Box<Bucket>,
}

impl S3 {
    pub(crate) fn new(storage_url: &str) -> Result<Self> {
        let url = Url::parse(storage_url)?;
        let host = url.host_str().context("Missing S3 endpoint host")?;
        let bucket_name = url.path().trim_matches('/');
        let access_key = Some(url.username())
            .filter(|v| !v.is_empty())
            .map(str::to_owned)
            .or_else(|| std::env::var("AWS_ACCESS_KEY_ID").ok())
            .context("Missing S3 access key")?;
        let secret_key = url
            .password()
            .map(str::to_owned)
            .or_else(|| std::env::var("AWS_SECRET_ACCESS_KEY").ok())
            .context("Missing S3 secret key")?;

        ensure!(!bucket_name.is_empty(), "Missing S3 bucket name");

        let region = Region::Custom { region: "auto".into(), endpoint: format!("https://{host}") };
        let credentials = Credentials::new(Some(&access_key), Some(&secret_key), None, None, None)?;
        let bucket = Bucket::new(bucket_name, region, credentials)?.with_path_style();

        Ok(Self { bucket })
    }
}

#[async_trait::async_trait]
impl StorageAdapter for S3 {
    async fn upload(&self, path: &Path, key: &str, content_type: &str) -> Result<()> {
        let mut file = tokio::fs::File::open(path)
            .await
            .context("Could not open file for upload")?;
        let response = self
            .bucket
            .put_object_stream_with_content_type(&mut file, key, content_type)
            .await
            .context("Could not upload file to S3")?;
        ensure!(
            (200..300).contains(&response.status_code()),
            "S3 upload failed with HTTP {}",
            response.status_code()
        );
        Ok(())
    }
}
