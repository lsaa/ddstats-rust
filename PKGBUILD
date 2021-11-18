_pkgname=ddstats-rust
pkgname=${_pkgname}-git
pkgver=0.6.10.stable1
pkgrel=1
source=("${_pkgname}::git+https://github.com/lsaa/ddstats-rust")
makedepends=('rust' 'cargo')
arch=('x86_64')

build() {
    cd "${_pkgname}"
	cargo build --release
}

package() {
	setcap cap_sys_ptrace=eip ddstats-rust
	mkdir -p ${DESTDIR}${PREFIX}/bin
    cp -f ddstats-rust ${DESTDIR}${PREFIX}/bin
    chmod 755 ${DESTDIR}${PREFIX}/bin/ddstats-rust
}
