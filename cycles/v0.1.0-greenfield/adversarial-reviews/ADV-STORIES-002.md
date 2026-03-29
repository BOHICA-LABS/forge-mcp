---
document_type: adversarial-review
phase: 2
target: stories
finding_id: ADV-STORIES-002
severity: HIGH
category: contradictions
status: open
depends_on: []
blocks: []
traces_to:
  - .factory/stories/STORY-INDEX.md
  - .factory/stories/epics.md
  - .factory/stories/wave-schedule.md
---

# ADV-STORIES-002 — Story and point totals contradict each other across the core planning artifacts

- **Severity:** HIGH
- **Category:** contradictions
- **Location:** `.factory/stories/STORY-INDEX.md`, `.factory/stories/epics.md`, `.factory/stories/wave-schedule.md`
- **Description:** The planning artifacts disagree on basic arithmetic: total story counts, total points, and epic counts. That is not cosmetic. Wave planning, staffing, and critical-path analysis all depend on these totals being trustworthy.
- **Evidence:**
  - STORY-INDEX summary says: `65 BCs → 60 implementation stories + 5 SR refinement stories = 65 total stories`, but the same file lists `EPIC-02` with 10 stories and `SR` with 2 stories while the summary claims `EPIC-00 through EPIC-10 + SR refinements` equals `12` epics.
  - STORY-INDEX summary says `Total story points | 276`.
  - Wave schedule effort table says the per-story totals sum to `303` and explicitly states `STORY-INDEX.md reports 276 total points; this schedule sums to 303`.
  - Wave schedule summary says `Wave 6 | 11 | 31`, but its own detailed effort table gives Wave 6 `50 pts`.
- **Proposed Fix:** Recompute all counts from the authoritative per-story shards, then regenerate STORY-INDEX, epics, dependency-graph summaries, and wave-schedule from that single source of truth. Do not leave reconciliation notes as a substitute for fixing the numbers.
