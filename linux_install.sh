sudo install -Dm755 ddstats-rust /usr/bin/ddstats-rust
sudo setcap cap_sys_ptrace=eip /usr/bin/ddstats-rust
mkdir -p ${XDG_CONFIG_HOME:-~/.config}/ddstats-rust/
cp config.ron ${XDG_CONFIG_HOME:-~/.config}/ddstats-rust/
