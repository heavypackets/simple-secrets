#!/bin/bash

envoy -c /etc/envoy.yaml --service-node `hostname` --restart-epoch $RESTART_EPOCH
