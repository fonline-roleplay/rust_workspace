#!/usr/bin/env sh
set -euo pipefail
toast "$@"
cargo test --features=bindings,r357,client,server --target=i686-pc-windows-msvc
