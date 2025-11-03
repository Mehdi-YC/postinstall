#!/bin/bash
set -e

# Check dependencies
if ! command -v wl-paste >/dev/null 2>&1; then
    notify-send "Slugify Clipboard" "❌ wl-paste not found (install wl-clipboard)"
    exit 1
fi

# Get clipboard content
clipboard=$(wl-paste)

# Exit if empty
if [[ -z "$clipboard" ]]; then
    notify-send "Slugify Clipboard" "❌ Clipboard is empty"
    exit 1
fi

# Slugify and normalize accents
slugified=$(echo "$clipboard" | \
    iconv -f utf-8 -t ascii//TRANSLIT 2>/dev/null | \
    tr '[:upper:]' '[:lower:]' | \
    sed 's/[^a-z0-9_ ]/_/g' | \
    tr '[:space:]' '_' | \
    sed 's/__*/_/g')

# Remove French stopwords at start or between underscores
slugified=$(echo "$slugified" | \
    sed -E "s/^(de|la|les|le|l|d|du|des|un|une)_//g" | \
    sed -E "s/_?(de|la|les|le|l|d|du|des|un|une)_/_/g")

# Final cleanup — remove extra or leading/trailing underscores
slugified=$(echo "$slugified" | \
    sed 's/__*/_/g' | \
    sed 's/^_//;s/_$//')

# Copy back to clipboard
echo -n "$slugified" | wl-copy

# Optional: show a notification (comment out if not desired)
notify-send "Slugify Clipboard" "✅ Copied: $slugified"
