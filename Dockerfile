FROM rust:1.70 as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/app/target/release/repo-analyzer-api .
COPY .env.example .env

EXPOSE 3000

CMD ["./repo-analyzer-api"] 