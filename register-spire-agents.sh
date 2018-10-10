#!/bin/bash
docker-compose exec -d simple-secrets sh -c "cd /opt/spire && ./spire-agent run -joinToken $(docker-compose exec spire-server sh -c 'cd /opt/spire && ./spire-server token generate -spiffeID spiffe://example.org/simple-secrets' | sed 's/Token: //' | sed "s/$(printf '\r')//")"
docker-compose exec -d simple-secrets sh -c "cd /opt/spiffe-helper && ./sidecar"

docker-compose exec -d prometheus-proxy sh -c "cd /opt/spire && ./spire-agent run -joinToken $(docker-compose exec spire-server sh -c 'cd /opt/spire && ./spire-server token generate -spiffeID spiffe://example.org/prometheus' | sed 's/Token: //' | sed "s/$(printf '\r')//")"
docker-compose exec -d prometheus-proxy sh -c "cd /opt/spiffe-helper && ./sidecar"

docker-compose exec -d fluentd-proxy sh -c "cd /opt/spire && ./spire-agent run -joinToken $(docker-compose exec spire-server sh -c 'cd /opt/spire && ./spire-server token generate -spiffeID spiffe://example.org/fluentd' | sed 's/Token: //' | sed "s/$(printf '\r')//")"
docker-compose exec -d fluentd-proxy sh -c "cd /opt/spiffe-helper && ./sidecar"