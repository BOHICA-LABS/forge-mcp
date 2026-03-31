Output docs/demo-evidence/test-simple.gif
Output docs/demo-evidence/test-simple.webm

Require cargo

Set Shell "bash"
Set FontFamily "JetBrains Mono"
Set FontSize 20
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Framerate 30
Set WaitTimeout 60s

Hide
Type "cd /Users/jmagady/Dev/forge-mcp/.worktrees/STORY-033 && clear"
Enter
Sleep 2s
Show

Type "cargo test -p forge-health --test metric_tests test_BC_6_13_001_latency_histogram_computed -- --nocapture"
Enter
Wait /1 passed/
Sleep 2s
