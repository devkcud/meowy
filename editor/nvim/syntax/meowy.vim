if exists('b:current_syntax')
  finish
endif

syn region meowyComment start=/#/ end=/#/ contains=@Spell
syn region meowyString start=/"/ skip=/\\./ end=/"/ contains=meowyInterp
syn region meowyInterp start=/{/ end=/}/ contained

syn match meowyNumber /\<\d\+\%(\.\d\+\)\?\>/
syn keyword meowyBoolean true false
syn keyword meowyNull null
syn keyword meowySelf self
syn keyword meowyScopeCtl leave restart

syn match meowyCall /\<\w\+\ze\s*(/
syn match meowyScope /'\w\+/
syn match meowyScope /&\w\+/
syn match meowyImport /@\ze"/

syn match meowyOperator /[:|]\|:=\|[+*\/%!=<>-]=\?/
syn match meowyOperator /\[\d\+\]/
syn match meowyEmit /->/
syn match meowyDispatch /\.\ze\s*[{'|]/

syn match meowyType /<\(\:\)\?[A-Za-z_][A-Za-z0-9_]*\(\.[A-Za-z_][A-Za-z0-9_]*\)*\(\[\d*\]\)\?>/
syn region meowyTypeBlock matchgroup=meowyTypeBracket start=/<{/ end=/}>/ contains=TOP

syn sync fromstart

hi def link meowyComment Comment
hi def link meowyString String
hi def link meowyInterp Special
hi def link meowyNumber Number
hi def link meowyBoolean Boolean
hi def link meowyNull Constant
hi def link meowySelf Identifier
hi def link meowyScopeCtl Statement
hi def link meowyCall Function
hi def link meowyScope Label
hi def link meowyImport Include
hi def link meowyOperator Operator
hi def link meowyEmit Statement
hi def link meowyDispatch Operator
hi def link meowyType Type
hi def link meowyTypeBracket Type

let b:current_syntax = 'meowy'
