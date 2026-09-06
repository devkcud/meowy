" Buffer-local editing defaults for meowy.
if exists('b:did_ftplugin')
  finish
endif
let b:did_ftplugin = 1

let s:save_cpo = &cpoptions
set cpoptions&vim

setlocal expandtab shiftwidth=4 softtabstop=4
setlocal autoindent nocindent nosmartindent
setlocal commentstring=#\ %s\ #
setlocal comments=s:#,e:#
" Repeating '#' on a new line would close a delimited comment. Do not wrap
" code or literal strings automatically, or apply C's word-based indentation.
setlocal formatoptions-=t formatoptions-=r formatoptions-=o
setlocal suffixesadd+=.mwy

let b:undo_ftplugin = 'setlocal expandtab< shiftwidth< softtabstop< autoindent< cindent< smartindent< commentstring< comments< formatoptions< suffixesadd<'

let &cpoptions = s:save_cpo
unlet s:save_cpo
