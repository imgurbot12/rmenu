#!/bin/sh

# basic dmenu-like implementation
eval "`printf 'ls /\ndf -h\nwho\nfoot -e top' | rmenu -f dmenu`"

# supports dmenu
echo "========="
echo try "\"printf 'foo\\\\nbar\\\\nbaz' | rmenu -f dmenu\""
echo try "\"exec \`dmenu_path | rmenu -f dmenu\`\""

