FROM rust:bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS runner

WORKDIR /app
COPY --from=builder /app/target/release/uljika-update-server /app/uljika-update-server
CMD ["/app/uljika-update-server"]