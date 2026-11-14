# GS-05 — Per-process scans only while the popup is open

**Date:** 2026-11-14
**Branch:** `perf/procs-only-when-popup-open` (not yet merged)
**Commit:** `7ea6bae` — `fix(sampler): scan processes only while the popup is open`

## Summary

Each applet instance was using about 14% of a core while idle in the
panel. The cause was the top-process scan: a full sysinfo refresh of
every process on every 500 ms tick, although its output is only shown
in the popup. The scan now runs only while the popup is open.

## Scope

**Files Modified**
- `src/sampler/mod.rs` — `Sampler::tick` takes `with_procs: bool`;
  the CPU and GPU process scans moved into `Sampler::top_procs`;
  `TOP_N` constant replaces the two literal `5`s.
- `src/app.rs` — `Message::Tick` passes `popup_id.is_some()`;
  `TogglePopup` scans once when the popup opens.
- `docs/ARCHITECTURE.md` — new intentional-decision entry, corrected
  the "well under 1 ms" sampling claim, refreshed shifted line refs.
- `docs/release-ledger.md` — GS-05 row.

**Files Created**
- `docs/releases/2026-11-14_GS-05_procs-only-when-popup-open.md` — this file.

## Behavioral Impact

- Popup closed: no per-process scanning (CPU via sysinfo, GPU via
  NVML or fdinfo). Sparklines and all other readings are unchanged.
- Popup open: same scanning as before, every tick.
- On open, the lists are filled from an immediate scan. Usage figures
  are deltas against the previous scan, so that first reading is an
  average since the popup was last open; it is replaced by an accurate
  one on the next tick (500 ms at the default refresh). On the very
  first open after start the CPU list is empty until that next tick.

## Test Plan

- Benchmark (standalone, sysinfo 0.32.1, 500 ms tick, 16 cores, 562
  processes / 2,880 tasks): CPU + memory + temperatures 0.4% of a
  core; all-process refresh 12.9%. The running applet measured 13.7%
  and 13.9% per instance before the change.
- Tests: `cargo test --all-features` — **41 passed, 0 failed** (2 new
  in `sampler::aggregator_tests`: skipped ticks return empty process
  lists; a scan after skipped ticks still reports a busy process).
- Clippy: `cargo clippy --all-features -- -D warnings` clean.
- Build: `cargo check --no-default-features` and
  `cargo build --release` clean.
- **Not yet done:** install and measure the live applet with the popup
  closed and open. Held at Peter's request on 2026-11-14.

## Docs Updated

- `docs/ARCHITECTURE.md` — see Scope. No SHA bump: no module
  boundaries changed.
- `docs/release-ledger.md` — GS-05 row.

## Rollback Plan

```bash
git revert 7ea6bae
```

No config schema change.

## Open Questions / Decisions

- With the popup open the scan still costs about 66 ms of CPU per tick
  on the UI thread. Wall-clock time was not measured; if the popup
  feels janky, the reopen trigger in ARCHITECTURE.md ("Synchronous
  sampling on the UI thread") applies.
