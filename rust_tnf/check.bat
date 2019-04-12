clear
clear
cls
cd client
cargo check --target=i686-pc-windows-msvc --color=always
cd ../server
cargo check --target=i686-pc-windows-msvc --color=always
cd ..
