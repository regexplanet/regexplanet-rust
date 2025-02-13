FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev

WORKDIR /app
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release

RUN find .

FROM scratch

ARG COMMIT="(not set)"
ARG LASTMOD="(not set)"
ENV COMMIT=$COMMIT
ENV LASTMOD=$LASTMOD

WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/regexplanet-rust /app/regexplanet-rust
COPY ./static /app/static
CMD ["/app/regexplanet-rust"]
