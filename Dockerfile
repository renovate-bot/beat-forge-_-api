FROM debian:bullseye-slim

LABEL org.opencontainers.image.source=https://github.com/beat-forge/api
EXPOSE 8000

# set the working directory
WORKDIR /usr/src/app/

# copy the binary from rust target folder
COPY target/release/api .

# set the entrypoint
CMD ["./api"]