# VHS Demo — STORY-033 AC-001: Latency Histogram (p50/p95/p99)
Output docs/demo-evidence/AC-001-latency-histogram.gif
Output docs/demo-evidence/AC-001-latency-histogram.webm

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

Type "cargo test -p forge-health --test metric_tests test_BC_6_13_001_latency_histogram_computed -- --nocapture"
Enter
Sleep 8s
