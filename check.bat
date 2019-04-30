clear
clear
cls
cd client_dll
cargo check --target=i686-pc-windows-msvc --color=always
cd ../server_dll
cargo check --target=i686-pc-windows-msvc --color=always
cd ..
