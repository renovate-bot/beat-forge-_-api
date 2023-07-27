FROM rust:latest

LABEL org.opencontainers.image.source=https://github.com/beat-forge/api

# set the application directory
WORKDIR /app

# copy the release binary
COPY ./target/release/gql-api /app

EXPOSE 8080

# set the entrypoint
CMD ["./gql-api"]
