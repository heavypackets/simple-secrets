FROM rust:1.29.0

RUN apt-get update && apt-get -yy install clang && rm -r /var/cache/apt/archives

WORKDIR /usr/src/simple-secrets
COPY . .

RUN cargo install

CMD ["simple-secrets"]