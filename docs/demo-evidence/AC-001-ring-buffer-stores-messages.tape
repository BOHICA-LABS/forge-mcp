# STORY-029 / AC-001 — Ring buffer stores messages (BC-4.09.003)
# Run from worktree root: vhs docs/demo-evidence/AC-001-ring-buffer-stores-messages.tape

Output docs/demo-evidence/AC-001-ring-buffer-stores-messages.gif
Output docs/demo-evidence/AC-001-ring-buffer-stores-messages.webm

Require cargo

Set Shell "zsh"
Set FontFamily "JetBrains Mono"
Set FontSize 20
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms
Set WaitTimeout 30s

Hide
Type "cd /Users/jmagady/Dev/forge-mcp/.worktrees/STORY-029"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

# AC-001: ring buffer stores 5 items, len()==5, iter() in insertion order
Type "cargo test -p forge-traffic --test buffer_tests test_BC_4_09_003_ring_buffer_stores_messages -- --nocapture"
Enter
Wait+Screen /test result/
Sleep 2s
