# GS-06 — README screenshot refresh

**Date:** 2026-11-14
**Branch:** `main`

## Summary

The README screenshot was out of date: it showed only the CPU, GPU and RAM sparklines and the pre-frosting popup. It is replaced with a current capture showing the CPU, GPU, RAM and NET sparklines and the frosted popup.

## Scope

**Files Modified**
- `assets/galaxy-trkr-screenshot.png` — replaced with the new capture (639x969 → 670x960).
- `README.md` — `<img>` width 511 → 536, keeping the same 80% display scale for the wider image.
- `docs/release-ledger.md` — GS-06 entry.

**Files Created**
- `docs/releases/2026-11-14_GS-06_readme-screenshot.md`

## Behavioral Impact

No behavior change. Documentation and image asset only.

## Test Plan

- `just test` — 41 passed, 0 failed (no code touched; run as the pre-commit gate).
- No tests added.

## Docs Updated

- `README.md` — screenshot width.
- `docs/release-ledger.md` — GS-06 entry.
- `docs/releases/2026-11-14_GS-06_readme-screenshot.md` — this file.

## Rollback Plan

`git revert b3d7857` restores the previous screenshot and width.

## Open Questions/Decisions

None.
