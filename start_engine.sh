#!/bin/bash

PORT=$1
echo "Starting engine on port $PORT..."

./target/release/engine $PORT &

echo "Engine started for port $PORT." &