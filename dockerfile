# Stage 1: Build the application using the Rust image
FROM rust:1.80.1 as builder

# Set the working directory inside the container
WORKDIR /usr/src/server

# Copy the files from your machine to the Docker image
COPY ./ ./

# Build your program for release
RUN cargo build --release

# Stage 2: Create a minimal runtime image using Debian Slim
FROM debian:bullseye-slim

# Install necessary libraries for the application to run
RUN apt-get update && apt-get install -y \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory in the final image (optional)
WORKDIR /usr/local/bin

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/server/target/release/server .

# Expose the necessary port (if your app uses one)
EXPOSE 8000

# Run the binary
CMD ["./server"]

