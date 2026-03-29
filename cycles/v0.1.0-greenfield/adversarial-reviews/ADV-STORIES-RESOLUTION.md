---
document_type: adversarial-review-resolution
phase: 2
target: stories
resolved_at: 2026-03-29T16:30:00-05:00
resolved_by: story-writer
findings_resolved: [ADV-STORIES-001, ADV-STORIES-002, ADV-STORIES-003]
findings_false_positive: [ADV-STORIES-004]
---

# Adversarial Review Resolution — Stories Phase 2

## Summary

Three real findings resolved; one false positive confirmed and closed.

---

## ADV-STORIES-001 (HIGH) — RESOLVED
**Finding:** STORY-026 assigned to Wave 2 despite a hard dependency on STORY-033 (Wave 3).

**Resolution:** Moved STORY-026 to Wave 3 across all affected artifacts.

**Files changed:**
- `STORY-INDEX.md` — Wave column for STORY-026 changed from `2` → `3`
- `wave-schedule.md` — Summary table updated (Wave 2: 11→10 stories; Wave 3: 10→11 stories); STORY-026 removed from Wave 2 execution plan; added to Wave 3 execution plan as step 3c (parallel with metric group, starts after STORY-023+STORY-033 complete); Complete Story-to-Wave Reference updated
- `epics.md` — Wave column for STORY-026 changed from `2` → `3`

---

## ADV-STORIES-002 (HIGH) — RESOLVED
**Finding:** Story and point totals contradicted each other across STORY-INDEX.md, epics.md, and wave-schedule.md.

**Root causes identified:**
1. **Wave 1 arithmetic error in wave-schedule:** The effort table listed 55 pts for Wave 1 but the per-story sum was 50 pts (5+5+3+5+5+3+5+3+3+5+5+3 = 50). The prior "55" was a typo.
2. **STORY-INDEX wave summary was stale approximations:** Wave point totals in the summary table were rough estimates never updated to match per-story values.
3. **STORY-INDEX epic table had stale values:** Multiple epics had wrong story counts or point totals (EPIC-00, EPIC-01, EPIC-02, EPIC-03, EPIC-05, EPIC-06, EPIC-07, EPIC-09, EPIC-10, SR).
4. **STORY-026 move cascades wave points:** Moving STORY-026 from Wave 2 to Wave 3 changes both waves by ±3 pts.
5. **Wave 6 was 31 pts in summary but 50 per story:** The wave-schedule summary row was stale.

**Authoritative per-story point totals (verified):**

| Wave | Stories | Points |
|------|---------|--------|
| Wave 0 | 3 | 8 |
| Wave 1 | 12 | 50 |
| Wave 2 | 10 | 51 |
| Wave 3 | 11 | 51 |
| Wave 4 | 10 | 49 |
| Wave 5 | 8 | 39 |
| Wave 6 | 11 | 50 |
| **Total** | **65** | **298** |

**Epic point totals (verified from per-story values):**

| Epic | Stories | Points |
|------|---------|--------|
| EPIC-00 | 3 | 8 |
| EPIC-01 | 9 | 37 |
| EPIC-02 | 10 | 51 |
| EPIC-03 | 9 | 44 |
| EPIC-04 | 6 | 28 |
| EPIC-05 | 3 | 13 |
| EPIC-06 | 6 | 28 |
| EPIC-07 | 8 | 39 |
| EPIC-08 | 4 | 18 |
| EPIC-09 | 2 | 11 |
| EPIC-10 | 3 | 13 |
| SR | 2 | 8 |
| **Total** | **65** | **298** |

**Files changed:**
- `STORY-INDEX.md` — Summary total points corrected to 298; Epic table corrected (EPIC-00: 10→8, EPIC-01: 38→37, EPIC-02: 9 stories/47pts → 10 stories/51pts, EPIC-03: 47→44, EPIC-05: 5 stories/21pts → 3 stories/13pts, EPIC-06: 27→28, EPIC-07: 7 stories/36pts → 8 stories/39pts, EPIC-09: 3 stories/12pts → 2 stories/11pts, EPIC-10: 12→13, SR: 1 story/5pts → 2 stories/8pts); Wave summary corrected to per-story actuals
- `wave-schedule.md` — Summary table corrected (all wave points and grand total); Effort Per Wave table corrected (Wave 1: 55→50, cumulative chain corrected); reconciliation note updated

---

## ADV-STORIES-003 (MEDIUM) — RESOLVED
**Finding:** STORY-026 dependency definitions differed between STORY-INDEX.md (STORY-023, STORY-030) and dependency-graph.md (STORY-023, STORY-033).

**Resolution:** The dependency-graph.md was correct: STORY-026 (Metric Snapshot CLI Export) requires STORY-033 (Passive Latency & Throughput Metric Collection), not STORY-030 (Traffic Filtering). The STORY-026.md shard file was also correct (depends_on: [STORY-023, STORY-033]).

**STORY-INDEX.md corrected:** Depends On column for STORY-026 changed from `STORY-023, STORY-030` → `STORY-023, STORY-033`.

**No changes needed to:**
- `dependency-graph.md` — already listed STORY-023, STORY-033 ✓
- `wave-schedule.md` — Wave 3 note already referenced STORY-033 dependency ✓
- `stories/STORY-026-metric-export.md` — frontmatter already listed STORY-033 ✓

---

## ADV-STORIES-004 (HIGH) — FALSE POSITIVE (confirmed)
**Finding:** "The required per-story shards are missing."

**Status:** All 65 story shard files confirmed present at `.factory/stories/stories/`:
STORY-001-workspace-scaffold.md through STORY-065-sr-refinements.md (65 files).
The adversary could not see these files during review (likely path resolution issue).
No changes required.
