# syntax=docker/dockerfile:1.7
#
# Multi-stage build for stellaux_server.
# All deps are pure-Rust (sea-orm/sqlx-postgres + rustls; reqwest + rustls-tls;
# jsonwebtoken via ring; argon2 RustCrypto) — no libpq, no libssl needed.

ARG RUST_VERSION=1.94

# ─── Stage 1: chef base ──────────────────────────────────────────────────────
FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef
WORKDIR /app

# ─── Stage 2: planner ────────────────────────────────────────────────────────
# Compute the dependency recipe. Re-runs only when Cargo.toml / Cargo.lock change.
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# ─── Stage 3: builder ────────────────────────────────────────────────────────
FROM chef AS builder

# No system deps needed — pure-Rust crypto/db/http stack.
ENV SQLX_OFFLINE=true

# Cache the dependency build. Reused whenever Cargo.toml is unchanged.
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy the full crate after dependency cooking so future additions such as
# `.sqlx/` metadata or embedded migrations are available to the release build
# without busting the cached dependency layer on every code change.
COPY . .
RUN cargo build --release --bin stellaux_server

# ─── Stage 4: runtime ────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# ca-certificates for outbound TLS (Stripe/Shippo/Resend). curl for HEALTHCHECK.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 1001 app \
    && useradd  --system --uid 1001 --gid app --no-create-home --shell /usr/sbin/nologin app

WORKDIR /app

COPY --from=builder /app/target/release/stellaux_server /usr/local/bin/stellaux_server

USER app
EXPOSE 8080

ENV RUST_LOG=info,stellaux_server=info \
    SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080

# Expects GET /healthz returning 200. Wire one up in src/server.rs.
HEALTHCHECK --interval=30s --timeout=3s --start-period=20s --retries=3 \
    CMD curl --fail --silent http://localhost:8080/healthz || exit 1

ENTRYPOINT ["/usr/local/bin/stellaux_server"]
