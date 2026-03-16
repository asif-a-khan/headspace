# --- Stage 1: Build the Vue frontend ---
FROM node:22-alpine AS frontend-builder

WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
# vite outputs to ../static/dist relative to frontend/
RUN mkdir -p /app/static/dist
RUN npm run build


# --- Stage 2: Build the Rust binary ---
FROM rust:1.88-bookworm AS rust-builder

WORKDIR /app

# Cache dependency compilation by building a dummy project first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release && rm -rf src

# Now copy real source and rebuild (only our code recompiles)
COPY src/ src/
COPY templates/ templates/
COPY askama.toml ./
COPY migrations/ migrations/
RUN touch src/main.rs
RUN cargo build --release


# --- Stage 3: Slim runtime image ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary
COPY --from=rust-builder /app/target/release/headspace ./headspace

# Copy frontend build output
COPY --from=frontend-builder /app/static/ ./static/

# Copy templates (askama renders at runtime for page shells)
COPY templates/ ./templates/
COPY askama.toml ./

# Copy fonts for PDF generation
COPY fonts/ ./fonts/

# Copy migrations (app runs them on startup)
COPY migrations/ ./migrations/

EXPOSE 8000

CMD ["./headspace"]
