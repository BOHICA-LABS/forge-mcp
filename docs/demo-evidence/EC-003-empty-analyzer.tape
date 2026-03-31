# EC-003: Empty Analyzer State (Edge Case)
# STORY-028 — Per-Message Timing & Throughput Analysis
# Demonstrates: empty TimingAnalyzer returns safe zero-state values

Output docs/demo-evidence/EC-003-empty-analyzer.gif
Output docs/demo-evidence/EC-003-empty-analyzer.webm

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

Hide
Type "cd /Users/jmagady/Dev/forge-mcp/.worktrees/STORY-028"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "cargo test -p forge-traffic --test timing_tests test_empty_analyzer -- --nocapture 2>&1"
Enter
Sleep 10s
