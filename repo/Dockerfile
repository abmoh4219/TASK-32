# syntax=docker/dockerfile:1.6
# ScholarVault — multi-stage build (frontend WASM via trunk + backend Axum binary)

# ============================================================
# Stage 1: WASM frontend builder
# ============================================================
FROM rust:slim-bookworm AS wasm-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk --version 0.20.3
RUN cargo install --locked wasm-bindgen-cli --version 0.2.95 || true

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
RUN trunk build --release --filehash false

# ============================================================
# Stage 2: Backend Rust binary builder
# ============================================================
FROM rust:slim-bookworm AS backend-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY shared ./shared
COPY backend ./backend
# We compile only the backend; frontend is built separately above.
# A stub frontend dir is needed because the workspace lists it as a member.
COPY frontend ./frontend

ENV SQLX_OFFLINE=true
RUN cargo build --release --package backend --bin scholarvault

# ============================================================
# Stage 3: Slim runtime
# ============================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=backend-builder /build/target/release/scholarvault /app/scholarvault
COPY --from=wasm-builder /build/dist /app/dist
COPY backend/src/db/migrations /app/migrations

RUN mkdir -p /app/data /app/evidence /app/backups

ENV HOST=0.0.0.0
ENV PORT=3000
ENV STATIC_DIR=/app/dist
ENV MIGRATIONS_DIR=/app/migrations

EXPOSE 3000

CMD ["/app/scholarvault"]
