#!/bin/bash

if ! command -v redis-commander >/dev/null 2>&1; then
  echo "redis-commander is not installed"
  exit 1
fi

if ! command -v pgweb >/dev/null 2>&1; then
  echo "pgweb is not installed"
  exit 1
fi

echo "Starting Redis Commander..."
PORT=8081 \
REDIS_HOSTS=local:redis:6379 \
redis-commander >/dev/null 2>&1 &

echo "Starting pgweb..."
pgweb \
  --listen=8082 \
  --url="postgresql://kestrel:kestrel@localhost:5432/kestrel" \
  >/dev/null 2>&1 &

echo "Tools running:"
echo " - Redis Commander: http://localhost:8081"
echo " - pgweb: http://localhost:8082"

wait
