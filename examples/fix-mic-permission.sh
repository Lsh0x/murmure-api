#!/bin/bash
# Quick script to open macOS microphone privacy settings

echo "🔧 Opening macOS Microphone Privacy Settings..."
echo ""
echo "In the settings window that opens:"
echo "1. Find your terminal app (Terminal, iTerm, Cursor, VS Code, etc.)"
echo "2. Check the box to enable microphone access"
echo "3. Restart your terminal app"
echo ""
echo "Press Enter to open settings, or Ctrl+C to cancel..."
read

# Open macOS Privacy & Security → Microphone settings
open 'x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone' 2>/dev/null || \
open 'x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone'

