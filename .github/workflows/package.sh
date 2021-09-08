sudo apt-get install zip
echo "# Automated Release from Github Action" > body.md
target=$1
if [[ $target == *"linux"* ]]; then
  zip ddstats-rust-linux-x86_64.zip target/x86_64-unknown-linux-gnu/release/ddstats-rust
fi

if [[ $target == *"windows"* ]]; then
  zip ddstats-rust-windows-x86_64.zip target/x86_64-pc-windows-gnu/release/ddstats-rust.exe
fi
