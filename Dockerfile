# Build
FROM ghcr.io/osgeo/gdal:ubuntu-small-latest AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    build-essential \
    pkg-config \
    libgdal-dev \
    libsqlite3-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy Cargo files to keep this layer cached unless dependencies change
COPY Cargo.toml Cargo.lock ./
# Build dummy project to cache dependencies
RUN mkdir -p src/bin/server && \
    echo 'fn main() { println!("dummy main"); }' > src/main.rs && \
    echo 'fn main() { println!("dummy server"); }' > src/bin/server/main.rs && \
    touch src/lib.rs
RUN cargo build --release --locked --bin los-server

# Now copy the actual source code and build the real project
COPY src ./src
RUN cargo clean --release -p los && cargo build --release --locked --bin los-server

# Runtime
FROM ghcr.io/osgeo/gdal:ubuntu-small-latest
WORKDIR /app
COPY --from=builder /app/target/release/los-server .
RUN ldd ./los-server
ENV HOST=0.0.0.0
CMD ["./los-server"]
