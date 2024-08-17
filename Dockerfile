FROM rust:1-bookworm as builder

RUN apt-get update && apt-get install -y \
    ca-certificates \
    dumb-init

WORKDIR /app
COPY . .
RUN cargo build --bins --release

FROM debian:bookworm-slim
LABEL org.opencontainers.image.source https://github.com/regexplanet/regexplanet-rust

ARG COMMIT="(not set)"
ARG LASTMOD="(not set)"
ENV COMMIT=$COMMIT
ENV LASTMOD=$LASTMOD

WORKDIR /app
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /usr/bin/dumb-init /usr/bin/dumb-init
COPY --from=builder /app/target/release/regexplanet-rust /app/regexplanet-rust
COPY ./static /app/static
ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["/app/regexplanet-rust"]
