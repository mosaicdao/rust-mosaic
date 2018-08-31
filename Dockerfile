# Using the official rust image as a builder stage
FROM rust:1.28.0-jessie as builder

# Copying the source and compiling it with cargo
WORKDIR /usr/src/rust-mosaic
COPY . .
RUN cargo build --release


# Second stage for distribution.
# Based on debian, as the rust image is based on debian and libraries
# are dynamically linked. We could use musl instead to use a smaller
# base image, e.g. alpine, for distribution.
FROM debian:jessie

# libssl-dev is a dependency of rust-mosaic.
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl-dev

# Creating a user mosaic with group and home dir.
ENV HOME /home/mosaic
RUN groupadd -r mosaic && \
    useradd --no-log-init -r -m -g mosaic mosaic
WORKDIR $HOME

# Copying the binary from the build stage to the home directory.
COPY --from=builder /usr/src/rust-mosaic/target/release/mosaic .
RUN chown -R mosaic:mosaic $HOME

USER mosaic:mosaic
ENTRYPOINT ["./mosaic"]
