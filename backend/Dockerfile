# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

# Copy source code
COPY src/ src/

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/resume-scanner-backend /app/resume-scanner-backend

# Create .env file placeholder
ENV PORT=3001
ENV RUST_LOG=info

EXPOSE 3001

CMD ["./resume-scanner-backend"]
