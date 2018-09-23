# simple-secrets-server

This is a simple API driven "secret" store server written in Rust, backed by [etcd](https://github.com/etcd-io/etcd). ***It is _only_ for demonstration***. It is ***intentionally insecure*** in various places and has an extremely naive implementation. Secrets are stored in plain text, and there is no built-in request limiting to prevent brute force attacks. This is to allow for exploits to be written against it for educational purposes.

This app exports various application access and security heuristics via Prometheus endpoints and a Fluentd forwarder for analysis in future examples.
