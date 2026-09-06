" Safe to source more than once. setfiletype respects earlier explicit detection
" while replacing generic FALLBACK guesses for files that start with '#'.
augroup meowy_filetype
  autocmd!
  autocmd BufRead,BufNewFile *.mwy if exists('g:did_load_filetypes') | setfiletype meowy | endif
augroup END
