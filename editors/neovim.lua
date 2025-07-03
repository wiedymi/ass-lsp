-- ASS LSP Server Configuration for Neovim
-- Add this to your Neovim configuration (init.lua or a separate file)

local M = {}

-- Function to setup ASS LSP
function M.setup()
  -- Ensure lspconfig is available
  local ok, lspconfig = pcall(require, 'lspconfig')
  if not ok then
    vim.notify('lspconfig not found. Please install nvim-lspconfig', vim.log.levels.ERROR)
    return
  end

  -- Register the ASS LSP server configuration
  local configs = require('lspconfig.configs')

  -- Only register if not already registered
  if not configs.ass_lsp then
    configs.ass_lsp = {
      default_config = {
        cmd = { 'ass-lsp' },
        filetypes = { 'ass', 'ssa' },
        root_dir = function(fname)
          return lspconfig.util.find_git_ancestor(fname)
            or lspconfig.util.path.dirname(fname)
        end,
        single_file_support = true,
        settings = {
          ['ass-lsp'] = {
            enable = true,
            -- Add any specific settings here
          }
        },
        init_options = {},
        capabilities = vim.lsp.protocol.make_client_capabilities(),
      },
      docs = {
        description = [[
ASS LSP Server - Language Server Protocol implementation for Advanced SubStation Alpha (ASS/SSA) subtitle format.

Installation:
```bash
cargo install ass-lsp
```

For more information, visit: https://github.com/wiedymi/ass-lsp
        ]],
        default_config = {
          root_dir = [[util.find_git_ancestor(fname) or util.path.dirname(fname)]],
        },
      },
    }
  end

  -- Setup the LSP with enhanced capabilities
  lspconfig.ass_lsp.setup({
    on_attach = M.on_attach,
    capabilities = M.get_capabilities(),
    settings = {
      ['ass-lsp'] = {
        enable = true,
      }
    },
  })

  -- Set up file type detection
  vim.filetype.add({
    extension = {
      ass = 'ass',
      ssa = 'ass', -- Treat SSA files as ASS for LSP purposes
    },
  })

  -- Set up syntax highlighting (basic)
  vim.api.nvim_create_autocmd('FileType', {
    pattern = 'ass',
    callback = function()
      vim.bo.commentstring = '!%s'
      vim.bo.comments = '!,:'
    end,
  })
end

-- LSP on_attach function with keybindings
function M.on_attach(client, bufnr)
  -- Enable completion triggered by <c-x><c-o>
  vim.bo[bufnr].omnifunc = 'v:lua.vim.lsp.omnifunc'

  -- Buffer local mappings
  local bufopts = { noremap = true, silent = true, buffer = bufnr }

  -- Navigation
  vim.keymap.set('n', 'gD', vim.lsp.buf.declaration, bufopts)
  vim.keymap.set('n', 'gd', vim.lsp.buf.definition, bufopts)
  vim.keymap.set('n', 'K', vim.lsp.buf.hover, bufopts)
  vim.keymap.set('n', 'gi', vim.lsp.buf.implementation, bufopts)
  vim.keymap.set('n', '<C-k>', vim.lsp.buf.signature_help, bufopts)

  -- Workspace
  vim.keymap.set('n', '<space>wa', vim.lsp.buf.add_workspace_folder, bufopts)
  vim.keymap.set('n', '<space>wr', vim.lsp.buf.remove_workspace_folder, bufopts)
  vim.keymap.set('n', '<space>wl', function()
    print(vim.inspect(vim.lsp.buf.list_workspace_folders()))
  end, bufopts)

  -- Code actions
  vim.keymap.set('n', '<space>D', vim.lsp.buf.type_definition, bufopts)
  vim.keymap.set('n', '<space>rn', vim.lsp.buf.rename, bufopts)
  vim.keymap.set({ 'n', 'v' }, '<space>ca', vim.lsp.buf.code_action, bufopts)
  vim.keymap.set('n', 'gr', vim.lsp.buf.references, bufopts)

  -- Formatting
  vim.keymap.set('n', '<space>f', function()
    vim.lsp.buf.format { async = true }
  end, bufopts)

  -- Diagnostics
  vim.keymap.set('n', '<space>e', vim.diagnostic.open_float, bufopts)
  vim.keymap.set('n', '[d', vim.diagnostic.goto_prev, bufopts)
  vim.keymap.set('n', ']d', vim.diagnostic.goto_next, bufopts)
  vim.keymap.set('n', '<space>q', vim.diagnostic.setloclist, bufopts)

  -- ASS-specific keybindings
  vim.keymap.set('n', '<space>ap', function()
    vim.notify('ASS LSP: Analyzing performance...', vim.log.levels.INFO)
  end, bufopts)

  -- Auto-completion setup
  if client.server_capabilities.completionProvider then
    vim.bo[bufnr].omnifunc = 'v:lua.vim.lsp.omnifunc'
  end

  -- Document highlighting
  if client.server_capabilities.documentHighlightProvider then
    vim.api.nvim_create_augroup('lsp_document_highlight', {})
    vim.api.nvim_create_autocmd({ 'CursorHold', 'CursorHoldI' }, {
      group = 'lsp_document_highlight',
      buffer = bufnr,
      callback = vim.lsp.buf.document_highlight,
    })
    vim.api.nvim_create_autocmd('CursorMoved', {
      group = 'lsp_document_highlight',
      buffer = bufnr,
      callback = vim.lsp.buf.clear_references,
    })
  end
end

-- Enhanced capabilities with completion support
function M.get_capabilities()
  local capabilities = vim.lsp.protocol.make_client_capabilities()

  -- Add completion capabilities if nvim-cmp is available
  local ok, cmp_lsp = pcall(require, 'cmp_nvim_lsp')
  if ok then
    capabilities = cmp_lsp.default_capabilities(capabilities)
  end

  return capabilities
end

-- Function to check if ASS LSP is available
function M.check_lsp()
  local handle = io.popen('which ass-lsp')
  if handle then
    local result = handle:read('*a')
    handle:close()
    if result and result ~= '' then
      vim.notify('ASS LSP found at: ' .. result:gsub('\n', ''), vim.log.levels.INFO)
      return true
    end
  end

  vim.notify('ASS LSP not found in PATH. Please install it with: cargo install ass-lsp', vim.log.levels.WARN)
  return false
end

-- Auto-setup function
function M.auto_setup()
  if M.check_lsp() then
    M.setup()
    vim.notify('ASS LSP configured successfully', vim.log.levels.INFO)
  end
end

-- User commands
vim.api.nvim_create_user_command('AssLspSetup', M.setup, {
  desc = 'Setup ASS LSP server'
})

vim.api.nvim_create_user_command('AssLspCheck', M.check_lsp, {
  desc = 'Check if ASS LSP server is available'
})

vim.api.nvim_create_user_command('AssLspRestart', function()
  vim.cmd('LspRestart ass_lsp')
end, {
  desc = 'Restart ASS LSP server'
})

-- Example usage in init.lua:
--
-- -- Method 1: Auto-setup (recommended)
-- require('path.to.this.file').auto_setup()
--
-- -- Method 2: Manual setup
-- require('path.to.this.file').setup()
--
-- -- Method 3: Lazy loading with lazy.nvim
-- {
--   'neovim/nvim-lspconfig',
--   config = function()
--     require('path.to.this.file').setup()
--   end,
-- }

return M
