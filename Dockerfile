# Stage 1: Build
FROM rust:1.82.0-alpine AS builder
WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev perl make gcc

# Add the musl target for static linking
RUN rustup target add x86_64-unknown-linux-musl

# Copy manifests and build dependencies
COPY Cargo.toml Cargo.lock ./
# Make folders for the source code
RUN mkdir src/
# Copy the source code
COPY src/ ./src/

RUN cargo build --release

# Stage 2: Runtime
FROM alpine:3.20
WORKDIR /app

# Install necessary runtime dependencies
RUN apk add --no-cache ca-certificates

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/azure_email_service .

# Set the binary as the entrypoint
ENTRYPOINT ["./azure_email_service"]