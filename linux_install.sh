sudo install -Dm755 ddstats-rust /usr/bin/ddstats-rust
sudo setcap cap_sys_ptrace=eip /usr/bin/ddstats-rust
