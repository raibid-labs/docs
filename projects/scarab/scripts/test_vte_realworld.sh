#!/bin/bash
# Real-world VTE parser testing script
# Tests with actual terminal applications

echo "=== Real-World VTE Parser Tests ==="
echo ""
echo "These tests verify that Scarab can handle real terminal applications."
echo ""

# Test 1: ls with colors
echo "Test 1: ls --color=auto"
echo "Running: ls --color=auto -lah"
ls --color=auto -lah
echo ""
sleep 2

# Test 2: grep with colors
echo "Test 2: grep with color"
echo "Running: echo 'test pattern test' | grep --color=auto pattern"
echo "test pattern test" | grep --color=auto "pattern"
echo ""
sleep 2

# Test 3: Tree command (if available)
if command -v tree &> /dev/null; then
    echo "Test 3: tree command"
    echo "Running: tree -L 2 -C"
    tree -L 2 -C 2>/dev/null || echo "Tree command failed"
    echo ""
    sleep 2
fi

# Test 4: Git status (if in a git repo)
if git rev-parse --is-inside-work-tree &> /dev/null; then
    echo "Test 4: git status with color"
    echo "Running: git status"
    git status
    echo ""
    sleep 2
fi

# Test 5: Colored output from Python
echo "Test 5: Python colored output"
python3 -c "
import sys
# ANSI color codes
RED = '\033[91m'
GREEN = '\033[92m'
YELLOW = '\033[93m'
BLUE = '\033[94m'
RESET = '\033[0m'

print(f'{RED}Red text{RESET}')
print(f'{GREEN}Green text{RESET}')
print(f'{YELLOW}Yellow text{RESET}')
print(f'{BLUE}Blue text{RESET}')
"
echo ""
sleep 2

# Test 6: Simple progress bar
echo "Test 6: Progress bar animation"
echo "Running a simple progress simulation..."
for i in {1..20}; do
    printf "\r["
    for j in $(seq 1 $i); do printf "="; done
    printf ">"
    for j in $(seq $i 20); do printf " "; done
    printf "] %d%%" $((i * 5))
    sleep 0.1
done
printf "\n"
echo ""

# Test 7: Cursor positioning
echo "Test 7: Cursor positioning demo"
echo -e "\x1b[s"  # Save cursor position
echo "This text will be overwritten..."
sleep 1
echo -e "\x1b[u"  # Restore cursor position
echo "Text has been replaced!        "
echo ""

# Test 8: Clear screen demo
echo "Test 8: Screen clearing"
echo "About to clear screen in 2 seconds..."
sleep 2
echo -e "\x1b[2J\x1b[H"  # Clear screen and move cursor to home
echo "Screen cleared! Terminal is working."
echo ""

echo "=== Real-World Tests Complete ==="
echo ""
echo "If you saw colors, cursor movements, and proper formatting,"
echo "the VTE parser is working correctly!"
