# =============================================================================
# Stage 1: base — shared Rust + Alpine toolchain
# =============================================================================
FROM rust:1.94-alpine3.23 AS base
# musl-dev: required for static linking on Alpine
# postgresql-dev: libpq headers for sqlx
# g++ / make: build scripts in some dependencies
RUN apk add --no-cache musl-dev postgresql-dev g++ make
WORKDIR /app

# =============================================================================
# Stage 2: development — mounts source, uses cargo-watch for hot reload
# =============================================================================
FROM base AS development

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo install cargo-watch && \
    cargo install sqlx-cli --no-default-features --features sqlx-cli/rustls,sqlx-cli/postgres

# Entrypoint: run migrations, then watch for changes.
# `sqlx migrate run` is idempotent — safe to run on every restart.
CMD sh -c "sqlx migrate run && cargo watch -x 'run --bin myplace'"

# =============================================================================
# Stage 3: builder — compiles release binaries
# =============================================================================
FROM base AS builder

# Install sqlx-cli for offline preparation (used during image build if needed)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install sqlx-cli --no-default-features --features sqlx-cli/rustls,sqlx-cli/postgres

# Pre-cache dependencies by building a stub binary first.
COPY Cargo.toml Cargo.lock ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    mkdir src src/bin && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/seed.rs && \
    cargo build --release && \
    rm -rf src

# Copy full source and build for real.
COPY . .
# SQLX_OFFLINE=true means the macro reads the .sqlx/ cache instead of a live DB.
# Run `cargo sqlx prepare` locally after schema changes to update the cache.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    SQLX_OFFLINE=true cargo build --release && \
    cp target/release/myplace  /usr/local/bin/myplace && \
    cp target/release/seed     /usr/local/bin/seed

# =============================================================================
# Stage 4: runtime — minimal image with just the binaries + migrations
# =============================================================================
FROM alpine:3.23 AS runtime
RUN apk add --no-cache libpq libgcc
WORKDIR /app

COPY --from=builder /usr/local/cargo/bin/sqlx  /usr/local/bin/sqlx
COPY --from=builder /usr/local/bin/myplace     .
COPY --from=builder /usr/local/bin/seed        .
# Migrations need to be available at runtime so sqlx migrate run works.
COPY migrations/ migrations/

EXPOSE 8080

# Run migrations → seed admin → start server.
# Each step is idempotent and safe to run on every deployment.
CMD sh -c "sqlx migrate run && ./seed && ./myplace"
