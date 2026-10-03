# =============================================================================
# Stage 1: base — shared Rust + Alpine toolchain
# =============================================================================
FROM rust:1.94-alpine3.23 AS base
# musl-dev: required for static linking on Alpine
# g++ / make: build scripts in some dependencies
# openssl: required for build-time cryptographic key generation
RUN apk add --no-cache musl-dev g++ make openssl
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
CMD sh -c "cargo run --bin migrate && cargo watch -x 'run --bin myplace'"

# =============================================================================
# Stage 3: builder — compiles release binaries
# =============================================================================
FROM base AS builder

# Pre-cache dependencies by building a stub crate first.
COPY Cargo.toml Cargo.lock ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    mkdir -p src src/bin && \
    touch src/lib.rs && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/seed.rs && \
    echo "fn main() {}" > src/bin/migrate.rs && \
    cargo build --release && \
    rm -rf src target/release/myplace* target/release/seed* target/release/migrate* \
           target/release/deps/myplace* target/release/deps/seed* target/release/deps/migrate*

# Copy full source including offline SQLx cache (.sqlx/), build script, and migrations.
COPY . .

# Generate Ed25519 keys during the build step if not present
RUN if [ ! -f private.pem ]; then openssl genpkey -algorithm ed25519 -out private.pem; fi && \
    if [ ! -f public.pem ]; then openssl pkey -in private.pem -pubout -out public.pem; fi

# SQLX_OFFLINE=true allows compilation against .sqlx/ metadata cache without a live database.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    touch src/lib.rs src/main.rs src/bin/*.rs && \
    SQLX_OFFLINE=true cargo build --release && \
    cp target/release/myplace  /usr/local/bin/myplace && \
    cp target/release/seed     /usr/local/bin/seed && \
    cp target/release/migrate  /usr/local/bin/migrate

# =============================================================================
# Stage 4: runtime — minimal production image
# =============================================================================
FROM alpine:3.23 AS runtime
# ca-certificates: required for outbound HTTPS (e.g. S3 / Cloudflare R2 / Render)
# ffmpeg + ffprobe: required for video metadata probing, poster thumbnail generation, and faststart optimization
# libgcc: runtime support for musl-linked binaries
RUN apk add --no-cache ca-certificates ffmpeg libgcc

WORKDIR /app

# Application binaries
COPY --from=builder /usr/local/bin/myplace  /app/myplace
COPY --from=builder /usr/local/bin/seed     /app/seed
COPY --from=builder /usr/local/bin/migrate  /app/migrate

# Convenience shim so `sqlx migrate run` invokes the migrate binary
RUN printf '#!/bin/sh\nexec /app/migrate "$@"\n' > /usr/local/bin/sqlx && \
    chmod +x /usr/local/bin/sqlx

# Static assets and migration definitions
COPY static/ static/
COPY migrations/ migrations/

EXPOSE 8080 8081 8082

# Run migrations → seed admin user → start server.
# Each step is idempotent and safe to run on every deployment/container start.
CMD ["sh", "-c", "./migrate && ./seed && ./myplace"]
