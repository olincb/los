# Base
FROM ghcr.io/osgeo/gdal:ubuntu-small-3.13.3 AS base
WORKDIR /app

# libpoppler156 and liblcms2-2 are the runtime libraries of the PDF driver plugin copied below.
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpoppler156 \
    liblcms2-2 \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# The small image's GDAL can write PDFs but not read them, and GeoPDF rasterization needs to read.
# The full image of the same release ships the reader as a plugin, which GDAL loads from this directory.
COPY --from=ghcr.io/osgeo/gdal:ubuntu-full-3.13.3 \
    /usr/lib/x86_64-linux-gnu/gdalplugins/gdal_PDF.so \
    /usr/lib/x86_64-linux-gnu/gdalplugins/
# The plugin defaults to PDFium. Poppler is what Ubuntu's and Homebrew's GDAL builds render with.
ENV GDAL_PDF_LIB=POPPLER

# Build
FROM base AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    build-essential \
    pkg-config \
    libsqlite3-dev \
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
FROM base AS runtime
COPY --from=builder /app/target/release/los-server .
COPY --from=builder /app/src/bin/server/static ./static
RUN ldd ./los-server
ENV HOST=0.0.0.0
CMD ["./los-server"]
