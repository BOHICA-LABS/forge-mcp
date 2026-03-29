---
document_type: adversarial-review
phase: 2
target: stories
finding_id: ADV-STORIES-001
severity: HIGH
category: wave-assignment-errors
status: open
depends_on: []
blocks: [ADV-STORIES-003]
traces_to:
  - .factory/stories/STORY-INDEX.md
  - .factory/stories/dependency-graph.md
  - .factory/stories/wave-schedule.md
---

# ADV-STORIES-001 — STORY-026 is assigned to Wave 2 despite a hard dependency on Wave 3 work

- **Severity:** HIGH
- **Category:** wave-assignment-errors
- **Location:** `.factory/stories/STORY-INDEX.md`, `.factory/stories/dependency-graph.md`, `.factory/stories/wave-schedule.md`
- **Description:** STORY-026 is scheduled in Wave 2, but the dependency graph says it depends on `STORY-033`, which is a Wave 3 story. That makes the Wave 2 assignment fiction rather than plan. The wave schedule even admits the story cannot actually start until Wave 3 is done, which means the decomposition is lying to downstream scheduling logic.
- **Evidence:**
  - STORY-INDEX: `STORY-026 | Metric Snapshot JSON Export via CLI | EPIC-06 | 2 | 3 | P0 | STORY-023, STORY-030 | draft |`
  - Dependency graph: `STORY-026 | STORY-023, STORY-033 | Metric export needs CLI + metrics |`
  - Wave schedule: `STORY-026 ... Assign to Wave 2 but CAN ONLY start after STORY-033 completes in practice`
- **Proposed Fix:** Move STORY-026 to Wave 3 and make all artifacts agree on the same dependency set. If you want a Wave 2 stub, split it into two stories: a Wave 2 CLI-skeleton story and a Wave 3 metric-wiring story.
