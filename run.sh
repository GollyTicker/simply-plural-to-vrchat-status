#!/bin/bash

set -euo pipefail

echo "Build..."
cargo build --release
echo "Done."

echo "Run:"
set -a; source .env; set +a
./target/release/vrchat_sps_status

