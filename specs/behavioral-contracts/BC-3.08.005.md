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
capability: "CAP-008"
lifecycle_status: active
introduced: v0.1.0
---

# BC-3.08.005 — Accessibility: No Color-Only Indicators

## Summary

Every visual indicator in the TUI that uses color to convey information must also use a non-color differentiator (shape, symbol, or text label). This ensures the TUI is usable by people with color vision deficiencies and in environments where color is unavailable. This contract applies globally to all TUI panes and widgets.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | TUI is rendering any visual indicator that conveys state or severity |
| PRE-002 | This contract applies regardless of detected color tier (truecolor through no-color) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Connection status uses symbol + text: `● Connected [✓]`, `○ Disconnected [—]`, `✕ Error [!]` |
| POST-002 | Severity levels include text labels: "critical", "warning", "info", "debug" alongside any color |
| POST-003 | Direction indicators use symbols: `→` (client-to-server), `←` (server-to-client) |
| POST-004 | Focus indication uses border style change (double-line or bright) in addition to color |
| POST-005 | Search match highlighting uses bold/reverse attribute in addition to color |
| POST-006 | In no-color mode, all information conveyed by color is still distinguishable by symbol/text |

## Indicator Catalog

| Indicator Type | Color | Symbol | Text Label |
|---------------|-------|--------|------------|
| Connected | Green | ● | [✓] Connected |
| Disconnected | Gray | ○ | [—] Disconnected |
| Error | Red | ✕ | [!] Error |
| Unknown | Yellow | ? | [?] Unknown |
| Client→Server | Blue | → | (direction) |
| Server→Client | Magenta | ← | (direction) |
| Focused pane | Bright border | Double-line border | (title bold) |
| Search match | Yellow highlight | Bold + reverse | — |
| Warning | Yellow | ⚠ | WARNING |
| Critical error | Red | ✖ | CRITICAL |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | No information is conveyed by color alone — every color has a paired non-color differentiator |
| INV-002 | The indicator catalog is exhaustive — adding a new colored indicator requires updating this BC |
| INV-003 | In no-color mode ($NO_COLOR=1), all state information remains distinguishable |
| INV-004 | Symbols used for indicators are from the Unicode Basic Multilingual Plane (wide compatibility) |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Terminal in no-color mode ($NO_COLOR=1) | All indicators rendered with symbols and text only; no color escape sequences | — |
| EC-002 | ASCII-only mode (no Unicode) | Symbols degrade: ● → *, ○ → o, ✕ → x, → → ->, ← → <-, ⚠ → !, ✖ → X | — |
| EC-003 | Screen reader consuming terminal output | Text labels ensure screen readers announce state correctly | NFR-015 |
| EC-004 | New widget type added without accessibility review | Missing non-color indicator fails accessibility audit (VP-001) | — |
| EC-005 | High-contrast terminal theme | Symbols and text remain visible regardless of theme (no hardcoded background colors) | — |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | Server in Connected state, truecolor terminal | `● Connected [✓]` rendered in green with both symbol and text |
| TV-HP-002 | Server in Error state, 16-color terminal | `✕ Error [!]` rendered in red with both symbol and text |
| TV-HP-003 | No-color mode | `● Connected [✓]` rendered without color; symbol and text convey state |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | ASCII mode + no-color | `* Connected [✓]` rendered as plain ASCII text |
| TV-EC-002 | All three connection states visible simultaneously | Each state distinguishable by symbol alone (●, ○, ✕) without relying on color |
| TV-EC-003 | Focused pane in no-color mode | Pane border uses double-line style; title is bold |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | Developer adds new status "Reconnecting" with orange color but no symbol | Accessibility audit (VP-001) fails and reports missing non-color indicator |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all rendered indicators: a non-color differentiator (symbol or text) is present | Exhaustive UI audit test |
| VP-002 | In no-color mode: for all pairs of distinct states, their rendered indicators are visually different | Property-based test |
| VP-003 | ASCII fallback mode produces valid output for all indicator types | Invariant test |
| VP-004 | Indicator catalog covers all state-conveying UI elements | Manual review + automated grep |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-008 (TUI Data Display) |
| NFR | NFR-015 (accessibility — no color-only indicators) |
| Related BCs | BC-3.06.002 (color system), BC-3.08.003 (server badges), BC-3.08.001 (message rendering) |
