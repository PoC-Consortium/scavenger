FROM liuchong/rustup:stable-musl as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features=simd

FROM alpine:latest
RUN apk update && apk upgrade && apk add --no-cache bash util-linux
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/scavenger .
ENTRYPOINT ["./scavenger"]
CMD ["--config","/data/config.yaml"]
