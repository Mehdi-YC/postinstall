#!/bin/bash
set -e

# Make sure wl-paste exists
if ! command -v wl-paste >/dev/null 2>&1; then
    notify-send "Slugify Clipboard" "wl-paste not found (install wl-clipboard)"
    exit 1
fi

# Get clipboard text
clipboard=$(wl-paste)

# Exit if empty
if [[ -z "$clipboard" ]]; then
    notify-send "Slugify Clipboard" "Clipboard is empty"
    exit 1
fi

# Slugify and clean
slugified=$(echo "$clipboard" | \
    iconv -f utf-8 -t ascii//TRANSLIT 2>/dev/null | \
    tr '[:upper:]' '[:lower:]' | \
    sed 's/[^a-z0-9_ ]/_/g' | \
    tr '[:space:]' '_' | \
    sed 's/__*/_/g' | \
    sed 's/^_//;s/_$//')

# Remove French stopwords
slugified=$(echo "$slugified" | \
    sed -E "s/^(de|la|les|l|d|un|une)_//g" | \
    sed -E "s/_?(de|la|les|l|d|un|une)_/_/g")

# Copy back
echo -n "$slugified" | wl-copy

# Optional: show a notification
notify-send "Slugify Clipboard" "✅ Copied: $slugified"
