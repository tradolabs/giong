# Giong CLI

Add repositories and stacks. Run from the project root:

```sh
cargo run -p giong-cli -- add repository --name=my-repo --url=https://github.com/you/repo.git --token=your-token
cargo run -p giong-cli -- add stack --name=my-app --repo-url=https://github.com/you/repo.git --branch=main --folder-path=.
```

Omit `--token` for public repositories. The stack folder must contain `giong.toml`.
State defaults to `~/.giong`; use `--data-dir` to change it.

Tokens are encrypted with AES-256-GCM before storage. Copy `apps/cli/.env.example` to `.env` in
your working directory and set
`GIONG_ENCRYPTION_KEY` using `openssl rand -base64 32`.
The CLI and controller must use the same key. Environment variables take
precedence over `.env`, which is searched in the current directory and its parents.
In Docker, pass `--env-file .env` to both apps. Keep the key to decrypt saved tokens.

The root Dockerfile packages this CLI and the controller in one image. Run CLI
commands with `docker run --rm -v giong-data:/data/.giong giong giong ...`.
Use the same volume as the controller. See the root README for complete examples.
