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

ENV LANG=C.UTF-8
ENV LC_ALL=C.UTF-8
ENV TERM=xterm-256color

COPY --from=builder /build/target/release/cmdtyper /usr/local/bin/cmdtyper
COPY data/commands/ /usr/local/share/cmdtyper/data/commands/
COPY data/lessons/ /usr/local/share/cmdtyper/data/lessons/
COPY data/symbols/ /usr/local/share/cmdtyper/data/symbols/
COPY data/system/ /usr/local/share/cmdtyper/data/system/

# User data volume
VOLUME /userdata
ENV CMDTYPER_DATA_DIR=/usr/local/share/cmdtyper/data
ENV CMDTYPER_USER_DIR=/userdata

ENTRYPOINT ["cmdtyper"]
