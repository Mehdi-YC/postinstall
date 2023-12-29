from libqtile import widget
from .theme import colors


# Define Widgets
widget_defaults = dict(
    font="JetBrainsMono Nerd Font",
    fontsize = 12,
    padding = 2,
    background=bg
)

def init_widgets_list(monitor_num):
    widgets_list = [
        widget.GroupBox(
            font="JetBrainsMono Nerd Font",
            fontsize = 16,
            margin_y = 2,
            margin_x = 4,
            padding_y = 6,
            padding_x = 6,
            borderwidth = 2,
            disable_drag = True,
            active = color2,
            inactive = fg,
            hide_unused = False,
            rounded = False,
            highlight_method = "line",
            highlight_color = [bg, bg],
            this_current_screen_border = color2,
            this_screen_border = color4,
            other_screen_border = color3,
            other_current_screen_border = color3,
            urgent_alert_method = "line",
            urgent_border = color1,
            urgent_text = color1,
            foreground = fg,
            background = bg,
            use_mouse_wheel = False
        ),
        widget.TaskList(
            icon_size = 0,
            font = "JetBrainsMono Nerd Font",
            foreground = fg,
            background = bg,
            borderwidth = 1,
            border = color2,
            margin = 0,
            padding = 10,
            highlight_method = "block",
            title_width_method = "uniform",
            urgent_alert_method = "border",
            urgent_border = color1,
            rounded = False,
            txt_floating = "🗗 ",
            txt_maximized = "🗖 ",
            txt_minimized = "🗕 ",
        ),
        widget.Sep(
            linewidth = 1,
            padding = 10,
            foreground = color2,
            background = bg
        ),
        widget.OpenWeather(
            app_key = "4cf3731a25d1d1f4e4a00207afd451a2",
            cityid = "4997193",
            format = '{icon} {main_temp}°',
            metric = False,
            font = "JetBrainsMono Nerd Font",
            foreground = fg,
        ),
#        widget.GenPollText(
#            name = "weather",
#            fmt = " {} ", update_interval = 3600,
#            foreground = fg,
#            func = lambda: subprocess.check_output("/usr/local/bin/weather.py").decode("utf-8"),
#            margin = 10 
#            ),
#        widget.Sep(
#            linewidth = 0,
#            padding = 10
#        ),
       widget.Sep(
            linewidth = 0,
            padding = 10
        ),
        widget.TextBox(
            text = "ﮮ",
            fontsize = 14,
            font = "JetBrainsMono Nerd Font",
            foreground = color6,
        ),
        widget.CheckUpdates(
            distro = 'Fedora',
            display_format = '{updates} updates',
            font = "JetBrainsMono Nerd Font",
            no_update_string = 'No Updates',
            update_interval = 3600
        ),
        widget.Sep(
            linewidth = 0,
            padding = 10
        ),
        widget.TextBox(
            text = "",
            fontsize = 14,
            font = "JetBrainsMono Nerd Font",
            foreground = color1,
        ),
        widget.CPU(
            font = "JetBrainsMono Nerd Font",
            update_interval = 1.0,
            format = '{load_percent}%',
            foreground = fg,
            padding = 5
        ),
        widget.Sep(
            linewidth = 0,
            padding = 10
        ),
        widget.TextBox(
            text = "",
            fontsize = 14,
            font = "JetBrainsMono Nerd Font",
            foreground = color3,
        ),
        widget.Memory(
            font = "JetBrainsMonoNerdFont",
            foreground = fg,
            format = '{MemUsed: .0f}{mm} /{MemTotal: .0f}{mm}',
            measure_mem='G',
            padding = 5,
        ),
        widget.Sep(
            linewidth = 0,
            padding = 10
        ),
        widget.TextBox(
            text = "",
            fontsize = 14,
            font = "JetBrainsMono Nerd Font",
            foreground = color5,
        ),
        widget.Clock(
            format='%I:%M %p',
            font = "JetBrainsMono Nerd Font",
            padding = 10,
            foreground = fg
        ),
        widget.Systray(
            background = bg,
            icon_size = 20,
            padding = 4,
        ),
        widget.Sep(
            linewidth = 1,
            padding = 10,
            foreground = color2,
            background = bg
        ),
        widget.CurrentLayoutIcon(
            scale = 0.5,
            foreground = fg,
            background = bg
        ),
    ]

    return widgets_list

def init_secondary_widgets_list(monitor_num):
    secondary_widgets_list = init_widgets_list(monitor_num)
    del secondary_widgets_list[16:17]
    return secondary_widgets_list

widgets_list = init_widgets_list("1")
secondary_widgets_list = init_secondary_widgets_list("2")
secondary_widgets_list_2 = init_secondary_widgets_list("3")


