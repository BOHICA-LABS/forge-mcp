---
document_type: adversarial-review
phase: 2
target: stories
finding_id: ADV-STORIES-003
severity: MEDIUM
category: dependency-problems
status: open
depends_on: [ADV-STORIES-001]
blocks: []
traces_to:
  - .factory/stories/STORY-INDEX.md
  - .factory/stories/dependency-graph.md
  - .factory/stories/wave-schedule.md
---

# ADV-STORIES-003 — STORY-026 dependency definitions differ between artifacts

- **Severity:** MEDIUM
- **Category:** dependency-problems
- **Location:** `.factory/stories/STORY-INDEX.md`, `.factory/stories/dependency-graph.md`
- **Description:** STORY-026 depends on different stories depending on which artifact you read. That means implementers can satisfy one document and still violate another.
- **Evidence:**
  - STORY-INDEX lists `STORY-026 ... Depends On | STORY-023, STORY-030`.
  - Dependency graph lists `STORY-026 | STORY-023, STORY-033 | Metric export needs CLI + metrics`.
  - Wave schedule prose sides with the `STORY-033` dependency and explains why the story cannot complete in Wave 2.
- **Proposed Fix:** Decide whether metric snapshot export depends on filtering (`STORY-030`) or metrics collection (`STORY-033`) — almost certainly the latter — and normalize all three artifacts to the same dependency set. If both are needed, list both explicitly everywhere.
