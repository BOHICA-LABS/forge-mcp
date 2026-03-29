---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:39:00
phase: 1d
inputs: [specs/prd.md, specs/architecture/ARCH-INDEX.md]
input-hash: ""
traces_to: specs/prd.md
source_bc: [BC-7.18.002]
module: forge-security
proof_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-015: Suppression Overlay Preserves Finding Integrity

## Property Statement

For any `AuditReport` `r` containing findings `F₁..Fₙ` and any set of `SuppressionRule`s `S`, applying suppression produces a new report `r'` where:

1. **Identity preservation:** Every finding in `r` exists in `r'` with the same UUID/identity.
2. **Content immutability:** For every finding `f` in `r`, the corresponding finding `f'` in `r'` has identical `severity`, `confidence`, `evidence`, `rule_id`, `server`, and `tool` fields.
3. **Overlay-only mutation:** The only field that may differ between `f` and `f'` is `suppressed` (false→true) and the associated `suppression_reason` metadata.
4. **No deletion:** `|r'.findings| == |r.findings|` — suppression never removes findings.
5. **Exit-code exclusion:** Suppressed findings do not contribute to the exit code, but unsuppressed findings are unaffected.

This is a **data integrity** property: suppression is a pure overlay that annotates without mutating or deleting.

## Source Contract

- **BC-7.18.002** — User-Defined Finding Suppression Rules. POST-002: suppressed finding is marked `suppressed: true` but NOT removed. POST-003: original finding content is unchanged after suppression.
- **DI-012** — Suppression is overlay: original finding is preserved, not deleted.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Property-based testing (proptest) |
| Tool | `proptest` crate |
| Input space | Arbitrary findings (1..50) × arbitrary suppression rules (0..20) |
| Iterations | 256+ cases |
| Shrinking | Automatic — fails shrink to minimal finding set + rule set |

## Harness Skeleton

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn suppression_preserves_finding_identity(
        findings in prop::collection::vec(arb_security_finding(), 1..50),
        rules in prop::collection::vec(arb_suppression_rule(), 0..20),
    ) {
        let report = AuditReport::from_findings(findings.clone());
        let suppressed_report = report.apply_suppression(&rules);

        // P1: No findings deleted
        prop_assert_eq!(
            suppressed_report.findings.len(),
            report.findings.len(),
            "Suppression must not add or remove findings"
        );

        // P2: Identity and content preserved
        for (original, after) in report.findings.iter().zip(suppressed_report.findings.iter()) {
            prop_assert_eq!(original.uuid, after.uuid, "Finding identity changed");
            prop_assert_eq!(original.severity, after.severity, "Severity mutated by suppression");
            prop_assert_eq!(original.confidence, after.confidence, "Confidence mutated by suppression");
            prop_assert_eq!(original.evidence, after.evidence, "Evidence mutated by suppression");
            prop_assert_eq!(original.rule_id, after.rule_id, "Rule ID mutated by suppression");
            prop_assert_eq!(original.server, after.server, "Server mutated by suppression");
            prop_assert_eq!(original.tool, after.tool, "Tool mutated by suppression");
        }
    }

    #[test]
    fn suppression_only_sets_overlay_flag(
        findings in prop::collection::vec(arb_security_finding(), 1..50),
        rules in prop::collection::vec(arb_suppression_rule(), 0..20),
    ) {
        let report = AuditReport::from_findings(findings.clone());
        let suppressed_report = report.apply_suppression(&rules);

        for (original, after) in report.findings.iter().zip(suppressed_report.findings.iter()) {
            // Only `suppressed` flag and `suppression_reason` may differ
            if original.suppressed == after.suppressed {
                prop_assert_eq!(
                    original.suppression_reason, after.suppression_reason,
                    "Suppression reason changed without flag change"
                );
            }
            // If suppressed changed from false→true, reason must be Some
            if !original.suppressed && after.suppressed {
                prop_assert!(
                    after.suppression_reason.is_some(),
                    "Suppressed finding must have a reason"
                );
            }
        }
    }

    #[test]
    fn suppressed_findings_excluded_from_exit_code(
        findings in prop::collection::vec(arb_security_finding(), 1..50),
        rules in prop::collection::vec(arb_suppression_rule(), 0..20),
    ) {
        let report = AuditReport::from_findings(findings.clone());
        let suppressed_report = report.apply_suppression(&rules);

        let exit_code = suppressed_report.compute_exit_code();
        let unsuppressed_count = suppressed_report.findings.iter()
            .filter(|f| !f.suppressed)
            .count();

        if unsuppressed_count == 0 {
            prop_assert_eq!(exit_code, 0, "All findings suppressed but exit code non-zero");
        }
        // If any unsuppressed findings exist, exit code must reflect them
        if unsuppressed_count > 0 {
            prop_assert_ne!(exit_code, 0, "Unsuppressed findings present but exit code is 0");
        }
    }
}
```

### Strategy: `arb_security_finding`

Generates arbitrary security findings with realistic fields:

```rust
fn arb_security_finding() -> impl Strategy<Value = SecurityFinding> {
    (
        "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}",  // uuid
        prop::sample::select(vec!["ssrf-private-ip", "dangerous-tool-exec", "schema-drift"]),
        prop::sample::select(vec!["critical", "high", "medium", "low"]),
        0.0f64..=1.0,  // confidence
        "[a-z-]{1,20}",  // server
        "[a-z_]{1,20}",  // tool
    ).prop_map(|(uuid, rule_id, severity, confidence, server, tool)| SecurityFinding {
        uuid,
        rule_id: rule_id.to_string(),
        severity: severity.to_string(),
        confidence,
        evidence: "test evidence".to_string(),
        server,
        tool,
        suppressed: false,
        suppression_reason: None,
    })
}
```

### Strategy: `arb_suppression_rule`

```rust
fn arb_suppression_rule() -> impl Strategy<Value = SuppressionRule> {
    (
        prop::sample::select(vec!["ssrf-private-ip", "dangerous-tool-exec", "schema-drift"]),
        prop::option::of("[a-z-]{1,20}"),  // server filter
        prop::option::of("[a-z_*?]{1,20}"),  // tool glob
        "test reason",
    ).prop_map(|(rule, server, tool, reason)| SuppressionRule {
        rule: rule.to_string(),
        server,
        tool,
        reason: reason.to_string(),
        created_at: "2026-03-29".to_string(),
        expires_at: None,
    })
}
```

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | Bounded findings (1..50), bounded rules (0..20), structured types |
| Complexity | Low — equality checks on struct fields before/after transformation |
| Tool support | Excellent — proptest with custom strategies, automatic shrinking |
| Pure function | `apply_suppression` is a pure transformation (takes report + rules, returns report) |
| Time | Milliseconds per case |
| Verdict | **FEASIBLE** |

## Composition

- Composes with VP-009 (IP classification) and VP-010 (confidence bounds): suppression operates on findings *after* classification and scoring. VP-015 guarantees those scores survive suppression.
- Composes with VP-011 (schema drift detection): drift findings must survive suppression overlay intact.

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 (ADV-P1-006 resolution) |
| Modified | — |
| Deprecated | — |
| Retired | — |
