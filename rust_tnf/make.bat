clear
clear
cls
cd client_dll
cargo build --release --target=i686-pc-windows-msvc --color=always
cd ../server_dll
cargo build --release --target=i686-pc-windows-msvc --color=always
cd ..
copy target\i686-pc-windows-msvc\release\tnf_client_dll.dll ..\scripts\rust_tnf_client.dll
copy target\i686-pc-windows-msvc\release\tnf_server_dll.dll ..\scripts\rust_tnf_server.dll
