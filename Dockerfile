FROM rust:latest

LABEL org.opencontainers.image.source=https://github.com/beat-forge/api

WORKDIR /app

COPY . .

RUN cargo build --release

RUN mv ./target/release/gql-api .

RUN rm -rf ./target

EXPOSE 8080

CMD ["./gql-api"]
