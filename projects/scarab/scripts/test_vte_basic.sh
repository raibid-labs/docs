#!/bin/bash
# Basic VTE parser testing script
# Tests fundamental terminal emulation features

echo "=== Scarab VTE Parser Test Suite ==="
echo ""

# Test 1: Basic text output
echo "Test 1: Basic text output"
echo "Hello, Scarab Terminal!"
echo ""

# Test 2: ANSI Colors (8 colors)
echo "Test 2: ANSI 8-color support"
echo -e "\x1b[30mBlack\x1b[0m"
echo -e "\x1b[31mRed\x1b[0m"
echo -e "\x1b[32mGreen\x1b[0m"
echo -e "\x1b[33mYellow\x1b[0m"
echo -e "\x1b[34mBlue\x1b[0m"
echo -e "\x1b[35mMagenta\x1b[0m"
echo -e "\x1b[36mCyan\x1b[0m"
echo -e "\x1b[37mWhite\x1b[0m"
echo ""

# Test 3: Bright colors (16 colors)
echo "Test 3: ANSI 16-color support (bright)"
echo -e "\x1b[90mBright Black\x1b[0m"
echo -e "\x1b[91mBright Red\x1b[0m"
echo -e "\x1b[92mBright Green\x1b[0m"
echo -e "\x1b[93mBright Yellow\x1b[0m"
echo -e "\x1b[94mBright Blue\x1b[0m"
echo -e "\x1b[95mBright Magenta\x1b[0m"
echo -e "\x1b[96mBright Cyan\x1b[0m"
echo -e "\x1b[97mBright White\x1b[0m"
echo ""

# Test 4: Background colors
echo "Test 4: Background colors"
echo -e "\x1b[41mRed Background\x1b[0m"
echo -e "\x1b[42mGreen Background\x1b[0m"
echo -e "\x1b[43mYellow Background\x1b[0m"
echo -e "\x1b[44mBlue Background\x1b[0m"
echo ""

# Test 5: Text attributes
echo "Test 5: Text attributes"
echo -e "\x1b[1mBold Text\x1b[0m"
echo -e "\x1b[2mDim Text\x1b[0m"
echo -e "\x1b[3mItalic Text\x1b[0m"
echo -e "\x1b[4mUnderline Text\x1b[0m"
echo -e "\x1b[7mInverse Text\x1b[0m"
echo -e "\x1b[1;31mBold Red\x1b[0m"
echo -e "\x1b[4;32mUnderline Green\x1b[0m"
echo ""

# Test 6: Cursor movement
echo "Test 6: Cursor movement"
echo -e "Start\x1b[10CMiddle\x1b[10CEnd"
echo -e "\x1b[5;10HPositioned at (10,5)"
echo ""

# Test 7: Clear operations
echo "Test 7: Clear operations"
echo "Line before clear"
echo -e "Start\x1b[KEnd"
echo "Line after clear"
echo ""

# Test 8: UTF-8 support
echo "Test 8: UTF-8 multibyte characters"
echo "English: Hello World"
echo "Chinese: 你好世界"
echo "Japanese: こんにちは"
echo "Emoji: 🚀 🎨 💻 ⚡"
echo "Math: π ≈ 3.14159"
echo ""

# Test 9: 256-color support
echo "Test 9: 256-color palette"
echo -e "\x1b[38;5;196mRed (196)\x1b[0m"
echo -e "\x1b[38;5;46mGreen (46)\x1b[0m"
echo -e "\x1b[38;5;21mBlue (21)\x1b[0m"
echo ""

# Test 10: Line wrapping
echo "Test 10: Line wrapping"
echo "This is a very long line that should wrap around when it reaches the edge of the terminal window if the terminal is narrow enough to cause wrapping behavior"
echo ""

echo "=== Test Suite Complete ==="
