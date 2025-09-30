#!/bin/bash

PORT=$1
echo "Starting engine on port $PORT..."

./target/release/engine $PORT &

echo "Engine placeholder started for port $PORT." &