---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-03-29T11:25:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: ""
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: "TUI Dashboard"
capability: "CAP-006"
lifecycle_status: active
introduced: v0.1.0
---

# BC-3.06.002 — Color System Auto-Detection and Degradation

## Summary

The TUI dashboard auto-detects the terminal's color capability tier (truecolor → 256-color → 16-color) and renders all UI elements using the best available tier. When a higher tier is unavailable, the system degrades gracefully through the chain. Unicode box-drawing characters fall back to ASCII equivalents when Unicode rendering is unsupported.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Terminal handle is initialized and accessible |
| PRE-002 | Environment variables ($COLORTERM, $TERM) are readable |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Color tier is detected before first frame render |
| POST-002 | All UI elements use colors from the detected tier's palette |
| POST-003 | No ANSI escape sequences exceed the detected tier's capability |
| POST-004 | Box-drawing characters render correctly for the detected character set |
| POST-005 | Color tier detection result is logged at startup (debug level) |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | The degradation chain is strictly ordered: truecolor → 256-color → 16-color → no-color |
| INV-002 | No color escape sequence is emitted that exceeds the detected tier |
| INV-003 | All semantic colors (error=red, success=green, warning=yellow) maintain their meaning across all tiers |
| INV-004 | Text remains readable at every color tier (sufficient contrast) |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | $COLORTERM and $TERM are both unset | Default to 16-color mode | — |
| EC-002 | $COLORTERM=truecolor but terminal actually only supports 256-color | Use truecolor sequences (trust environment); document as known limitation | — |
| EC-003 | Terminal reports Unicode support but renders box-drawing as garbled characters | User can force ASCII mode via `--ascii` flag or config `tui.ascii_mode = true` | FM-013 |
| EC-004 | piped output (not a TTY) | Disable all color and formatting; output plain text | — |
| EC-005 | $NO_COLOR environment variable is set | Disable all color output per no-color.org convention | — |
| EC-006 | SSH session with limited color forwarding | Detect via $TERM (e.g., "xterm" vs "xterm-256color") and use appropriate tier | — |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | $COLORTERM=truecolor | Truecolor tier selected; RGB color values used in escape sequences |
| TV-HP-002 | $TERM=xterm-256color, $COLORTERM unset | 256-color tier selected; indexed color values 0–255 used |
| TV-HP-003 | $TERM=xterm, $COLORTERM unset | 16-color tier selected; basic ANSI color codes used |
| TV-HP-004 | Unicode-capable terminal | Box-drawing uses ─│┌┐└┘├┤┬┴┼ characters |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | $NO_COLOR=1 | Zero color escape sequences in output |
| TV-EC-002 | --ascii flag provided | Box-drawing uses -|+++ ASCII characters |
| TV-EC-003 | stdout is a pipe (not TTY) | No ANSI escapes emitted; plain text only |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | Terminal capability query returns error | Fall back to 16-color; log warning "Could not detect color capability, defaulting to 16-color" |
| TV-ERR-002 | Invalid $COLORTERM value (e.g., "rainbow") | Ignore invalid value; probe $TERM instead |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all detected tiers: no escape sequence exceeds tier capability | Property-based test |
| VP-002 | For all semantic colors: meaning (error, success, warning) is preserved across all tiers | Visual regression test |
| VP-003 | When $NO_COLOR is set: zero color escape sequences appear in output | Invariant test |
| VP-004 | ASCII fallback produces valid layout identical in structure to Unicode layout | Comparison test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-006 (TUI Dashboard) |
| Failure Mode | FM-013 (terminal capability failure) |
| Related BCs | BC-3.06.001 (layout uses box-drawing), BC-3.08.005 (accessibility) |
| NFR | NFR-015 (accessibility — no color-only indicators) |
