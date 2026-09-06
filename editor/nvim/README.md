# Vim and Neovim support

This directory is a standalone runtime bundle for `.mwy` files. It works in both
Vim and Neovim without an external plugin dependency.

- [File detection](ftdetect/meowy.vim) handles new and existing files, replaces
  generic fallback guesses, and respects an earlier explicit filetype detection.
- [Syntax highlighting](syntax/meowy.vim) follows the
  [punctuation-based language reference](../../docs/reference/syntax.md).
- [Buffer settings](ftplugin/meowy.vim) use four-space indentation, expand tabs,
  provide `# %s #` for comment commands, and add `.mwy` to filename suffix lookup.

## Setup

Add this directory to the runtime path before enabling filetype plugins.

For Neovim, in `init.lua`:

```lua
vim.opt.runtimepath:prepend("/absolute/path/to/meowy/editor/nvim")
vim.cmd("filetype plugin on")
vim.cmd("syntax enable")
```

For Vim, or Neovim with an `init.vim`:

```vim
execute 'set runtimepath^=' . fnameescape('/absolute/path/to/meowy/editor/nvim')
filetype plugin on
syntax enable
```

Replace the path with the checkout location. Reopen a `.mwy` file after changing
setup; `:setlocal filetype?` should report `meowy`. Only buffer-local editing
options change, and the filetype plugin provides an undo command when switching
filetypes. It installs no key mappings.

The defaults retain the current indentation on a new line. They disable C-style
smart indentation and automatic comment-leader insertion: a second `#` closes a
meowy comment. Automatic hard wrapping of source and literal strings is disabled.

## Highlighted syntax

The highlighter handles:

- Nested generic types, record types, function signatures, constrained binders,
  union alternatives, type subtraction, and type queries.
- References and pointers, including `<&!T>`, `<*!T>`, and `&!value`, plus `!{ ... }`
  safety boundaries.
- Task submission and joins (`>>`, `<<`), group declarations, labeled scopes,
  emissions, matchers, field selection, and dispatch.
- Decimal, hexadecimal, and binary integers with separators, floating-point
  literals and exponents, and arithmetic and logical operators.
- Delimited multiline comments, multiline strings, supported escapes, and
  interpolation containing nested expressions, blocks, or quoted strings.
- Literal module imports and ordinary calls, including generic calls.

Type regions nest, so `>>` inside a generic type closes two type arguments while
`>>` in an expression denotes task submission. A function type's `->` does not
close its angle brackets. A comparison such as `count < limit` does not start a
multiline type region. Unrecognized string escapes are highlighted as errors.

Blocks, record types, and multiline comments also expose syntax folds. Enable
those explicitly with `:setlocal foldmethod=syntax` if desired.

## Values are not keywords

Names such as `self`, `leave`, `restart`, `unsafe`, and `where` receive ordinary
identifier or call highlighting. Type names receive type highlighting inside
explicit type syntax, rather than from a reserved list of type words.

Reads spelled `true`, `false`, or `null` receive optional constant highlighting;
plain binding declarations such as `true : "local name"` remain identifiers.
This is lexical emphasis: later uses of a shadowed name cannot be distinguished
without resolving bindings. To disable that emphasis, set this before loading
syntax:

```lua
vim.g.meowy_highlight_builtin_values = false
```

Or in Vimscript:

```vim
let g:meowy_highlight_builtin_values = 0
```

The highlighter does not type-check expressions, resolve aliases, or infer whether
an arbitrary `&name` refers to a group or a borrowed value. Incomplete or ambiguous
annotations may stay plain until enough punctuation has been entered. The
[language reference](../../docs/reference/syntax.md) defines their meaning.

## Checks

From the repository root, run either command:

```sh
nvim --headless -u NONE -i NONE -n -S editor/nvim/tests/run.vim
vim -Nu NONE -i NONE -n -es -S editor/nvim/tests/run.vim
```

The [regression script](tests/run.vim) checks actual syntax groups against a
[focused fixture](tests/fixtures/syntax.mwy), file detection, buffer settings,
reloading, option cleanup, and loading each documented program and the sample
manifest. It exits unsuccessfully on failed assertions or editor errors.

The bundle uses the standard [syntax runtime](https://neovim.io/doc/user/syntax/)
and [filetype runtime](https://neovim.io/doc/user/filetype/) conventions.
