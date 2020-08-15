#!/usr/bin/env bash

RED='\033[0;31m'
GREEN='\033[0;32m'
LIGHT_GREEN='\033[1;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)
cd "${SCRIPT_DIR}"

# compile shaders
./build-shaders.sh || exit 1

# run rust
cargo +nightly build --release --features simd-nightly