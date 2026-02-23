# Insecure Temporary Files in Rust 🦀

This repo demonstrates an **insecure temporary file** pattern (predictable path under `/tmp`) and a safer alternative using Rust’s `tempfile` crate.

## What the app exposes

- `POST /vulnerable_path`
  - Writes request data to a predictable path: `/tmp/sensitive_data.csv`
  - Leaves a short window where another local user/process could read it
- `POST /secure_path`
  - Uses `tempfile::NamedTempFile` (randomized name, restrictive permissions)
  - File is removed automatically when it goes out of scope

## Run

```bash
cargo run
```

Server listens on `http://127.0.0.1:8080`.

## Demo

### Observe the vulnerable behavior

Terminal 1 (watch the predictable file):

```bash
watch -n 0.1 'cat /tmp/sensitive_data.csv 2>/dev/null'
```

Terminal 2 (send data):

```bash
curl -X POST -H "Content-Type: application/json" \
  -d '{"content":"user,password123,secret_token"}' \
  http://127.0.0.1:8080/vulnerable_path
```

You should briefly see the content appear in Terminal 1.

### Verify the secure behavior

```bash
curl -X POST -H "Content-Type: application/json" \
  -d '{"content":"user,password123,secret_token"}' \
  http://127.0.0.1:8080/secure_path
```

The request succeeds, but there’s no predictable path to watch.

## Mitigation takeaways

- **Avoid predictable temp paths** (and hardcoded names).
- **Prefer `tempfile`** (random names + restrictive permissions by default).
- **Keep temp files short-lived** and clean them up reliably.
