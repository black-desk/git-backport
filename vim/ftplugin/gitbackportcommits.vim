" SPDX-FileCopyrightText: 2025 Chen Linxuan <me@black-desk.cn>
"
" SPDX-License-Identifier: MIT

" Git Backport Commits filetype plugin
" Maintainer: Chen Linxuan <me@black-desk.cn>
" Last Change: 2025 Sep 04

if exists("b:did_ftplugin")
  finish
endif
let b:did_ftplugin = 1

" If vim-fugitive is available, enable jump mappings
if exists(':Git')
  " Function to show Git commit under cursor
  function! s:GitShowUnderCursor()
    let word = expand('<cword>')
    if word != ''
      execute 'Git show ' . word
    endif
  endfunction

  " Map <CR> in normal mode to show Git commit under cursor
  nnoremap <buffer> <CR> :call <SID>GitShowUnderCursor()<CR>
endif
