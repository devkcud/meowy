" Vim/Neovim syntax for meowy. Names are values, not reserved keywords.
if exists('b:current_syntax')
  finish
endif

let s:save_cpo = &cpoptions
set cpoptions&vim
syntax case match

" Explicit clusters keep task/comparison operators out of type arguments while
" allowing full expressions, including nested strings, inside interpolation.
syntax cluster meowyCode contains=meowyIdentifier,meowyBuiltinValue,meowyBinding,meowyCall,meowyNumber,meowyFloat,meowyOperator,meowyEmit,meowyTaskOperator,meowyBorrow,meowyUnchecked,meowyDispatch,meowyScope,meowyTaskGroup,meowyPunctuation,meowyBlock,meowyParen,meowyList,meowyType,meowyString,meowyImport,meowyComment
syntax cluster meowyTypeBody contains=meowyTypeName,meowyTypeArguments,meowyTypeRecord,meowyTypeParameters,meowyTypeExtent,meowyTypeOperator,meowyTypePunctuation,meowyNumber,meowyString,meowyComment

syntax match meowyIdentifier /\<[A-Za-z_][A-Za-z0-9_]*\>/
" This is optional lexical emphasis, not a claim about name resolution.
if get(g:, 'meowy_highlight_builtin_values', 1)
  syntax match meowyBuiltinValue /\<\%(true\|false\|null\)\>/
endif
syntax match meowyBinding /\<[A-Za-z_][A-Za-z0-9_]*\>\ze\s*:\%(:\)\@!/
syntax match meowyCall /\<[A-Za-z_][A-Za-z0-9_]*\>\ze\s*(/
syntax match meowyCall /\<[A-Za-z_][A-Za-z0-9_]*\>\ze<[^"#{};|=]\+>\s*(/

syntax match meowyOperator /[-+*\/%=<>!~^&|:]/
syntax match meowyOperator /:=\|==\|!=\|<=\|>=\|&&\|||/
syntax match meowyEmit /->/
syntax match meowyTaskOperator />>\|<</
syntax match meowyBorrow /\%(&\)\@<!&\%(&\)\@!\%(!\)\?/
syntax match meowyUnchecked /!\ze\s*{/
syntax match meowyDispatch /\./
syntax match meowyScope /'[A-Za-z_][A-Za-z0-9_]*/
syntax match meowyTaskGroup /&[A-Za-z_][A-Za-z0-9_]*\ze\%(<\|\s*>>\)/
syntax match meowyPunctuation /[,;]/

syntax match meowyNumber /\<\d\%(_\?\d\)*\>/
syntax match meowyNumber /\<0[xX][0-9A-Fa-f]\%(_\?[0-9A-Fa-f]\)*\>/
syntax match meowyNumber /\<0[bB][01]\%(_\?[01]\)*\>/
syntax match meowyFloat /\<\d\%(_\?\d\)*\.\d\%(_\?\d\)*\%([eE][+-]\?\d\%(_\?\d\)*\)\?\>/
syntax match meowyFloat /\<\d\%(_\?\d\)*[eE][+-]\?\d\%(_\?\d\)*\>/

syntax region meowyBlock matchgroup=meowyDelimiter start=/{/ end=/}/ transparent fold contains=@meowyCode
syntax region meowyParen matchgroup=meowyDelimiter start=/(/ end=/)/ transparent contains=@meowyCode
syntax region meowyList matchgroup=meowyDelimiter start=/\[/ end=/\]/ transparent contains=@meowyCode

" A named outer type must reach a type delimiter. In particular, '< limit'
" and '<< task' must not start a region that consumes the rest of the file.
syntax region meowyType matchgroup=meowyTypeDelimiter start=/\%(<\)\@<!<\ze\%([:&*!({"]\|>\|[A-Za-z_][A-Za-z0-9_]*\%(\.[A-Za-z_][A-Za-z0-9_]*\)*\s*\%([><]\|\[\)\)/ end=/\%(-\)\@<!>/ contains=@meowyTypeBody
" Once inside a type, angle brackets nest; adjacent '>>' close two levels.
" The '>' in a function arrow never closes either region.
syntax region meowyTypeArguments matchgroup=meowyTypeDelimiter start=/</ end=/\%(-\)\@<!>/ contained contains=@meowyTypeBody
syntax region meowyTypeRecord matchgroup=meowyTypeDelimiter start=/{/ end=/}/ contained transparent fold contains=@meowyCode
syntax region meowyTypeParameters matchgroup=meowyTypeDelimiter start=/(/ end=/)/ contained transparent contains=@meowyTypeBody
syntax region meowyTypeExtent matchgroup=meowyTypeDelimiter start=/\[/ end=/\]/ contained contains=meowyNumber,meowyTypeName
syntax match meowyTypeName /\<[A-Za-z_][A-Za-z0-9_]*\>/ contained
syntax match meowyTypeOperator /[&*!:]\|->/ contained
syntax match meowyTypePunctuation /[,.]/ contained

syntax region meowyComment start=/#/ end=/#/ fold contains=meowyTodo,@Spell
syntax match meowyTodo /\<\%(TODO\|FIXME\|NOTE\|XXX\)\>/ contained
syntax region meowyString start=/"/ skip=/\\./ end=/"/ contains=meowyInvalidEscape,meowyEscape,meowyInterpolation,@Spell
syntax match meowyInvalidEscape /\\./ contained
syntax match meowyEscape /\\[nrt0"\\{}]/ contained
" No keepend: a nested block or quoted string owns its own closing delimiter.
syntax region meowyInterpolation matchgroup=meowyInterpolationDelimiter start=/{/ end=/}/ contained contains=@meowyCode
syntax region meowyImport start=/@"/ skip=/\\./ end=/"/ contains=meowyInvalidEscape,meowyEscape

" Delimited comments, strings, and types can all span lines.
syntax sync fromstart

highlight default link meowyIdentifier Identifier
highlight default link meowyBuiltinValue Constant
highlight default link meowyBinding Identifier
highlight default link meowyCall Function
highlight default link meowyNumber Number
highlight default link meowyFloat Float
highlight default link meowyOperator Operator
highlight default link meowyEmit Statement
highlight default link meowyTaskOperator Special
highlight default link meowyBorrow Operator
highlight default link meowyUnchecked Special
highlight default link meowyDispatch Operator
highlight default link meowyScope Label
highlight default link meowyTaskGroup Label
highlight default link meowyPunctuation Delimiter
highlight default link meowyDelimiter Delimiter
highlight default link meowyTypeDelimiter Delimiter
highlight default link meowyTypeName Type
highlight default link meowyTypeOperator Operator
highlight default link meowyTypePunctuation Delimiter
highlight default link meowyComment Comment
highlight default link meowyTodo Todo
highlight default link meowyString String
highlight default link meowyEscape SpecialChar
highlight default link meowyInvalidEscape Error
highlight default link meowyInterpolationDelimiter Special
highlight default link meowyImport Include

let b:current_syntax = 'meowy'
let &cpoptions = s:save_cpo
unlet s:save_cpo
