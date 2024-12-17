# Use Debian Bookworm as the base image for both build and runtime
FROM debian:bookworm-slim as sys

# Install build dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    nodejs \
    npm \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install Rust using rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the Rust project files
COPY . .

# Build the Rust project
RUN cargo build --release

FROM scratch

COPY --from=sys /usr/bin/npm /usr/bin/npm
COPY --from=sys /usr/share/nodejs /usr/share/nodejs
COPY --from=sys /usr/lib/x86_64-linux-gnu/node_modules /usr/lib/x86_64-linux-gnu/node_modules
COPY --from=sys /usr/share/node_modules /usr/share/node_modules

COPY --from=sys /usr/bin/python3 /usr/bin/python3
COPY --from=sys /lib/x86_64-linux-gnu/libm.so.6 /lib/x86_64-linux-gnu/libm.so.6
COPY --from=sys /lib/x86_64-linux-gnu/libz.so.1 /lib/x86_64-linux-gnu/libz.so.1
COPY --from=sys /lib/x86_64-linux-gnu/libexpat.so.1 /lib/x86_64-linux-gnu/libexpat.so.1
COPY --from=sys /lib/x86_64-linux-gnu/libc.so.6 /lib/x86_64-linux-gnu/libc.so.6
COPY --from=sys /lib64/ld-linux-x86-64.so.2 /lib64/ld-linux-x86-64.so.2
COPY --from=sys /usr/lib/python3.11 /usr/lib/python3.11

COPY --from=sys /usr/bin/node /usr/bin/node
COPY --from=sys /usr/bin/nodejs /usr/bin/nodejs
COPY --from=sys /lib/x86_64-linux-gnu/libnode.so.108 /lib/x86_64-linux-gnu/libnode.so.108
COPY --from=sys /lib/x86_64-linux-gnu/libuv.so.1 /lib/x86_64-linux-gnu/libuv.so.1
COPY --from=sys /lib/x86_64-linux-gnu/libbrotlidec.so.1 /lib/x86_64-linux-gnu/libbrotlidec.so.1
COPY --from=sys /lib/x86_64-linux-gnu/libbrotlienc.so.1 /lib/x86_64-linux-gnu/libbrotlienc.so.1
COPY --from=sys /lib/x86_64-linux-gnu/libcares.so.2 /lib/x86_64-linux-gnu/libcares.so.2
COPY --from=sys /lib/x86_64-linux-gnu/libnghttp2.so.14 /lib/x86_64-linux-gnu/libnghttp2.so.14
COPY --from=sys /lib/x86_64-linux-gnu/libcrypto.so.3 /lib/x86_64-linux-gnu/libcrypto.so.3
COPY --from=sys /lib/x86_64-linux-gnu/libssl.so.3 /lib/x86_64-linux-gnu/libssl.so.3
COPY --from=sys /lib/x86_64-linux-gnu/libicui18n.so.72 /lib/x86_64-linux-gnu/libicui18n.so.72
COPY --from=sys /lib/x86_64-linux-gnu/libicuuc.so.72 /lib/x86_64-linux-gnu/libicuuc.so.72
COPY --from=sys /lib/x86_64-linux-gnu/libstdc++.so.6 /lib/x86_64-linux-gnu/libstdc++.so.6
COPY --from=sys /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/x86_64-linux-gnu/libgcc_s.so.1
COPY --from=sys /lib/x86_64-linux-gnu/libbrotlicommon.so.1 /lib/x86_64-linux-gnu/libbrotlicommon.so.1
COPY --from=sys /lib/x86_64-linux-gnu/libicudata.so.72 /lib/x86_64-linux-gnu/libicudata.so.72

COPY --from=sys /usr/src/app/target/release/code-runner /usr/local/bin/code-runner
