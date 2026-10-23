# Giong Controller

Poll Git repositories and run pipelines for changed commits. Run in `apps/controller`:

```sh
cargo run
```

HTTP listens on `0.0.0.0:3000`. Press Ctrl-C to stop.
Set `GIONG_SCHEDULE_ENABLED=true` to poll Git every 30 seconds and queue new jobs.
Scheduling is disabled when this variable is unset or has another value.
Queued jobs run concurrently as they arrive. Pending jobs resume on startup.
State is stored in `~/.giong`.
Use the same state directory as the CLI.

Repository tokens are encrypted in SQLite and decrypted for Git authentication.
Set the same `GIONG_ENCRYPTION_KEY` as the CLI through the environment or `.env`.
Copy this app's `.env.example` to `.env` in your working directory.
In Docker, pass `--env-file .env`.

Set `GIONG_GITHUB_WEBHOOK_SECRET` in `.env` to enable GitHub webhooks.
Generate a secret with `openssl rand -hex 32`.

- `GET /`: the legend of Thánh Gióng in Vietnamese and English.
- `POST /webhooks/github`: accepts signed GitHub `push` and `ping` events.

In the GitHub repository's webhook settings, use
`https://your-host/webhooks/github`, content type `application/json`, the same
secret, and subscribe to push events. Put an HTTPS proxy in front of the HTTP server.
Webhook signatures use `X-Hub-Signature-256`; unsigned or invalid requests return `401`.
Without a configured secret, the webhook endpoint returns `503`.

Pushes return `202` once accepted into the trigger queue. The controller checks
the current Git tip for each configured stack branch in the matching repository.
Use the same HTTPS clone URL registered with the CLI. Tag pushes, branch
deletions, and other events are ignored. Repeated deliveries and polling do not
queue another job for an unchanged tip. Trigger queue capacity is 100; a full
queue returns `503`. Pending triggers are in memory; polling catches changes
after a restart.

Requires Docker. The root Dockerfile builds a shared controller/CLI image with
the Docker CLI included. Mount `/var/run/docker.sock` so the controller can start
job containers on the host, and persist `/data/.giong` for state.
Build from the repository root and publish the HTTP port:

```sh
docker build -t giong .
docker run --name giong-controller --env-file apps/controller/.env \
  -p 3000:3000 -v giong-data:/data/.giong \
  -v /var/run/docker.sock:/var/run/docker.sock giong
```
