# Build
FROM ghcr.io/osgeo/gdal:alpine-small-latest AS builder
WORKDIR /app

# Rust toolchain
RUN apk add --no-cache curl build-base gdal-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy Cargo files to keep this layer cached unless dependencies change
COPY Cargo.toml Cargo.lock ./
# Build dummy project to cache dependencies
RUN mkdir -p src/bin/server && \
    echo 'fn main() { println!("dummy main"); }' > src/main.rs && \
    echo 'fn main() { println!("dummy server"); }' > src/bin/server/main.rs && \
    touch src/lib.rs
RUN cargo build --release

# Now copy the actual source code and build the real project
COPY src ./src
RUN cargo build --release --bin los-server

# Runtime
FROM ghcr.io/osgeo/gdal:alpine-small-latest
WORKDIR /app
COPY --from=builder /app/target/release/los-server .
RUN ldd ./los-server
ENV HOST=0.0.0.0
CMD ["./los-server"]
