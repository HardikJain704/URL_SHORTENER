FROM rust:1.78

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

EXPOSE 3008

ENV DATABASE_URL=postgres://postgres:password@db:5433/postgres

CMD ["cargo", "run", "--release"]
