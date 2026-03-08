# === Build Stage ===
FROM rust:1.94-bookworm AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
# Pre-build dependencies (cache layer)
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release 2>/dev/null; true
RUN rm -rf src

COPY . .
RUN cargo build --release

# === Runtime Stage ===
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    locales \
    && sed -i 's/# zh_CN.UTF-8/zh_CN.UTF-8/' /etc/locale.gen \
    && locale-gen \
    && rm -rf /var/lib/apt/lists/*

ENV LANG=zh_CN.UTF-8
ENV LC_ALL=zh_CN.UTF-8
ENV TERM=xterm-256color

COPY --from=builder /build/target/release/cmdtyper /usr/local/bin/cmdtyper
COPY data/ /usr/local/share/cmdtyper/data/

# User data volume
VOLUME /data
ENV CMDTYPER_DATA_DIR=/data

ENTRYPOINT ["cmdtyper"]
