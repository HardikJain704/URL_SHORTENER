# URL Shortener Rust

A lightweight URL shortener built with Rust, Axum, SQLx, and PostgreSQL.

## Features

- Create a short URL from a long URL
- Redirect short URLs back to their original destination
- Accept plain text and JSON request bodies
- Protect the shortening endpoint with an API key header
- Log incoming requests and apply a simple in-memory rate limiter
- Run locally or with Docker Compose

## Project structure

- src/main.rs: app entry point and database setup
- src/routes.rs: HTTP routes, middleware, and request handling
- src/db/queries.rs: database query helpers
- migrations/: SQL migrations for the links table
- Dockerfile: container definition for the Rust app
- docker-compose.yaml: starts PostgreSQL and the app together

## Prerequisites

- Rust
- PostgreSQL (or Docker)

## Running locally

### Option 1: Without Docker

1. Start PostgreSQL locally and make sure it is reachable on port 5432.
2. Set the database URL and API key:

```powershell
$env:DATABASE_URL = "postgres://postgres:password@127.0.0.1:5432/postgres"
$env:API_KEY = "dev-secret"
```

3. Run the app:

```powershell
cargo run
```

The server will start on:

```text
http://127.0.0.1:3008
```

If API_KEY is not set, the app falls back to `dev-secret`.

### Option 2: With Docker Compose

Run:

```bash
docker compose up --build
```

This starts:
- PostgreSQL on port 5433
- the Rust app on port 3008

## API usage

### Create a shortened URL

```powershell
curl.exe -i -X POST "http://127.0.0.1:3008/" `
  -H "Content-Type: application/json" `
  -H "x-api-key: dev-secret" `
  -d '{"url":"https://www.youtube.com/"}'
```

Example response:

```text
http://localhost:3008/<short_id>
```

### Redirect to the original URL

```powershell
curl.exe -I "http://127.0.0.1:3008/<short_id>"
```

This returns an HTTP `303 See Other` redirect to the target URL.

## Notes

- The POST endpoint `/` requires the `x-api-key` header.
- Requests are logged and rate-limited with a simple in-memory limiter, suitable for local development and demos.
- For production, consider using a reverse proxy or a shared rate limiter.

## Database

The app uses a `links` table with the following columns:
- `id`
- `target_url`

Migrations are stored in the `migrations/` folder.


