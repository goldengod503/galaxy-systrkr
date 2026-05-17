# GS-01 — Popup gap below panel bar

**Date:** 2026-05-16
**Branch:** `main`

## Summary

The system-stats popup was anchored only ~4px from the panel bar by libcosmic's default `get_popup_settings` offset, causing it to visually overlap the bar's bottom edge. The applet now bumps the positioner offset by an extra 14px along whichever axis the panel is anchored to, leaving a clean gap regardless of panel orientation.

## Scope

**Files Modified**
- `src/app.rs` — mutate `positioner.offset` returned by `get_popup_settings` to push the popup away from the panel along the anchor axis (uses `signum()` so it works for Top/Bottom/Left/Right panels).

**Files Created**
- `docs/release-ledger.md`
- `docs/releases/2026-05-16_GS-01_popup-bar-gap.md`

## Behavioral Impact

- Popup now sits 14px further from the panel bar than the libcosmic default. No other behavior changes.

## Test Plan

- Manual: clicked the applet on a Top-anchored panel; confirmed the popup no longer overlaps the bar and sits with a small visible gap.
- Build: `cargo build` and `just install` succeed; only the pre-existing `unused import: std::path::PathBuf` warning in `src/sampler/gpu/procs/fdinfo.rs` remains.

## Docs Updated

- `docs/release-ledger.md` — initialized; first entry GS-01.
- `docs/releases/2026-05-16_GS-01_popup-bar-gap.md` — this file.

## Rollback Plan

Revert the commit that introduces the offset bump (the commit containing this release doc). The change is isolated to a single `update()` arm in `src/app.rs` and has no schema, config, or persistence impact.

## Open Questions/Decisions

None. Magnitude (14px) was chosen by eye; if a future panel theme makes it look too far, lower `extra_gap` in `src/app.rs`.
