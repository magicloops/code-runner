# Use Alpine as the base image for both build and runtime
FROM alpine:3.18 as builder

# Install build dependencies
RUN apk add --no-cache \
    curl \
    gcc \
    musl-dev \
    nodejs \
    npm \
    python3 \
    py3-pip

# Install Rust using rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the Rust project files
COPY . .

# Build the Rust project
RUN cargo build --release

# Create the final image
FROM alpine:3.18

# Install runtime dependencies
RUN apk add --no-cache \
    nodejs \
    npm \
    python3 \
    py3-pip \
    libgcc

# Copy the built executable from the builder stage
COPY --from=builder /usr/src/app/target/release/code-runner /usr/local/bin/code-runner

# Set the working directory
WORKDIR /usr/src/app

# Copy any additional files needed for your application
COPY . .

# Expose the port your Rust webserver will run on
EXPOSE 4000 

# Command to run your application
CMD ["code-runner"]
