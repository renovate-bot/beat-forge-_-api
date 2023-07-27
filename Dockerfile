FROM rust:latest

LABEL org.opencontainers.image.source=https://github.com/beat-forge/api
EXPOSE 8000

# install the build dependencies
WORKDIR /usr/src/app/source

# copy the source tree 
COPY . .

# build the application
RUN cargo build --release

# move to the root directory
WORKDIR /usr/src/app/

# copy the binary
RUN cp ./source/target/release/api/gql-api .

# remove the source tree
RUN rm -rf ./source

# set the startup command
CMD ["./gql-api"]