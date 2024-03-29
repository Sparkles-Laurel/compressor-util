FROM rust:latest AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM fedora:latest AS runtime
WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/target/release/compressor-util .
EXPOSE 8080
CMD ["./compressor-util"]
