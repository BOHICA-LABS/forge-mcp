# STORY-029 / AC-005 — O(1) append at scale (BC-4.09.003 / AD-006)
# Run from worktree root: vhs docs/demo-evidence/AC-005-o1-append.tape

Output docs/demo-evidence/AC-005-o1-append.gif
Output docs/demo-evidence/AC-005-o1-append.webm

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

# AC-005: 1000 pushes into cap-100 buffer; len==100, is_full()==true, no panic
Type "cargo test -p forge-traffic --test buffer_tests test_BC_4_09_003_append_is_constant_time -- --nocapture"
Enter
Wait+Screen /test result/
Sleep 2s
