# ASS LSP Server

A Language Server Protocol (LSP) implementation for Advanced SubStation Alpha (ASS/SSA) subtitle format.

## Overview

This LSP server provides rich language support for ASS/SSA subtitle files, including syntax highlighting, code completion, hover information, and validation. It's designed to work with any LSP-compatible editor like VS Code, Neovim, Emacs, and others.

## Features

- **Syntax Validation**: Real-time validation of ASS/SSA syntax and structure
- **Code Completion**: Intelligent autocompletion for:
  - Section headers (`[Script Info]`, `[V4+ Styles]`, `[Events]`, etc.)
  - Override tags (`\pos`, `\move`, `\c`, `\t`, etc.)
  - Script info properties
  - Style and event format fields
- **Hover Information**: Detailed documentation for override tags and sections
- **Advanced Analysis**: 
  - Style inheritance validation
  - Timing overlap detection
  - Performance suggestions
- **Error Reporting**: Clear diagnostics with line numbers and descriptions

## Installation

### From Source

```bash
git clone https://github.com/wiedymi/ass-lsp
cd ass-lsp
cargo build --release
```

The compiled binary will be available at `target/release/ass-lsp`.

### From Crates.io

```bash
cargo install ass-lsp
```

## Usage

### VS Code

1. Install the ASS LSP extension from the marketplace
2. The extension will automatically use the LSP server

### Neovim

Add this to your Neovim configuration:

```lua
local lspconfig = require('lspconfig')

lspconfig.ass_lsp = {
  default_config = {
    cmd = { 'ass-lsp' },
    filetypes = { 'ass', 'ssa' },
    root_dir = function(fname)
      return lspconfig.util.find_git_ancestor(fname) or vim.loop.os_homedir()
    end,
    settings = {},
  },
}

-- Setup the LSP
require('lspconfig').ass_lsp.setup{}
```

### Emacs

Add this to your Emacs configuration:

```elisp
(use-package lsp-mode
  :config
  (add-to-list 'lsp-language-id-configuration '(ass-mode . "ass"))
  (lsp-register-client
   (make-lsp-client
    :new-connection (lsp-stdio-connection "ass-lsp")
    :major-modes '(ass-mode)
    :server-id 'ass-lsp)))
```

### Generic LSP Client

For any LSP-compatible editor, configure it to:
- Run the command: `ass-lsp`
- Associate with file extensions: `.ass`, `.ssa`
- Use stdio for communication

## File Format Support

This LSP server supports:
- Advanced SubStation Alpha (ASS) format
- SubStation Alpha (SSA) format
- All standard sections: Script Info, Styles, Events, Fonts, Graphics
- Override tags and animations
- Style inheritance and references

## Development

### Prerequisites

- Rust 1.70+ 
- Cargo

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running in Development

```bash
cargo run
```

## Configuration

The LSP server currently runs with default settings. Future versions may support configuration files for customizing behavior.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Supported Override Tags

The LSP provides completion and documentation for all standard ASS override tags:

- **Position & Movement**: `\pos`, `\move`, `\org`, `\clip`
- **Fonts & Text**: `\fn`, `\fs`, `\fsp`, `\fscx`, `\fscy`
- **Colors**: `\c`, `\1c`, `\2c`, `\3c`, `\4c`, `\alpha`, `\1a`, `\2a`, `\3a`, `\4a`
- **Animation**: `\t`, `\fade`, `\fad`
- **Rotation**: `\frx`, `\fry`, `\frz`
- **Borders & Shadows**: `\bord`, `\shad`, `\be`, `\blur`
- **Layout**: `\an`, `\a`, `\q`, `\r`
- **Effects**: `\k`, `\kf`, `\ko`, `\kt`

## Troubleshooting

### LSP Server Not Starting

1. Ensure the `ass-lsp` binary is in your PATH
2. Check that your editor is configured to run the LSP for `.ass` and `.ssa` files
3. Look at your editor's LSP logs for error messages

### No Completions Showing

1. Make sure your file has the correct extension (`.ass` or `.ssa`)
2. Try triggering completion manually (usually Ctrl+Space)
3. Check that the LSP server is running and connected

### Performance Issues

For large subtitle files, you may experience slower response times. The LSP includes performance monitoring and will provide suggestions for optimization.
