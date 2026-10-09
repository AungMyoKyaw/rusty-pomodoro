#!/bin/sh
# Compatibility bundle for existing release assets and the Homebrew cask.
set -eu
exec "$(dirname "$0")/package-macos.sh" --compat-slint
