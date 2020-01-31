#!/usr/bin/env sh
set -euo pipefail
export CPLUS_INCLUDE_PATH="D:\\Program Files (x86)\\Microsoft Visual Studio 10.0\\VC\\include"
cd generate
cargo run --release --features=generate,r476
cd ..
cargo test --features=bindings,r476,client,server --target=i686-pc-windows-msvc
