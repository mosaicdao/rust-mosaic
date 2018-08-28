FROM rust:1.28.0-jessie as builder

WORKDIR /usr/src/rust-mosaic
COPY . .
RUN cargo build --release


FROM debian:jessie

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl-dev

ENV HOME /home/mosaic
RUN groupadd -r mosaic && \
    useradd --no-log-init -r -m -g mosaic mosaic
WORKDIR $HOME
COPY --from=builder /usr/src/rust-mosaic/target/release/mosaic .
RUN chown -R mosaic:mosaic $HOME

USER mosaic:mosaic
ENTRYPOINT ["./mosaic"]
