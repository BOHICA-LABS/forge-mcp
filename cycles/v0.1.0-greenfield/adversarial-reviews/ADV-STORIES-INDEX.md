---
document_type: adversarial-review
phase: 2
target: stories
status: open
finding_count: 4
created_at: 2026-03-29T16:13:00-05:00
---

# Adversarial Review Index — Stories

## Findings Summary

| ID | Severity | Category | Title | Depends On | Blocks |
|----|----------|----------|-------|------------|--------|
| ADV-STORIES-001 | HIGH | wave-assignment-errors | STORY-026 is assigned to Wave 2 despite a hard dependency on Wave 3 work | — | ADV-STORIES-003 |
| ADV-STORIES-002 | HIGH | contradictions | Story and point totals contradict each other across the core planning artifacts | — | — |
| ADV-STORIES-003 | MEDIUM | dependency-problems | STORY-026 dependency definitions differ between artifacts | ADV-STORIES-001 | — |
| ADV-STORIES-004 | HIGH | ac-quality | The required per-story shards are missing, so acceptance-criteria quality cannot be reviewed at all | — | — |

## Category Groups

### wave-assignment-errors
- ADV-STORIES-001

### contradictions
- ADV-STORIES-002

### dependency-problems
- ADV-STORIES-003

### ac-quality
- ADV-STORIES-004

## Dependency Notes

- ADV-STORIES-003 depends on ADV-STORIES-001 because the dependency mismatch and the bad wave placement are the same planning break viewed from two angles.

## Review Notes

- BC coverage appears complete at the aggregate-matrix level, but the story package is not trustworthy because the underlying shard files are missing and the totals are internally inconsistent.
- Critical-path reasoning is also degraded by the arithmetic contradictions between STORY-INDEX and wave-schedule.
