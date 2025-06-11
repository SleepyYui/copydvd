#!/bin/bash

echo "=== Copy DVD Notification Test ==="
echo ""
echo "1. First, let's check if the app bundle was created correctly:"
ls -la "Copy DVD.app/Contents/Info.plist"
echo ""

echo "2. Opening the app bundle to trigger notification permissions..."
open "Copy DVD.app"
echo "   -> The app should launch now"
echo ""

echo "3. To check if notifications are working:"
echo "   a) Go to System Preferences > Notifications & Focus"
echo "   b) Look for 'Copy DVD' in the left sidebar"
echo "   c) Make sure notifications are enabled"
echo ""

echo "4. Alternative: Open notification settings directly"
echo "   Running: open /System/Library/PreferencePanes/Notifications.prefPane"
open /System/Library/PreferencePanes/Notifications.prefPane
echo ""

echo "5. In the Copy DVD app:"
echo "   - Go to the HandBrake tab"
echo "   - Click 'Test OS Notification' button"
echo "   - You should see a notification in the top-right corner"
echo ""

echo "6. If you don't see the 'Copy DVD' app in notification settings:"
echo "   - Try running the app bundle again: open 'Copy DVD.app'"
echo "   - Or try double-clicking on 'Copy DVD.app' in Finder"
echo "   - The first time you run an app bundle, macOS asks for permissions"
echo ""

echo "=== Done! ==="