# Build stage
FROM rust:1.87-alpine AS builder
WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev postgresql-dev

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached if Cargo.toml doesn't change)
RUN cargo build --release && rm -rf src target/release/deps/portfolio*

# Copy source code
COPY src ./src

# Build the application with optimizations
RUN cargo build --release --locked \
    && strip target/release/portfolio-server

# Runtime stage - ultra lightweight
FROM alpine:3.19 AS runtime
WORKDIR /app

# Install only essential runtime dependencies
RUN apk add --no-cache ca-certificates libgcc postgresql-client

# Create uploads directory with proper permissions
RUN mkdir -p /uploads && chmod 755 /uploads

# Copy the binary from builder stage
COPY --from=builder /app/target/release/portfolio-server /usr/local/bin/app
RUN chmod +x /usr/local/bin/app

ENTRYPOINT ["/usr/local/bin/app"]