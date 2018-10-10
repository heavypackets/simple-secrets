# simple-secrets

This is a simple API driven "secret" store server written in Rust, backed by [etcd](https://github.com/etcd-io/etcd). ***It is _only_ for demonstration***. It is ***intentionally insecure*** in various places and has an extremely naive implementation. Secrets are stored in plain text, and there is no built-in request limiting to prevent brute force.

This app exports various application access and security heuristics via Prometheus endpoints and a Fluentd forwarder for analysis in future examples. 

## API

| Action                    | Verb | Path                              | Returns      | Port |
| ------------------------- | ---- | --------------------------------- | ------------ | ---- |
| Login (Basic Auth Header) | GET  | /login                            | token        | 3000 |
| Get secret                | GET  | /get/{name}?token={token}         | secret value | 3000 |
| Set secret                | POST | /set/{name}/{value}?token={token} | secret hash  | 3000 |
| Metrics                   | GET  | /metrics                          | metrics      | 3001 |

### Environmental variables

| Variable              | Default               | Description                             |
| --------------------- | --------------------- | --------------------------------------- |
| ETCD_CLUSTER_MEMBERS  | http://localhost:2379 | Colon seperated list of ectd members    |
| FLUENTD_FORWARD_ADDR  | 127.0.0.1:24224       | TCP address of fluentd to foward logs   |
| TOKEN_EXPIRATION_SECS | 600                   | After how long do session tokens expire |

## Running

To build using local development environment:

```bash
cargo build
```

To build Docker container, containing a [SPIRE](https://github.com/spiffe/spire) agent and Envoy proxy):

```bash
make server
```

To build the demo environment, containing a 3 node [etcd](https://github.com/etcd-io/etcd) cluster, a [SPIRE](https://github.com/spiffe/spire) server, [Prometheus](https://github.com/prometheus/prometheus) and [Fluentd](https://github.com/fluent/fluentd):

```bash
make
make env
```

### Docker volumes

| Path                | Mount           | Description                          |
| ------------------- | --------------- | ------------------------------------ |
| ./server/envoy.yaml | /etc/envoy.yaml | Required. Envoy client configuration |

See docker-compose.yml for more details