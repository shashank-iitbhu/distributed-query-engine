#!/bin/bash

ENGINE_PORTS="$@"
echo "Starting driver with engine ports: $ENGINE_PORTS"

./target/release/driver $@

touch output.txt
echo "Driver finished and created output.txt."