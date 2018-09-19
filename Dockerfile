FROM rust:1.29.0

WORKDIR /usr/src/simple-secrets
COPY . .

RUN cargo install

CMD ["simple-secrets"]