#!/bin/bash
set -e

trap 'kill $(jobs -p) 2>/dev/null' EXIT

# Free ports if already in use
lsof -ti :3000 | xargs kill -9 2>/dev/null || true
lsof -ti :4173 | xargs kill -9 2>/dev/null || true

(cd backend && cargo run --release) &
(cd frontend && npm run prod) &

# Wait for vite preview to be ready, then open browser
until curl -s http://localhost:4173 > /dev/null 2>&1; do sleep 1; done
open http://localhost:4173

wait
