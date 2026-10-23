# giong-database

SQL database for Giong, using SeaORM.
Includes database migrations and repository APIs.

SQLite uses `sqlite://PATH?mode=rwc`. Cloudflare D1 uses
`d1://:API_TOKEN@ACCOUNT_ID/DATABASE_ID` (D1 read/write).
Set `GIONG_DATABASE_URL` in the CLI/controller to select D1; otherwise they use
local SQLite. Both backends run migrations automatically on startup.

D1 reuses the SeaORM query builders and sends parameterized SQL over HTTPS.
`bind_parameter` encodes SQL values and `bind_response` decodes D1's native values:
null, integers, real numbers, text and BLOBs. The shared repository decodes rows
into the expected model, so `i32` and `i64` are not guessed from SQLite's schema.
Models derive `serde::Deserialize`; boolean and JSON columns use their model
column definitions. UUIDs deserialize from text, and BLOBs preserve arbitrary bytes.

UUID, enum, JSON, decimal and chrono/time parameters use their text representations.
Date/time and decimal model fields must deserialize the stored representation.
Non-finite floats and integers outside D1 REST's safe JSON integer range
(`±9,007,199,254,740,991`) are rejected instead of silently losing data.
For larger integers, bind text with `CAST(? AS INTEGER)` and read them as text.
See [D1 type conversion](https://developers.cloudflare.com/d1/worker-api/#type-conversion).

`transaction` opens a transaction on SQLite. On D1 it runs the callback directly;
writes are independent and cannot be rolled back together.
