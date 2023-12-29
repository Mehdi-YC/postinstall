
from typing import List  # noqa: F401
import os
import subprocess
from os import path

from libqtile import bar, layout, widget, hook, qtile
from libqtile.config import Click, Drag, Group, ScratchPad, DropDown, Key, Match, Screen
from libqtile.lazy import lazy
from settings.path import qtile_path
import colors
import bar



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
            active = colors[4],
            inactive = foregroundColor,
            hide_unused = False,
            rounded = False,
            highlight_method = "line",
            highlight_color = [backgroundColor, backgroundColor],
            this_current_screen_border = colors[5],
            this_screen_border = colors[7],
            other_screen_border = colors[6],
            other_current_screen_border = colors[6],
            urgent_alert_method = "line",
            urgent_border = colors[9],
            urgent_text = colors[1],
            foreground = foregroundColor,
            background = backgroundColor,
            use_mouse_wheel = False
        ),
        widget.TaskList(
            icon_size = 0,
            font = "JetBrainsMono Nerd Font",
            foreground = colors[0],
            background = colors[2],
            borderwidth = 0,
            border = colors[6],
            margin = 0,
            padding = 8,
            highlight_method = "block",
            title_width_method = "uniform",
            urgent_alert_method = "border",
            urgent_border = colors[1],
            rounded = False,
            txt_floating = "🗗 ",
            txt_maximized = "🗖 ",
            txt_minimized = "🗕 ",
        ),
        widget.Sep(linewidth = 1, padding = 10, foreground = colors[5],background = backgroundColor),
        widget.OpenWeather(
            app_key = "4cf3731a25d1d1f4e4a00207afd451a2",
            cityid = "4997193",
            format = '{icon} {main_temp}°',
            metric = False,
            font = "JetBrainsMono Nerd Font",
            foreground = foregroundColor,
        ),
        widget.Sep(linewidth = 0, padding = 10),
        widget.TextBox(text = " ", fontsize = 14, font = "JetBrainsMono Nerd Font", foreground = colors[7]),
        widget.CPU(
            font = "JetBrainsMono Nerd Font",
            update_interval = 1.0,
            format = '{load_percent}%',
            foreground = foregroundColor,
            padding = 5
        ),
        widget.Sep(linewidth = 0, padding = 10),
        widget.TextBox(text = "", fontsize = 14, font = "JetBrainsMono Nerd Font", foreground = colors[3]),
        widget.Memory(
            font = "JetBrainsMonoNerdFont",
            foreground = foregroundColor,
            format = '{MemUsed: .0f}{mm} /{MemTotal: .0f}{mm}',
            measure_mem='G',
            padding = 5,
        ),
        widget.Sep(linewidth = 0, padding = 10),
        widget.TextBox(text = " ", fontsize = 14, font = "JetBrainsMono Nerd Font", foreground = colors[10]),
        widget.Clock(format='%I:%M %p', font = "JetBrainsMono Nerd Font", padding = 10, foreground = foregroundColor),
        widget.Systray(background = backgroundColor, icon_size = 20, padding = 4),
        widget.Sep(linewidth = 1, padding = 10, foreground = colors[5], background = backgroundColor),
        widget.CurrentLayoutIcon(scale = 0.5, foreground = colors[6], background = colors[6]),
    ]

    return widgets_list

def init_secondary_widgets_list(monitor_num):
    secondary_widgets_list = init_widgets_list(monitor_num)
    del secondary_widgets_list[13:15]
    return secondary_widgets_list

widgets_list = init_widgets_list("1")
secondary_widgets_list = init_secondary_widgets_list("2")

screens = [
    Screen(top=bar.Bar(widgets=widgets_list, size=36, background=backgroundColor, margin=6, opacity=0.8),),
    Screen(top=bar.Bar(widgets=secondary_widgets_list, size=36, background=backgroundColor, margin=6, opacity=0.8),),
    ]


