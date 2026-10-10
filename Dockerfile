# =============================================================================
# Stage 1: base — shared Rust + Alpine toolchain
# =============================================================================
FROM rust:1.94-alpine3.23 AS base
# musl-dev: required for static linking on Alpine
# g++ / make: build scripts in some dependencies
# openssl: required for build-time cryptographic key generation
# ffmpeg: required for video metadata probing and processing in dev & test
RUN apk add --no-cache musl-dev g++ make openssl ffmpeg
WORKDIR /app

# =============================================================================
# Stage 2: development — mounts source, uses cargo-watch for hot reload
# =============================================================================
FROM base AS development

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo install cargo-watch

# Entrypoint: run migrations, then watch for changes.
CMD sh -c "cargo run --bin migrate && cargo watch -x 'run --bin myplace'"

# =============================================================================
# Stage 3: chef — install cargo-chef for reliable Docker layer dependency caching
# =============================================================================
FROM base AS chef
RUN cargo install cargo-chef --locked

# =============================================================================
# Stage 4: planner — generates recipe.json lockfile skeleton
# =============================================================================
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# Stage 5: cacher — compiles only third-party dependencies into a Docker layer
# =============================================================================
# This layer is committed to the Docker image and cached in Render's registry.
# As long as Cargo.toml and Cargo.lock do not change, this step is 100% cached.
FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# =============================================================================
# Stage 6: builder — compiles application release binaries
# =============================================================================
FROM chef AS builder

# Restore cached dependencies from cacher layer
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

# Copy full source including offline SQLx cache (.sqlx/), build script, and migrations
COPY . .

# SQLX_OFFLINE=true allows compilation against .sqlx/ metadata cache without a live database.
ENV SQLX_OFFLINE=true
RUN cargo build --release && \
    cp target/release/myplace  /usr/local/bin/myplace && \
    cp target/release/seed     /usr/local/bin/seed && \
    cp target/release/migrate  /usr/local/bin/migrate

# =============================================================================
# Stage 7: runtime — minimal production image
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

