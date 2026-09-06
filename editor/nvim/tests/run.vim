" Run with: nvim --headless -u NONE -i NONE -n -S editor/nvim/tests/run.vim
"       or: vim -Nu NONE -i NONE -n -es -S editor/nvim/tests/run.vim
set nocompatible nomore noswapfile hidden
let s:runtime = fnamemodify(expand('<sfile>:p'), ':h:h')
let s:root = fnamemodify(s:runtime, ':h:h')
execute 'set runtimepath^=' . fnameescape(s:runtime)
filetype plugin on
syntax enable
let v:errmsg = ''

function! s:At(line, token, group, ...) abort
  let l:lnum = index(getline(1, '$'), a:line) + 1
  call assert_true(l:lnum > 0, 'Fixture line exists: ' . a:line)
  if !l:lnum
    return
  endif
  let l:column = stridx(a:line, a:token) + 1 + get(a:000, 0, 0)
  call assert_true(l:column > 0, 'Fixture token exists: ' . a:token)
  call assert_equal(a:group, synIDattr(synID(l:lnum, l:column, 1), 'name'),
        \ a:token . ' in ' . a:line)
endfunction

execute 'edit ' . fnameescape(s:runtime . '/tests/fixtures/syntax.mwy')
call assert_equal('meowy', &l:filetype)
call assert_equal('meowy', b:current_syntax)
call assert_equal('# %s #', &l:commentstring)
call assert_equal('s:#,e:#', &l:comments)
call assert_equal(4, &l:shiftwidth)
call assert_equal(4, &l:softtabstop)
call assert_true(&l:expandtab)
call assert_true(&l:autoindent)
call assert_false(&l:cindent)
call assert_false(&l:smartindent)
call assert_false(&l:formatoptions =~# '[tro]')
call assert_true(index(split(&l:suffixesadd, ','), '.mwy') >= 0)

" No language keyword groups: known names remain ordinary bindings/calls.
for s:name in ['true', 'false', 'null', 'self', 'unsafe', 'where', 'mut', 'leave', 'restart', 'int32']
  let s:line = filter(getline(1, '$'), 'v:val =~# "^" . s:name . " :"')[0]
  call s:At(s:line, s:name, 'meowyBinding')
endfor
call s:At('enabled : true', 'true', 'meowyBuiltinValue')
call s:At('disabled : false', 'false', 'meowyBuiltinValue')
call s:At('empty : null', 'null', 'meowyBuiltinValue')
call s:At('object.leave()', 'leave', 'meowyCall')
call s:At('object.restart()', 'restart', 'meowyCall')
call s:At('debug : @"debug"', '@"', 'meowyImport')
call s:At('local : @"./path/packet.mwy"', 'packet', 'meowyImport')

for s:pair in [['decimal : 1_024', '1_024'], ['hex : 0xff_AB', 'ff_AB'], ['binary : 0b1010_0011', '0011'], ['negative : -42', '42']]
  call s:At(s:pair[0], s:pair[1], 'meowyNumber')
endfor
call s:At('fraction : 3.5', '3.5', 'meowyFloat')
call s:At('exponent : 1.0e-3', 'e-3', 'meowyFloat')
call s:At('negative : -42', '-', 'meowyOperator')
call s:At('21.(double)', '.', 'meowyDispatch')

call s:At('read_word <uint32> : (address <*uint32>) !{', '!', 'meowyUnchecked')
call s:At('    -> memory.read<uint32>(address)', '->', 'meowyEmit')
call s:At('    -> memory.read<uint32>(address)', 'read', 'meowyCall')
call s:At('    -> memory.read<uint32>(address)', 'uint32', 'meowyTypeName')
call s:At('pointer <*!uint32> : address', '*!', 'meowyTypeOperator')
call s:At('shared <&uint32> : &count', '&count', 'meowyBorrow')
call s:At('exclusive <&!uint32> : &!count', '&!', 'meowyTypeOperator')
call s:At('exclusive <&!uint32> : &!count', '&!count', 'meowyBorrow')
call s:At('inverted : !({ -> true })', '!', 'meowyOperator')

call s:At('<Pair<:T : memory.Copy & tasks.Send>> : <{', 'Pair', 'meowyTypeName')
call s:At('<Pair<:T : memory.Copy & tasks.Send>> : <{', 'Copy', 'meowyTypeName')
call s:At('<Pair<:T : memory.Copy & tasks.Send>> : <{', '&', 'meowyTypeOperator')
call s:At('<Pair<:T : memory.Copy & tasks.Send>> : <{', '>>', 'meowyTypeDelimiter')
call s:At('<Pair<:T : memory.Copy & tasks.Send>> : <{', '>>', 'meowyTypeDelimiter', 1)
call s:At('    first <T>', 'first', 'meowyIdentifier')
call s:At('callback <!(uint8[], <&!uint32>) -> uint32> : read_word', '->', 'meowyTypeOperator')
call s:At('callback <!(uint8[], <&!uint32>) -> uint32> : read_word', 'uint32> :', 'meowyTypeName')
call s:At('handler <(channel.Item<uint32>) -> tasks.Outcome<uint32>> : work', 'Outcome', 'meowyTypeName')
call s:At('nested <collections.Vector<collections.Vector<uint8>>> : value', 'uint8', 'meowyTypeName')
for s:offset in range(3)
  call s:At('nested <collections.Vector<collections.Vector<uint8>>> : value', '>>>', 'meowyTypeDelimiter', s:offset)
endfor
call s:At('rows <uint8[16]> : []', '16', 'meowyNumber')
call s:At('view <uint8[]> : rows.slice()', '[]', 'meowyTypeDelimiter')
call s:At('<Theme> : <"dark"><"light">', 'dark', 'meowyString')
call s:At('result <uint32><error><null> : operation()', 'error', 'meowyTypeName')
call s:At('present : result<>!<error>!<null>', '<>', 'meowyTypeDelimiter')
call s:At('present : result<>!<error>!<null>', '!', 'meowyOperator')
call s:At('projected : result<uint32>', 'uint32', 'meowyTypeName')
call s:At('        field <collections.Vector<uint8>>', 'field', 'meowyIdentifier')
call s:At('after_record : 42', '42', 'meowyNumber')
call s:At('    collections.Vector<uint8>', 'collections', 'meowyTypeName')
call s:At('>> : nested', '>>', 'meowyTypeDelimiter')
call s:At('>> : nested', '>>', 'meowyTypeDelimiter', 1)
call s:At('after_generic : 43', 'after_generic', 'meowyBinding')

call s:At('| age < 18 && age >= 0 | debug.print("minor")', '<', 'meowyOperator')
call s:At('| age < 18 && age >= 0 | debug.print("minor")', '&&', 'meowyOperator')
call s:At('| count <limit && other > 0 | debug.print("comparison")', 'limit', 'meowyIdentifier')
call s:At('| count <limit && other > 0 | debug.print("comparison")', '>', 'meowyOperator')
call s:At('| result <error> || result <null> | ''scope.leave()', '||', 'meowyOperator')
call s:At('mask : (left | right) & 0xff', '|', 'meowyOperator')
call s:At('    finish : ''scope.leave', '''scope', 'meowyScope')
call s:At('    finish : ''scope.leave', 'leave', 'meowyIdentifier')
call s:At('    ''scope.restart()', 'restart', 'meowyCall')

call s:At('&jobs<tasks.Outcome<uint32>[4]>', '&jobs', 'meowyTaskGroup')
call s:At('&jobs >> work(1)', '&jobs', 'meowyTaskGroup')
call s:At('&jobs >> work(1)', '>>', 'meowyTaskOperator')
call s:At('job : >> work(2)', '>>', 'meowyTaskOperator')
call s:At('result : << job', '<<', 'meowyTaskOperator')
call s:At('results : << &jobs', '<<', 'meowyTaskOperator')
call s:At('    .{ -> self }', '.', 'meowyDispatch')
call s:At('    .{ -> self }', 'self', 'meowyIdentifier')

call s:At('plain : "a # is string content"', '#', 'meowyString')
let s:escaped = 'escaped : "quote: \" slash: \\ braces: \{literal\} tab: \t nul: \0"'
for s:escape in ['\"', '\\', '\{', '\}', '\t', '\0']
  call s:At(s:escaped, s:escape, 'meowyEscape')
endfor
call s:At('invalid : "unknown: \q"', '\q', 'meowyInvalidEscape')
call s:At('interpolated : "value: {value + 1} tail"', '{', 'meowyInterpolationDelimiter')
call s:At('interpolated : "value: {value + 1} tail"', '+', 'meowyOperator')
call s:At('interpolated : "value: {value + 1} tail"', 'tail', 'meowyString')
call s:At('nested_string : "outer {format("inner {value}")} tail"', 'inner', 'meowyString')
call s:At('nested_string : "outer {format("inner {value}")} tail"', 'value', 'meowyIdentifier')
call s:At('nested_string : "outer {format("inner {value}")} tail"', 'tail', 'meowyString')
call s:At('quoted_brace : "outer {format("}")} tail"', 'tail', 'meowyString')
call s:At('nested_block : "outer {value.{ -> { -> "inner" } }} tail"', 'inner', 'meowyString')
call s:At('nested_block : "outer {value.{ -> { -> "inner" } }} tail"', 'tail', 'meowyString')
call s:At('escaped_interpolation : "{format("\"quoted\"")} tail"', 'tail', 'meowyString')
call s:At('second {value}', 'second', 'meowyString')
call s:At('second {value}', 'value', 'meowyIdentifier')
call s:At('after_string : 44', 'after_string', 'meowyBinding')
call s:At('TODO >> <{ nested { [ ( and another "', 'TODO', 'meowyTodo')
call s:At('TODO >> <{ nested { [ ( and another "', '>>', 'meowyComment')
call s:At('after_comment : 45', 'after_comment', 'meowyBinding')

" Explicit opt-out for users who do not want lexical emphasis on prelude names.
let g:meowy_highlight_builtin_values = 0
syntax clear
unlet! b:current_syntax
runtime syntax/meowy.vim
call s:At('enabled : true', 'true', 'meowyIdentifier')
unlet g:meowy_highlight_builtin_values

" Reloads must not duplicate autocmds or append settings repeatedly.
let s:undo = b:undo_ftplugin
let s:suffixes = &l:suffixesadd
runtime ftplugin/meowy.vim
call assert_equal(s:undo, b:undo_ftplugin)
call assert_equal(s:suffixes, &l:suffixesadd)
runtime ftdetect/meowy.vim
runtime ftdetect/meowy.vim
let s:detectors = execute('autocmd meowy_filetype')
call assert_equal(2, len(split(s:detectors, 'setfiletype meowy', 1)) - 1)

" Buffer-local settings are reversible, and detection respects an existing type.
execute b:undo_ftplugin
for s:option in ['expandtab', 'shiftwidth', 'softtabstop', 'autoindent', 'cindent', 'smartindent', 'commentstring', 'comments', 'formatoptions', 'suffixesadd']
  call assert_equal(eval('&g:' . s:option), eval('&l:' . s:option), s:option . ' restored')
endfor
augroup meowy_test_override
  autocmd!
  autocmd BufRead *.mwy setfiletype text
augroup END
" Re-register our detector after the user's explicit detection in this event.
runtime ftdetect/meowy.vim
doautocmd BufRead fixture.mwy
call assert_equal('text', &l:filetype)
augroup meowy_test_override
  autocmd!
augroup END

enew
execute 'file ' . fnameescape(tempname() . '.mwy')
doautocmd meowy_filetype BufNewFile
call assert_equal('meowy', &l:filetype)

" A private autocmd group must still respect :filetype off.
filetype off
enew
setlocal filetype=
doautocmd meowy_filetype BufNewFile disabled.mwy
call assert_equal('', &l:filetype)
filetype plugin on

" Exercise the actual documented source, not just the focused lexer fixture.
for s:file in glob(s:root . '/docs/programs/*.mwy', 0, 1) + [s:root . '/mod.sample.mwy']
  execute 'edit ' . fnameescape(s:file)
  call assert_equal('meowy', &l:filetype, s:file)
  call assert_equal('meowy', b:current_syntax, s:file)
  for s:lnum in range(1, line('$'))
    call synID(s:lnum, max([1, strlen(getline(s:lnum))]), 1)
  endfor
endfor

call assert_equal('', v:errmsg, 'No editor errors')
if !empty(v:errors)
  for s:error in v:errors
    echomsg s:error
  endfor
  cquit
endif
echomsg 'meowy runtime checks passed'
qa!
