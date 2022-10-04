#!/bin/bash

podman run --network=host --rm redislabs/memtier_benchmark:1.3.0 -p 11311 --hide-histogram -t 20 -c 10 -n 50000 -d 65536
