---
document_type: verification-property-index
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [specs/prd.md, specs/architecture/ARCH-INDEX.md]
traces_to: specs/architecture/ARCH-INDEX.md
---

# Verification Property Index

## Overview

This index catalogs all 15 verification properties for Forge MCP. Each VP traces to one or more behavioral contracts (BC) from the L3 PRD and specifies a formal proof method, target module, and feasibility assessment.

## Property Registry

| VP ID | File | Title | Module | Proof Method | Feasibility | Status | Source BC |
|-------|------|-------|--------|-------------|-------------|--------|----------|
| VP-001 | vp-001-jsonrpc-parse-no-panic.md | JSON-RPC Message Parsing Never Panics | forge-core | fuzz | feasible | draft | BC-2.04.001 |
| VP-002 | vp-002-error-classification.md | Error Classification Correctness | forge-core | kani | feasible | draft | BC-2.05.010 |
| VP-003 | vp-003-pagination-terminates.md | Pagination State Machine Termination | forge-core | kani | feasible | draft | BC-2.05.001 |
| VP-004 | vp-004-config-parse-no-panic.md | Config Parsing Never Panics | forge-discovery | fuzz | feasible | draft | BC-1.01.002 |
| VP-005 | vp-005-ring-buffer-bounds.md | Ring Buffer Bounds and FIFO Ordering | forge-traffic | kani | feasible | draft | BC-4.09.003 |
| VP-006 | vp-006-capture-content-integrity.md | Message Capture Preserves Content | forge-traffic | proptest | feasible | draft | BC-4.09.001 |
| VP-007 | vp-007-alert-state-machine.md | Alert State Machine Transitions | forge-health | kani | feasible | draft | BC-6.14.002 |
| VP-008 | vp-008-histogram-correctness.md | Latency Histogram Mathematical Correctness | forge-health | proptest | feasible | draft | BC-6.13.001 |
| VP-009 | vp-009-ip-classification.md | IP Classification Correctness | forge-security | kani | feasible | draft | BC-7.16.002 |
| VP-010 | vp-010-confidence-bounds.md | Confidence Score Bounds | forge-security | kani | feasible | draft | BC-7.16.004 |
| VP-011 | vp-011-schema-drift-detection.md | Schema Drift Detection Completeness | forge-security | proptest | feasible | draft | BC-7.16.003 |
| VP-012 | vp-012-tui-state-machine.md | TUI State Machine No Invalid States | forge-tui | proptest | feasible | draft | BC-3.07.001 |
| VP-013 | vp-013-connection-state-machine.md | Connection State Machine Validity | forge-core | kani | feasible | draft | BC-1.02.003 |
| VP-014 | vp-014-filter-preserves-ordering.md | Traffic Filter Preserves Message Ordering | forge-traffic | proptest | feasible | draft | BC-4.10.001 |
| VP-015 | vp-015-daemon-session-multiplexing.md | Daemon Session-Pool Multiplexing Correctness | forge-daemon | proptest | feasible | draft | BC-1.03.001 |

## Summary by Proof Method

| Method | Count | VPs |
|--------|-------|-----|
| Kani (model checking) | 7 | VP-002, VP-003, VP-005, VP-007, VP-009, VP-010, VP-013 |
| Proptest (property-based testing) | 6 | VP-006, VP-008, VP-011, VP-012, VP-014, VP-015 |
| Fuzz (cargo-fuzz) | 2 | VP-001, VP-004 |

## Summary by Module

| Module | Count | VPs |
|--------|-------|-----|
| forge-core | 4 | VP-001, VP-002, VP-003, VP-013 |
| forge-traffic | 3 | VP-005, VP-006, VP-014 |
| forge-security | 3 | VP-009, VP-010, VP-011 |
| forge-health | 2 | VP-007, VP-008 |
| forge-discovery | 1 | VP-004 |
| forge-tui | 1 | VP-012 |
| forge-daemon | 1 | VP-015 |

## Feasibility Summary

All 15 VPs are assessed as **feasible**. No properties require further research before proof harness construction.
