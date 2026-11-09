--------------------------------------------------------------------------------
-- File: vnchkey.lua
-- Description: Official vnchKey (Tự Do Edition) Integration Module for Neovim.
--
-- Features:
--   1. Zero-latency async IME switching via vim.loop.spawn / vim.uv.spawn.
--   2. Prevents Normal-mode navigation keys (h, j, k, l, u, t, d, y, etc.)
--      from leaking into vnchKey's keystroke buffer and corrupting insert mode.
--   3. Ultra-fast escape with 'jk' or physical <Esc> without input lag.
--   4. One-time Load Guard: prevents duplicate loading and displays a friendly
--      warning if the user attempts to load it again.
--------------------------------------------------------------------------------

local M = {}

-- Guard: One-time load flag across the entire Neovim session
if _G._vnchkey_loaded then
  function M.setup()
    vim.notify("⚠️ [vnchKey] Module đã được nạp (loaded) trước đó rồi! Không cần nạp lại.", vim.log.levels.WARN)
  end
  function M.load()
    M.setup()
  end
  return M
end

-- Resolve fcitx-remote binary once
local fcitx_cmd = nil
if vim.fn.executable("fcitx5-remote") == 1 then
  fcitx_cmd = "fcitx5-remote"
elseif vim.fn.executable("fcitx-remote") == 1 then
  fcitx_cmd = "fcitx-remote"
end

-- Fallback for spawn across Neovim versions
local spawn = (vim.uv or vim.loop).spawn

-- Function: fcitx2en (Switch to English / Latin input when in Normal mode)
function M.fcitx2en()
  if not fcitx_cmd then return end
  spawn(fcitx_cmd, { args = { "-c" } }, nil)
  vim.b.input_toggle_flag = true
end

-- Function: fcitx2vi (Restore Vietnamese input when entering Insert mode)
function M.fcitx2vi()
  if not fcitx_cmd then return end
  if vim.b.input_toggle_flag == true then
    spawn(fcitx_cmd, { args = { "-o" } }, nil)
    vim.b.input_toggle_flag = false
  end
end

-- Function: fast_escape (Instant escape to Normal mode with IME reset)
function M.fast_escape()
  M.fcitx2en()
  local esc = vim.api.nvim_replace_termcodes("<Esc>", true, false, true)
  vim.api.nvim_feedkeys(esc, "n", false)
  vim.cmd("redraw")
end

-- Function: setup / load
function M.setup()
  if _G._vnchkey_loaded then
    vim.notify("⚠️ [vnchKey] Module đã được nạp (loaded) trước đó rồi! Không cần nạp lại.", vim.log.levels.WARN)
    return
  end

  if not fcitx_cmd then
    vim.notify("❌ [vnchKey] Không tìm thấy fcitx5-remote hoặc fcitx-remote trên hệ thống!", vim.log.levels.ERROR)
    return
  end

  -- Mark as loaded globally
  _G._vnchkey_loaded = true

  -- Key mappings for ultra-responsive exit
  vim.keymap.set("i", "jk", M.fast_escape, {
    noremap = true,
    silent = true,
    desc = "vnchKey: Fast Escape & Reset IME",
  })

  vim.keymap.set("i", "<Esc>", function()
    M.fcitx2en()
    return "<Esc>"
  end, {
    expr = true,
    silent = true,
    desc = "vnchKey: Escape & Reset IME",
  })

  -- Autocommands for seamless Insert / Normal mode toggling
  local augroup = vim.api.nvim_create_augroup("vnchkey_neovim_integration", { clear = true })

  -- Leaving Insert mode: immediately switch to English to protect Normal-mode keystrokes
  vim.api.nvim_create_autocmd("InsertLeave", {
    group = augroup,
    callback = function()
      M.fcitx2en()
      vim.cmd("redraw")
    end,
  })

  -- Entering Insert mode: restore Vietnamese input
  vim.api.nvim_create_autocmd("InsertEnter", {
    group = augroup,
    callback = M.fcitx2vi,
  })

  -- Command-line mode setup (/ ? :)
  vim.api.nvim_create_autocmd("CmdlineEnter", {
    group = augroup,
    pattern = { "/", "?", ":" },
    callback = M.fcitx2vi,
  })

  vim.api.nvim_create_autocmd("CmdlineLeave", {
    group = augroup,
    pattern = { "/", "?", ":" },
    callback = M.fcitx2en,
  })

  -- If currently in Normal mode when loaded, ensure IME is turned off to prevent keystroke leakage
  if vim.fn.mode() == "n" then
    M.fcitx2en()
  end

  vim.notify("🟡 [vnchKey] Đã nạp thành công bộ tích hợp Neovim! (Zero-latency modal sync active)", vim.log.levels.INFO)
end

-- Alias load() to setup() for intuitive user calling
M.load = M.setup

return M

