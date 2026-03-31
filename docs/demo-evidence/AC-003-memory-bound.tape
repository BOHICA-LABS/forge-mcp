# STORY-029 / AC-003 — Memory bound (BC-4.09.003 / NFR-012)
# Run from worktree root: vhs docs/demo-evidence/AC-003-memory-bound.tape

Output docs/demo-evidence/AC-003-memory-bound.gif
Output docs/demo-evidence/AC-003-memory-bound.webm

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

# AC-003: 200 pushes, memory_usage_bytes() never exceeds 1000B max_bytes
Type "cargo test -p forge-traffic --test buffer_tests test_BC_4_09_003_memory_bound -- --nocapture"
Enter
Wait+Screen /test result/
Sleep 2s
