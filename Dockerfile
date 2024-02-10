FROM rust:1.76 as builder

WORKDIR /app

COPY . .

RUN rustup default nightly && rustup update
RUN cargo build --release

FROM alpine:latest

COPY --from=builder /app/target/release/rusty_rest /usr/local/bin/rusty_rest

ENV RUST_LOG=info

EXPOSE 8080

CMD ["rusty_rest"]