clear
clear
cls
cd client_dll
cargo build --target=i686-pc-windows-msvc --color=always
cd ../server_dll
cargo build --target=i686-pc-windows-msvc --color=always
cd ..
copy target\i686-pc-windows-msvc\debug\tnf_client_dll.dll ..\scripts\rust_tnf_client.dll
copy target\i686-pc-windows-msvc\debug\tnf_server_dll.dll ..\scripts\rust_tnf_server.dll
