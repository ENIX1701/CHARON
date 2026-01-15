# === BUILD STAGE ===
FROM rust:alpine AS builder

WORKDIR /usr/src/app

RUN apk add --no-cache musl-dev pkgconfig perl make

COPY . .

RUN cargo test --release
RUN cargo build --release

# === RUNTIME ===
FROM alpine:edge

RUN apk add --no-cache libgcc ca-certificates
RUN addgroup -S charongroup && adduser -S charonuser -G charongroup

WORKDIR /home/charonuser

COPY --from=builder /usr/src/app/target/release/charon /usr/local/bin/charon

USER charonuser

CMD ["charon"]
