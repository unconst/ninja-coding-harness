#!/bin/bash
timeout 30s cargo run --bin continuous-loop 2>&1 | while read line; do
    echo "[$(date '+%H:%M:%S')] $line"
    if [[ "$line" == *"DEBUG: Converting challenge"* ]]; then
        echo "  >> CHALLENGE CONVERSION IN PROGRESS"
    fi
done