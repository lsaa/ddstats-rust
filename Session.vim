let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
silent only
silent tabonly
cd ~/.local/source/ddstats-rust/ddstats-rust
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
set shortmess=aoO
badd +51 .github/workflows/rust.yml
badd +1 .github/workflows/package.sh
badd +1 /opt/st/config.h
badd +13 Cargo.toml
badd +61 ~/.Xresources
badd +1 test.sh
badd +1 test.sg
badd +46 src/main.rs
badd +29 src/threads.rs
badd +47 src/client.rs
badd +200 src/mem.rs
badd +21 src/consts.rs
badd +283 src/ui.rs
badd +96 src/config.rs
argglobal
%argdel
$argadd src/main.rs
set stal=2
tabnew
tabnew
tabnew
tabnew
tabnew
tabnew
tabnew
tabnew
tabnew
tabrewind
edit src/config.rs
argglobal
balt src/threads.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
silent! normal! zE
let &fdl = &fdl
let s:l = 107 - ((15 * winheight(0) + 32) / 64)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 107
normal! 019|
lcd ~/.local/source/ddstats-rust/ddstats-rust
tabnext
edit ~/.local/source/ddstats-rust/ddstats-rust/src/threads.rs
argglobal
balt ~/.local/source/ddstats-rust/ddstats-rust/src/config.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
silent! normal! zE
let &fdl = &fdl
let s:l = 29 - ((21 * winheight(0) + 32) / 64)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 29
normal! 053|
lcd ~/.local/source/ddstats-rust/ddstats-rust
tabnext
edit ~/.local/source/ddstats-rust/ddstats-rust/src/client.rs
argglobal
balt ~/.local/source/ddstats-rust/ddstats-rust/src/config.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
silent! normal! zE
let &fdl = &fdl
let s:l = 47 - ((19 * winheight(0) + 32) / 64)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 47
normal! 044|
lcd ~/.local/source/ddstats-rust/ddstats-rust
tabnext
edit ~/.local/source/ddstats-rust/ddstats-rust/src/ui.rs
argglobal
balt ~/.local/source/ddstats-rust/ddstats-rust/src/threads.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
silent! normal! zE
let &fdl = &fdl
let s:l = 284 - ((37 * winheight(0) + 32) / 64)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 284
normal! 09|
lcd ~/.local/source/ddstats-rust/ddstats-rust
tabnext
edit ~/.local/source/ddstats-rust/ddstats-rust/src/main.rs
argglobal
balt ~/.local/source/ddstats-rust/ddstats-rust/src/config.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
silent! normal! zE
let &fdl = &fdl
let s:l = 15 - ((14 * winheight(0) + 32) / 64)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 15
normal! 0
lcd ~/.local/source/ddstats-rust/ddstats-rust
tabnext
edit ~/.local/source/ddstats-rust/ddstats-rust/.github/workflows/package.sh
argglobal
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
silent! normal! zE
let &fdl = &fdl
let s:l = 1 - ((0 * winheight(0) + 32) / 64)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 1
normal! 024|
lcd ~/.local/source/ddstats-rust/ddstats-rust
tabnext
edit ~/.local/source/ddstats-rust/ddstats-rust/.github/workflows/package.sh
argglobal
balt ~/.local/source/ddstats-rust/ddstats-rust/.github/workflows/rust.yml
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
silent! normal! zE
let &fdl = &fdl
let s:l = 4 - ((3 * winheight(0) + 32) / 64)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 4
normal! 072|
lcd ~/.local/source/ddstats-rust/ddstats-rust
tabnext
edit ~/.local/source/ddstats-rust/ddstats-rust/src/mem.rs
argglobal
balt ~/.local/source/ddstats-rust/ddstats-rust/src/client.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
silent! normal! zE
let &fdl = &fdl
let s:l = 331 - ((26 * winheight(0) + 32) / 64)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 331
normal! 017|
lcd ~/.local/source/ddstats-rust/ddstats-rust
tabnext
edit ~/.local/source/ddstats-rust/ddstats-rust/src/config.rs
argglobal
balt ~/.local/source/ddstats-rust/ddstats-rust/src/mem.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
silent! normal! zE
let &fdl = &fdl
let s:l = 68 - ((24 * winheight(0) + 32) / 64)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 68
normal! 017|
lcd ~/.local/source/ddstats-rust/ddstats-rust
tabnext
edit ~/.local/source/ddstats-rust/ddstats-rust/src/consts.rs
argglobal
balt ~/.local/source/ddstats-rust/ddstats-rust/src/config.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
silent! normal! zE
let &fdl = &fdl
let s:l = 31 - ((15 * winheight(0) + 32) / 64)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 31
normal! 0
lcd ~/.local/source/ddstats-rust/ddstats-rust
tabnext 7
set stal=1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0&& getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20 shortmess=filnxtToOF
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
nohlsearch
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
