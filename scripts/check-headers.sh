#!/usr/bin/env bash

PATTERN="Kestrel - a modern instant-messaging service written in Rust"

find . -name "*.rs" | while read -r file; do
    if ! head -n 30 "$file" | grep -q "$PATTERN"; then
        echo "MISSING HEADER: $file"
    fi
done
