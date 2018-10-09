# Base image
FROM envoyproxy/envoy

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
COPY proxy/start_envoy.sh /usr/local/bin/start_envoy.sh
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

CMD bash