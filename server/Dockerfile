# Build OpenSSL 1.1
FROM envoyproxy/envoy as openssl

RUN apt-get update && \
    apt-get install -qqy \
    curl libfile-spec-perl gcc make && \
    rm -r /var/cache/apt/archives

ARG OPENSSL_RELEASE=https://www.openssl.org/source/openssl-1.1.0f.tar.gz
RUN curl --silent --location $OPENSSL_RELEASE | tar -xzf -
RUN cd openssl-1.1.0f && ./config && make && make install > /dev/null

# Base image
FROM envoyproxy/envoy as base

RUN apt-get update && \
    apt-get install -qqy \
    ca-certificates gettext dnsutils python curl && \
    rm -r /var/cache/apt/archives

# Install SPIRE agent
ARG SPIRE_VERSION=0.6.0
ARG SPIRE_RELEASE=https://github.com/spiffe/spire/releases/download/${SPIRE_VERSION}/spire-${SPIRE_VERSION}-linux-x86_64-glibc.tar.gz
ARG SPIRE_DIR=/opt/spire

RUN curl --silent --location $SPIRE_RELEASE | tar -xzf -
RUN mv spire-${SPIRE_VERSION} ${SPIRE_DIR}

WORKDIR ${SPIRE_DIR}
COPY conf/spire-agent.conf conf/agent/agent.conf

# Install envoy config
COPY server/envoy.yaml /etc/envoy.yaml
COPY server/start_envoy.sh /usr/local/bin/start_envoy.sh
RUN chmod +x /usr/local/bin/start_envoy.sh

# Install sidecar
ARG SPIFFE_HELPER_RELEASE="https://github.com/spiffe/spiffe-helper/releases/download/0.3/spiffe-helper_0.3_linux_amd64.tar.gz"
ARG SPIFFE_HELPER_DIR=/opt/spiffe-helper
RUN mkdir ${SPIFFE_HELPER_DIR}
COPY conf/spiffe-helper.conf ${SPIFFE_HELPER_DIR}/helper.conf
COPY bin/hot-restarter.py /usr/local/bin/
RUN mkdir -p /certs && chmod 777 /certs
RUN curl --silent --location ${SPIFFE_HELPER_RELEASE} | tar -xzf -
RUN mv sidecar ${SPIFFE_HELPER_DIR}

# Install OpenSSL
COPY --from=openssl /usr/local/bin/openssl /usr/bin/
COPY --from=openssl /usr/local/lib/libssl.so /usr/lib/
COPY --from=openssl /usr/local/lib/libssl.so.1.1 /usr/lib/
COPY --from=openssl /usr/local/lib/libcrypto.so /usr/lib/
COPY --from=openssl /usr/local/lib/libcrypto.so.1.1 /usr/lib/

# Build app
FROM rust:1.29.0 as build

RUN apt-get update && apt-get -yy install clang && rm -r /var/cache/apt/archives

WORKDIR /usr/src
RUN USER=root cargo new --bin simple-secrets
WORKDIR /usr/src/simple-secrets
COPY server/Cargo.lock ./Cargo.lock
COPY server/Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm ./src/*.rs
COPY server/src ./src
RUN cargo build --release

FROM base

# Install server
COPY --from=build /usr/src/simple-secrets/target/release/simple-secrets /usr/local/bin

ENTRYPOINT ["simple-secrets"]