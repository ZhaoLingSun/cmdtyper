#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release
install -Dm755 target/release/cmdtyper "$HOME/.local/bin/cmdtyper"
echo "Installed: $HOME/.local/bin/cmdtyper"
echo "Source: $(pwd)/target/release/cmdtyper"
