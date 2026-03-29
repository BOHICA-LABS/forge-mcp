---
document_type: adversarial-review
phase: 2
target: stories
finding_id: ADV-STORIES-004
severity: HIGH
category: ac-quality
status: open
depends_on: []
blocks: []
traces_to:
  - .factory/stories/STORY-INDEX.md
  - .factory/stories/dependency-graph.md
---

# ADV-STORIES-004 — The required per-story shards are missing, so acceptance-criteria quality cannot be reviewed at all

- **Severity:** HIGH
- **Category:** ac-quality
- **Location:** `.factory/stories/STORY-INDEX.md` and expected `.factory/stories/stories/` shard set
- **Description:** The index claims all stories are sharded one-per-file, but the referenced story files are not present at the advertised locations. Without the shard files, there is no way to verify token-budget estimates, per-story acceptance criteria, architecture mapping, or whether the stories are actually implementation-ready. So the package fails the Phase 2 review bar on structure alone.
- **Evidence:**
  - STORY-INDEX states: `All stories sharded: one file per story in .factory/stories/stories/`.
  - Attempted reads of representative shards from multiple waves (for example STORY-004, STORY-013, STORY-027, STORY-037, STORY-047, STORY-055) at the documented location returned `ENOENT`.
  - The available aggregate artifacts only expose AC ranges like `AC-001–009`, not the actual AC text required to judge testability.
- **Proposed Fix:** Restore the missing story shard files at the documented path, or correct the index to the real path if the shards were written elsewhere. Until the per-story documents exist and are addressable, treat the decomposition as incomplete and do not advance it as implementation-ready.
