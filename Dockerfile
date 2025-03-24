# Use the official Rust image
FROM rust:1.75-slim

# Install required build tools and Clang (for bindgen)
RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    libclang-dev \
    cmake \
    pkg-config \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy project files
COPY . .

# Build in release mode
RUN cargo build --release

# Run server by default
CMD ["./target/release/rust-distributed-cache"]
