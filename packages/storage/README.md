# giong-storage

Cloud object storage abstraction for Giong.

Use `Storage::new(storage_url)` with `STORAGE_URL`, for example:
- `s3://ACCESS_KEY:SECRET_KEY@HOST/BUCKET`

`upload(path, key, content_type)` streams the file; callers choose the object key.
