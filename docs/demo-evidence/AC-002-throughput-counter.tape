# VHS Demo — STORY-033 AC-002: Throughput Counter (messages/sec)
Output docs/demo-evidence/AC-002-throughput-counter.gif
Output docs/demo-evidence/AC-002-throughput-counter.webm

Require cargo

Set Shell "bash"
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

Hide
Type "cd /Users/jmagady/Dev/forge-mcp/.worktrees/STORY-033 && clear"
Enter
Sleep 2s
Show

Type "cargo test -p forge-health --test metric_tests test_BC_6_13_001_throughput_counter_updates -- --nocapture"
Enter
Sleep 8s
