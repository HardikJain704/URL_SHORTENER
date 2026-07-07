# URL Shortener Rust

A lightweight URL shortener built with Rust, Axum, SQLx, and PostgreSQL.

## Features

- Create a short URL from a long URL
- Redirect short URLs back to their original destination
- Supports both plain text and JSON request bodies
- Includes Docker support for the app and database

## Project structure

- src/main.rs: app entry point and database setup
- src/routes.rs: HTTP routes for shortening and redirecting
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
2. Set the database URL:

```powershell
$env:DATABASE_URL="postgres://postgres:password@127.0.0.1:5432/postgres"
```

3. Run the app:

```powershell
cargo run
```

The server will start on:

```text
http://127.0.0.1:3008
```

### Option 2: With Docker Compose

Run:

```bash
docker compose up --build
```

This starts:
- PostgreSQL on port 5432
- the Rust app on port 3008

## API usage

### Create a shortened URL

```bash
curl -X POST http://127.0.0.1:3008/ -H "Content-Type: application/json" -d '{"url":"https://example.com"}'
```

Example response:

```text
http://localhost:3008/<short_id>
```

### Redirect to the original URL

```bash
curl -I http://127.0.0.1:3008/<short_id>
```

## Database

The app uses a `links` table with the following columns:
- `id`
- `target_url`

Migrations are stored in the `migrations/` folder.


