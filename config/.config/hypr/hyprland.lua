-- Simple Hyprland + Noctalia configuration
-- Fedora KDE base | 4 workspaces | French AZERTY

local terminal     = "konsole"
local file_manager = "dolphin"
local browser      = "chromium"

-- ── Monitor ──────────────────────────────────
hl.monitor({ name = "", resolution = "preferred", position = "auto", scale = 1 })

-- ── Config ───────────────────────────────────
hl.config({
    input = {
        kb_layout = "fr", kb_variant = "azerty",
        numlock_by_default = true, follow_mouse = 1, sensitivity = 0,
        touchpad = { natural_scroll = false },
    },
    general = {
        gaps_in = 5, gaps_out = 8, border_size = 3,
        ["col.active_border"]   = "rgb(45aed1)",
        ["col.inactive_border"] = "rgba(eeeeee00)",
        layout = "dwindle", allow_tearing = false,
    },
    decoration = {
        rounding = 0,
        blur   = { enabled = true, size = 3, passes = 1 },
        shadow = { enabled = false },
    },
    animations = {
        enabled = true,
        bezier  = { "myBezier, 0.05, 0.9, 0.1, 1.05" },
        animation = {
            "windows, 1, 4, myBezier",
            "windowsOut, 1, 3, default, popin 80%",
            "fade, 1, 7, default",
            "workspaces, 1, 3, default",
        },
    },
    dwindle  = { pseudotile = true, preserve_split = true },
    master   = { new_is_master = true },
    gestures = { workspace_swipe = true },
    misc     = { force_default_wallpaper = 0 },
})

-- ── Noctalia ─────────────────────────────────
hl.on("hyprland.start", function() hl.exec_cmd("noctalia") end)

-- ── Helper ───────────────────────────────────
local function bind(keys, disp, desc) hl.bind(keys, disp, { description = desc }) end

-- ── Apps ─────────────────────────────────────
bind("SUPER + RETURN", hl.dsp.exec_cmd(terminal),     "Terminal")
bind("SUPER + E",      hl.dsp.exec_cmd(file_manager), "File manager")
bind("SUPER + B",      hl.dsp.exec_cmd(browser),      "Browser")
bind("SUPER + P",      hl.dsp.exec_cmd("noctalia msg panel-toggle launcher"), "Launcher")

-- ── Windows ──────────────────────────────────
bind("SUPER + Q",     hl.dsp.window.close(), "Close window")
bind("SUPER + SPACE", hl.dsp.window.float({ action = "toggle" }), "Toggle floating")
bind("SUPER + M",     hl.dsp.window.fullscreen({ mode = "fullscreen", action = "toggle" }), "Fullscreen")

-- ── Focus / Move ─────────────────────────────
for _, d in ipairs({ "left", "right", "up", "down" }) do
    bind("SUPER + " .. d:upper(),         hl.dsp.focus({ direction = d }),       "Focus " .. d)
    bind("SUPER + SHIFT + " .. d:upper(), hl.dsp.window.move({ direction = d }), "Move window " .. d)
end

-- ── Workspaces (digits + AZERTY symbols) ─────
local ws_keys = { "1", "2", "3", "4" }
local az_keys = { "AMPERSAND", "EACUTE", "DOUBLE_QUOTE", "APOSTROPHE" }

for i = 1, 4 do
    for _, k in ipairs({ ws_keys[i], az_keys[i] }) do
        bind("SUPER + " .. k,         hl.dsp.focus({ workspace = i }),    "Workspace " .. i)
        bind("SUPER + SHIFT + " .. k, hl.dsp.window.move_to_workspace(i), "Move window to workspace " .. i)
    end
end

-- ── Layout ───────────────────────────────────
bind("SUPER + T", hl.dsp.toggle_split(), "Toggle split")

-- ── Mouse ────────────────────────────────────
hl.bind_mouse("SUPER", "BTN_LEFT",  hl.dsp.window.move())
hl.bind_mouse("SUPER", "BTN_RIGHT", hl.dsp.window.resize())
