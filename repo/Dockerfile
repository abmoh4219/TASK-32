# ScholarVault — multi-stage build (frontend WASM via trunk + backend Axum binary)
# syntax=docker/dockerfile:1

# ============================================================
# Stage 1: WASM frontend builder
# ============================================================
FROM rust:slim-bookworm AS wasm-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Harden crates.io downloads against transient stalls: retry aggressively and
# disable cargo's "low speed" abort that otherwise kills the build whenever
# the connection briefly drops below ~10 B/s.
ENV CARGO_NET_RETRY=10
ENV CARGO_HTTP_TIMEOUT=300
ENV CARGO_HTTP_LOW_SPEED_LIMIT=0
ENV CARGO_HTTP_MULTIPLEXING=false
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN rustup target add wasm32-unknown-unknown

RUN curl -fsSL \
    "https://github.com/thedodd/trunk/releases/download/v0.20.3/trunk-x86_64-unknown-linux-gnu.tar.gz" \
    | tar -xzC /usr/local/bin
RUN curl -fsSL \
    "https://github.com/rustwasm/wasm-bindgen/releases/download/0.2.118/wasm-bindgen-0.2.118-x86_64-unknown-linux-musl.tar.gz" \
    | tar -xzC /usr/local/bin --strip-components=1 || true

WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY shared ./shared
COPY backend ./backend
COPY frontend ./frontend
COPY index.html ./
COPY style ./style
COPY tailwind.config.js ./

# Build the frontend WASM bundle into ./dist
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/target \
    trunk build --release --filehash false

# ============================================================
# Stage 2: Backend Rust binary builder
# ============================================================
FROM rust:slim-bookworm AS backend-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ENV CARGO_NET_RETRY=10
ENV CARGO_HTTP_TIMEOUT=300
ENV CARGO_HTTP_LOW_SPEED_LIMIT=0
ENV CARGO_HTTP_MULTIPLEXING=false
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY shared ./shared
COPY backend ./backend
# We compile only the backend; frontend is built separately above.
# A stub frontend dir is needed because the workspace lists it as a member.
COPY frontend ./frontend

ENV SQLX_OFFLINE=true
# Build with persistent cargo and target caches so incremental rebuilds are fast.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/target \
    cargo build --release --package backend --bin scholarvault && \
    cp /build/target/release/scholarvault /scholarvault

# ============================================================
# Stage 3: Slim runtime
# ============================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 sqlite3 curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=backend-builder /scholarvault /app/scholarvault
COPY --from=wasm-builder /build/dist /app/dist
COPY backend/src/db/migrations /app/migrations

RUN mkdir -p /app/data /app/evidence /app/backups

ENV HOST=0.0.0.0
ENV PORT=3000
ENV STATIC_DIR=/app/dist
ENV MIGRATIONS_DIR=/app/migrations

EXPOSE 3000

CMD ["/app/scholarvault"]
