# Use Rust official image
FROM rust:latest

# Set working directory inside the container
WORKDIR /app

# Copy Cargo files first (for caching dependencies)
COPY Cargo.toml Cargo.lock ./

# Create empty src directory to avoid build errors
RUN mkdir -p src 

# Copy the source code after dependencies are cached
COPY src/ src/

# Build the application in release mode
RUN cargo build --release

# Set the binary as the entry point
CMD ["./target/release/MongoTurbo"]
