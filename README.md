# simple-secrets-server

This is an HTTP API driven secret store server written in Rust, backed by [etcd](https://github.com/etcd-io/etcd). ***It is _only_ for demonstration***. It is ***intentionally insecure*** in various places and has a naive implementation -- this is to allow for automated attacks and exploits to be written against it for demonstrational purposes. 

This app exports various application security heuristics via Prometheus endpoints for analysis in future examples.