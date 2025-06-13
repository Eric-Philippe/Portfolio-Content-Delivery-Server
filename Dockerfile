# Multi-stage build for smaller final image
FROM rust:1.70-alpine AS builder

# Install required dependencies for building
RUN apk add --no-cache \
    musl-dev \
    sqlite-dev \
    pkgconfig \
    openssl-dev

# Set the working directory
WORKDIR /app

# Copy the Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release
RUN rm src/main.rs

# Copy the actual source code
COPY src/ ./src/

# Build the application
RUN cargo build --release

# Runtime stage
FROM alpine:3.18

# Install runtime dependencies
RUN apk add --no-cache \
    sqlite \
    ca-certificates \
    tzdata

# Create a non-root user
RUN addgroup -g 1000 appuser && \
    adduser -D -s /bin/sh -u 1000 -G appuser appuser

# Set the working directory
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/portfolio-server .

# Create necessary directories
RUN mkdir -p data uploads && \
    chown -R appuser:appuser /app

# Copy sample data (optional)
COPY sample_data.sql ./

# Switch to non-root user
USER appuser

# Expose the port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/albums || exit 1

# Run the application
CMD ["./portfolio-server"]
