# GS-04 — Frosted-glass popup (follow COSMIC theme)

**Date:** 2026-10-02
**Branch:** `feat/frosted-popup` (merged to `main`)
**Commit:** `0e9d6cf` — `feat(popup): frost popup when COSMIC "frosted applets" is enabled`

## Summary

The popup now renders frosted (blurred, translucent) when COSMIC's
**Settings → Appearance → frosted applets** option is on, and stays
opaque when it is off — the same behavior as the stock COSMIC applets.
No new applet setting or config key.

Achieved by bumping `libcosmic` from `1729153` (2026-04-23) to upstream
HEAD `5a8bd94`. The newer libcosmic reads the theme's per-surface
`frosted_*` flags and enables blur on applet popups automatically via
`ext-background-effect-v1`; the old pin only exposed a raw blur action
and ignored those flags.

## Scope

**Files Modified**
- `Cargo.lock` — `libcosmic` `1729153` → `5a8bd94` (plus transitive
  updates pulled by the bump: pop-os winit, zbus, cosmic-panel config).
- `src/app.rs` — `Message::TogglePopup` adapted to the new surface API:
  surface actions are now `cosmic::Action::Surface(..)` (was
  `cosmic::Action::Cosmic(cosmic::app::Action::Surface(..))`), and
  `app_popup` takes a leading `LiveSettings` closure, passed as
  `|_| Default::default()` so blur is theme-driven.
- `docs/ARCHITECTURE.md` — new intentional-decision entry for
  theme-driven frosting.
- `docs/release-ledger.md` — GS-04 row.

**Files Created**
- `docs/releases/2026-10-02_GS-04_frosted-popup.md` — this file.

## Behavioral Impact

- Frosted applets **on**: popup background is blurred and translucent.
- Frosted applets **off** (the default): no visible change.
- The panel button is drawn inside cosmic-panel's surface and follows
  the separate "frosted panel" option, not this change.
- Requires a compositor advertising `ext_background_effect_manager_v1`;
  the installed `cosmic-comp` (`0fbd457`) does. On compositors without
  it the popup stays opaque.

## Test Plan

- Build: `cargo check --all-features` and
  `cargo check --no-default-features` clean.
- Tests: `cargo test --all-features` — **39 passed, 0 failed**.
- Clippy: `just clippy` (`-D warnings`) clean.
- Manual: `just install`, restarted cosmic-panel (one applet instance
  per output, DP-2 and DP-3), toggled Settings → Appearance → frosted
  applets — popup blurs when on, opaque when off; sparkline and
  threshold colors legible. Confirmed by Peter.

## Docs Updated

- `docs/ARCHITECTURE.md` — intentional-decision entry. No SHA bump: no
  module boundaries or public symbols changed.
- `docs/release-ledger.md` — GS-04 row.

## Rollback Plan

```bash
git revert 0e9d6cf
```

Restores the previous `Cargo.lock` pin and `TogglePopup` code together;
no config schema change.

## Open Questions / Decisions

- Bumped to upstream HEAD rather than the cached `8a017a1` (2026-08-07)
  to pick up the latest fixes; both expose the same auto-blur behavior.
