#!/bin/bash

# Get current workspace number
current_ws=$(i3-msg -t get_workspaces | jq '.[] | select(.focused).num')

# Calculate the next workspace number
next_ws=$((current_ws + $1))

# Move the container to the next workspace
i3-msg move container to workspace "$next_ws"

# Switch to the next workspace
i3-msg workspace "$next_ws"
