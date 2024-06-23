# Stage 1: Build the Rust application
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt -y update && apt install -y musl-tools musl-dev gcc-x86-64-linux-gnu build-essential pkg-config libssl-dev

WORKDIR /app

COPY ./ ./

ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'
ENV CC='gcc'
ENV CC_x86_64_unknown_linux_musl=x86_64-linux-gnu-gcc
ENV CC_x86_64-unknown-linux-musl=x86_64-linux-gnu-gcc

RUN cargo build --target x86_64-unknown-linux-musl --release

# Stage 2: Create a minimal image with the compiled binary
FROM scratch

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rusty-web-development ./
COPY --from=builder /app/.env ./

CMD ["/app/rusty-web-development"]