# Neovim syntax support

The [file detector](ftdetect/meowy.vim) recognizes `.mwy` files. The
[syntax file](syntax/meowy.vim) highlights core literals, comments, scopes,
emissions, dispatch, and common type forms.

Add this directory to Neovim's runtime path in `init.lua`:

```lua
vim.opt.runtimepath:append("/absolute/path/to/meowy/editor/nvim")
vim.cmd("filetype on")
vim.cmd("syntax enable")
```

Replace the path with your checkout location. This is basic lexical highlighting;
nested generic types and contextual punctuation can require manual reading. The
[language reference](../../docs/reference/syntax.md) defines syntax and semantics.
