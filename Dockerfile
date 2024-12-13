# Start with a base image that includes Rust
FROM rust:latest as builder

# Install Node.js and Python
RUN apt-get update && apt-get install -y \
    nodejs \
    npm \
    python3 \
    python3-pip

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the Rust project files
COPY . .

# Build the Rust project
RUN cargo build --release

# Create the final image
FROM debian:buster-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    nodejs \
    npm \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Copy the built executable from the builder stage
COPY --from=builder /usr/src/app/target/release/code-runner /usr/local/bin/code-runner

# Set the working directory
WORKDIR /usr/src/app

# Copy any additional files needed for your application
COPY . .

# Expose the port your Rust webserver will run on
EXPOSE 8080

# Command to run your application
CMD ["code-runner"]
