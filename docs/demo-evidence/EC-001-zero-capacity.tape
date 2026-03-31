# STORY-029 / EC-001 — Zero-capacity buffer (edge case)
# Run from worktree root: vhs docs/demo-evidence/EC-001-zero-capacity.tape

Output docs/demo-evidence/EC-001-zero-capacity.gif
Output docs/demo-evidence/EC-001-zero-capacity.webm

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

# EC-001: capacity=0; every push returns Some(item), len() stays 0
Type "cargo test -p forge-traffic --test buffer_tests test_zero_capacity_buffer -- --nocapture"
Enter
Wait+Screen /test result/
Sleep 2s
