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
badd +44 .github/workflows/rust.yml
badd +1 test.sh
badd +1 test.sg
badd +36 src/main.rs
badd +36 src/threads.rs
badd +1 src/client.rs
badd +200 src/mem.rs
badd +43 src/consts.rs
badd +1 src/ui.rs
argglobal
%argdel
$argadd src/main.rs
set stal=2
tabnew
tabnew
tabnew
tabnew
tabrewind
edit src/main.rs
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
let s:l = 23 - ((22 * winheight(0) + 22) / 45)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 23
normal! 05|
lcd ~/.local/source/ddstats-rust/ddstats-rust
tabnext
edit ~/.local/source/ddstats-rust/ddstats-rust/src/ui.rs
argglobal
balt ~/.local/source/ddstats-rust/ddstats-rust/src/main.rs
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
let s:l = 24 - ((23 * winheight(0) + 22) / 45)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 24
normal! 05|
lcd ~/.local/source/ddstats-rust/ddstats-rust
tabnext
edit ~/.local/source/ddstats-rust/ddstats-rust/src/threads.rs
argglobal
balt ~/.local/source/ddstats-rust/ddstats-rust/src/main.rs
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
let s:l = 21 - ((5 * winheight(0) + 22) / 45)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 21
normal! 0
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
let s:l = 41 - ((4 * winheight(0) + 22) / 45)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 41
normal! 05|
lcd ~/.local/source/ddstats-rust/ddstats-rust
tabnext
edit ~/.local/source/ddstats-rust/ddstats-rust/src/consts.rs
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
let s:l = 1 - ((0 * winheight(0) + 22) / 45)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 1
normal! 0
lcd ~/.local/source/ddstats-rust/ddstats-rust
tabnext 5
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
