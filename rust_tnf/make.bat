cd client
cargo build --release --target=i686-pc-windows-msvc
cd ../server
cargo build --release --target=i686-pc-windows-msvc
cd ..
copy target\i686-pc-windows-msvc\release\tnf_client.dll ..\scripts\rust_tnf_client.dll
copy target\i686-pc-windows-msvc\release\tnf_server.dll ..\scripts\rust_tnf_server.dll