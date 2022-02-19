sudo apt-get install zip
echo "# Automated Release from Github Action" > body.md
mkdir ddstats-rust-windows
mkdir ddstats-rust-linux
cp default_cfg.ron ddstats-rust-windows
cp default_cfg.ron ddstats-rust-linux
cp default_cfg.ron ddstats-rust-windows/config.ron
cp default_cfg.ron ddstats-rust-linux/config.ron
cp target/x86_64-unknown-linux-gnu/release/ddstats-rust ddstats-rust-linux
cp target/x86_64-pc-windows-gnu/release/ddstats-rust.exe ddstats-rust-windows
cp linux_install.sh ddstats-rust-linux
cp README_LINUX.txt ddstats-rust-linux
zip -j ddstats-rust-linux-x86_64.zip ddstats-rust-linux
zip -j ddstats-rust-windows-x86_64.zip ddstats-rust-windows
