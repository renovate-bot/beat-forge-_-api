FROM debian:bookworm-slim

LABEL org.opencontainers.image.source=https://github.com/beat-forge/api
EXPOSE 8000

# set the working directory
WORKDIR /usr/src/app/

# install dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# copy the binary from rust target folder
COPY target/release/api .

# set the entrypoint
CMD ["./api"]