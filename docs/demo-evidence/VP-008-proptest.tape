# VHS Demo — STORY-033 VP-008: Proptest — Latency Histogram Correctness
Output docs/demo-evidence/VP-008-proptest.gif
Output docs/demo-evidence/VP-008-proptest.webm

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

Type "cargo test -p forge-health --test metric_tests proptest_latency_histogram_correctness -- --nocapture"
Enter
Sleep 8s
