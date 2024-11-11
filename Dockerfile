# Stage 1: Build
FROM rust:1.82.0-alpine AS builder
WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev perl make gcc

# Add the musl target for static linking
RUN rustup target add x86_64-unknown-linux-musl

# Copy manifests and build dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src/
COPY src/ ./src/

# Build the application
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Runtime
FROM scratch
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/azure_email_service .

# Set the binary as the entrypoint
ENTRYPOINT ["./azure_email_service"]