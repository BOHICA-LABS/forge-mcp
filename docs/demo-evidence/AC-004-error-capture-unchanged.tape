# AC-004 (ERROR PATH): Capture buffer IDs and payloads unchanged after replay (INV-004)
# BC-4.10.003 VP-004 | STORY-032
Output docs/demo-evidence/AC-004-error-capture-unchanged.gif
Output docs/demo-evidence/AC-004-error-capture-unchanged.webm
Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 20
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms
Hide
Type "cd /Users/jmagady/Dev/forge-mcp/.worktrees/STORY-032 && clear"
Enter
Sleep 1s
Show
Type "cargo test -p forge-traffic test_BC_4_10_003_invariant_capture_buffer_unchanged -- --nocapture 2>&1 | grep -E 'test |ok$|FAILED'"
Enter
Sleep 5s
Sleep 2s
