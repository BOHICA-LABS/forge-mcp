# STORY-029 / AC-004 — Eviction warning threshold (BC-4.09.003 / E-CAP-001)
# Run from worktree root: vhs docs/demo-evidence/AC-004-eviction-warning.tape

Output docs/demo-evidence/AC-004-eviction-warning.gif
Output docs/demo-evidence/AC-004-eviction-warning.webm

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

# AC-004: 9/10 items => 90% capacity threshold; overflow pushes stay within bounds
Type "cargo test -p forge-traffic --test buffer_tests test_BC_4_09_003_eviction_warning_emitted -- --nocapture"
Enter
Wait+Screen /test result/
Sleep 2s
