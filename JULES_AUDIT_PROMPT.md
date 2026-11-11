# Jules Task — Feature & performance audit: fork `master` (0.8.3) vs upstream `v0.9.0+` (local clone `openlogi_0.9/`)

## Prompt (paste to Jules)

You are performing a **read-only feature and performance audit** in this repository
(WillkTech/OpenLogi, default branch `master`). You will not modify any source file.
Your single deliverable is a new file, **`docs/audit-features-perf-0.9.md`**, opened as
a pull request. The audit answers two questions for the maintainer:

1. **Features** — what can upstream AprilNEA/OpenLogi (now at **v0.9.0 + 143 commits**)
   do that this fork (still versioned **0.8.3**) cannot, and vice versa? Which
   upstream feature clusters are worth adopting, at what integration cost?
2. **Performance** — where do the two lineages differ in runtime behavior: hot paths
   (input hook, scroll engine, IPC, device I/O, overlay render), startup, memory,
   dependency drift, and the ~7 upstream `perf` commits since `v0.8.3`?

This is **not** a re-run of the earlier divergence audit (`0.8.3 → v0.8.15`); that
work is input (see below). Upstream has moved six more releases past it.

## Ground truth you can rely on (verified 2026-11-12)

| Source | What it is | Status |
|---|---|---|
| `master` | fork tip = `78e7ec6`, workspace version `0.8.3`, 16 commits past fork tag `0.8.3` | HEAD |
| tag `0.8.3` | **fork-side** tag, commit `859ee94`, ancestor of `master` | ⚠ NOT the same commit or tree as upstream `v0.8.3` |
| `openlogi_0.9/` | full git clone of AprilNEA/OpenLogi, branch `master` = `f3b57974` = `v0.9.0-143` | canonical upstream, **complete history available locally — no network needed** |
| upstream tag `v0.8.3` | commit `4d0936d`, tree differs from fork tag `0.8.3` | in the clone |
| `docs/OpenLogi-0.8.15-vs-main-diff.md` | prior tree-diff line list vs the v0.8.15 snapshot | cross-check only; upstream has since shipped 0.8.16–0.8.21-beta.58 and 0.9.0 |

⚠ **Headline correction:** the fork's tag `0.8.3` and upstream's `v0.8.3` are different
commits with different trees. Do not assume the fork point is `v0.8.3` — compute the
true merge-base (Setup step 2) and report it. If it is not `v0.8.3`, say which upstream
commit the fork actually diverged from.

⚠ `openlogi_0.9/` is **untracked and not gitignored** (`?? openlogi_0.9/`). Never run
`git add -A` / `git add .`; your commit must contain only the deliverable file. Do not
delete or modify anything inside the clone.

## Setup

```sh
# 1. Make upstream objects available locally from the clone (no network):
git remote add upstream-clone ./openlogi_0.9
git fetch upstream-clone --tags

# 2. Find the TRUE fork point (do not trust either 0.8.3 tag):
git merge-base master upstream-clone/master        # report this SHA + nearest upstream tag
git rev-parse 0.8.3^{tree} v0.8.3^{tree}           # show the tag mismatch yourself

# 3. Sanity-check the clone is where the table says it is:
git -C openlogi_0.9 rev-parse HEAD                 # f3b57974…
git -C openlogi_0.9 describe --tags                # v0.9.0-143-gf3b57974
```

If the fetch from the local path fails, fall back to tree-level diffing against
`openlogi_0.9/` (exclude `.git`, `target`, `.DS_Store`) and label all commit-level
claims as inferences.

## Method

1. **Upstream evolution (merge-base → `upstream-clone/master`)** — ~946+ commits,
   `v0.8.3`→`v0.9.0`→tip. Read `openlogi_0.9/CHANGELOG.md` sections 0.8.4 … 0.9.0 for
   the curated feature list; use `git log`/`git diff --stat` for attribution.
2. **Fork-only work (merge-base → `master`)** — 16 commits: branding, `.claude/rules/**`,
   `prek.toml`, `release-plz.toml`, the prior audit artifacts, refactors/fixes.
3. **Feature inventory** — build the per-feature matrix (Deliverable §3). A feature is
   "present in a lineage" only if you found its implementation, not just a changelog line.
4. **Performance assessment** (§5 below) — commit triage + hot-path reading + any
   measurements you can run read-only.
5. **Spot-read actual diffs** (not `--stat`) for every file in the wire-format,
   config-schema, and platform-FFI areas.

## Focus areas — answer each explicitly

### A. Feature inventory delta (the core deliverable)
Known upstream-only feature clusters to verify and expand into a complete matrix:
- **Flow** — new crate `openlogi-flow` (`discovery`, `transport`, `session`, `pairing`,
  `sas`, QUIC keep-alive fix in 0.9.0): cross-machine control. The fork has nothing.
  Scope what ships: UI surfaces (`flow inspector` in GUI), IPC surface, identity/pairing.
- **Actions Ring 0.9** — floating translucent slots, tailed labels, per-device ring
  size, direction aiming, slot selection on held-trigger release.
- **Camera** — both lineages have `openlogi-camera`; upstream 0.9 fixed reset-to-defaults
  and profile persistence on macOS. Establish parity per feature.
- **Config features** — per-app / per-device profiles, gestures, wheel-hold,
  button-map, function keys (upstream split `config.rs` into a `config/` module).
- **Device support** — registry additions (`camera`, `headset`), write modules
  (`dpi`, `easy_switch`, `lighting`, `litra`, `os_layout`, `report_rate`, `smartshift`),
  Unifying slot naming, DPI-read fix (#2321).
- **Platform** — upstream hook splits `macos/linux/windows` (`foreground`, `pointer`,
  `grant`, `sender`, `translate`, `supervisor`), `openlogi-inject` `uinput`/
  `space_switch`, agent `lifecycle`/`shutdown`/`startup`/`resume_linux`.
  Fork-only: `openlogi-hook/src/windows.rs`, `openlogi-agent/src/launch_agent.rs`,
  `openlogi-camera/src/{capture,uvc}.rs`. Flag every rename/relocation pair.
- **Fork-only features** — enumerate honestly (branding, rules, tooling, any fixes
  upstream lacks).

### B. IPC protocol (highest risk)
`PROTOCOL_VERSION` is **29 on the fork, 46 on upstream** (`crates/openlogi-ipc/src/ipc.rs`
in each tree). Enumerate every wire-visible type change across those 17 versions, which
are append-only, and whether the fork's GUI/agent/overlay trio could mix with upstream
binaries mid-integration.

### C. Config schema & user data
Fork: single `crates/openlogi-core/src/config.rs`. Upstream: `config/` module with
`per_app.rs`, `per_device.rs`, `migrate.rs`, `revision.rs`, `button_map.rs`,
`function_key.rs`, `gestures.rs`, `wheel_hold.rs` + `config/tests/`. Assess TOML on-disk
compatibility, whether upstream's `migrate.rs` would accept a fork-written config
unchanged, and data-loss risk in each merge direction.

### D. Locale system
Fork: `openlogi-ui/locales/*.yml` (23 languages) + `openlogi-ui/src/locale.rs`.
Upstream: `*.toml` (confirmed: `openlogi-ui/locales/*.toml`) with locale logic in
`openlogi-core`. State which convergence direction is cheaper and what breaks
(Crowdin config, `merge_crowdin_download.py`, i18n CI job, `services/i18n/`).

### E. Test & fixture infrastructure
Upstream-only: `fixtures/devices/`, crate `openlogi-fixture`, replay system
(`openlogi-device/src/replay*`, `openlogi-hid/src/recording*`, mock/observation/pending
channels), `openlogi-cli` `fixture`/`api` commands, `docs/MOCK_DEVICE_TESTING.md`.
Fork-only: `crates/openlogi-agent-core/benches/`. Estimate how much of the fork's test
surface rewrites onto the fixture system.

### F. HID++ layer
`crates/openlogi-hidpp` (54 files differed at 0.8.15; re-measure at 0.9) plus
`openlogi-hidpp-derive` — **both trees now have this crate**: diff them and state
whether the fork's copy is an early copy of upstream's, a divergent port, or unrelated.
Classify upstream protocol fixes vs new feature support (`lock_key_state`,
`adc_measurement`, `pointer_motion_scaling`, `device_information`, …). Read
`openlogi_0.9/crates/openlogi-hidpp/AGENTS.md` for upstream's own fork rules.

### G. Performance (co-equal with features)
1. **Perf-commit triage** — `git log --oneline v0.8.3..HEAD | grep -i perf` (7 commits
   upstream): read each diff, state what it optimizes, whether the fork has the same
   hot-path shape, and the win the fork would get.
2. **Benchmarks** — the fork has `cargo bench -p openlogi-agent-core --bench
   scroll_engine`; run it and report numbers. Upstream has **no** benches. A fair
   A/B requires porting the criterion harness into an upstream scratch worktree —
   allowed **only outside the repo tree and never committed**; if you do not run it,
   compare the two `scroll_engine` implementations by reading and say so.
3. **Hot-path reading** — hook event-tap callback (CGEventTap/evdev/WH_MOUSE_LL),
   scroll dispatch, DPI cycle, IPC round-trip (bincode payload sizes if cheap to
   measure), device probe/inventory caching, overlay frame loop, agent startup.
4. **Dependency drift** — diff `Cargo.lock`: versions of `async-hid`, tokio, GPUI /
   gpui-component, rustls/quic (Flow), anything with runtime-performance implications.
5. **Size/startup** — optional: `cargo build --release` both trees, compare binary and
   .app bundle sizes and cold-start behavior. Label measurements with machine + commit.
6. State clearly what is **measured** vs **read-and-inferred**.

### H. CI, release & toolchain
Upstream-only: ast-grep lint infra (`.ast-grep/`, `sgconfig.yml`, `xtask` lint
commands), Sparkle/appcast in `xtask`, `stale-allows.yml`, flatpak + Windows
installation JSON, per-crate `AGENTS.md`. Fork-only: `.claude/rules/**`, `prek.toml`,
`release-plz.toml`, `.wix/`, `design/` branding, desktop assets. Diff
`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` (MSRV both sides), lint tables,
`devenv.*`/`flake.*`; explain every meaningful delta.

### I. Licensing & attribution
Dual MIT/Apache-2.0; `design/` proprietary; upstream adds `design/LICENSE`. Confirm no
SPDX/header conflicts; note attribution implications of adopting upstream features
(commit history is available locally, so credit is easy to preserve).

## Deliverable — `docs/audit-features-perf-0.9.md`

1. **Executive summary** — top 10 findings, each with evidence (SHA / path / command).
2. **Topology** — true merge-base (with the 0.8.3-tag-mismatch explanation), commit
   counts both directions, files changed both sides (conflict hot-zones, computed not
   estimated).
3. **Feature matrix** — one row per user-visible feature: fork status (present /
   missing / diverged), upstream status, evidence, adoption cost (S/M/L), recommendation.
4. **Focus-area answers** (A–I).
5. **Performance report** — perf-commit table, benchmark numbers (or explicit
   "not measured"), hot-path comparison, dependency-drift table.
6. **Adoption plan input** — per contested cluster (Flow, Actions Ring 0.9, config
   split, locales, fixtures, hook/camera naming, CI/lint stack): both options,
   switching cost, recommendation. Propose; do not execute.
7. **Risk register** — IPC 29→46 mixing, user-config migration, TCC/permission
   identity, bundle IDs / helper-launch tables, anything that could silently break the
   three-process bundle (GUI/agent/overlay).
8. **Not-verified list** — everything not confirmable from the two trees alone.

## Constraints

- Audit only: the deliverable file is the **only** file you may create or commit.
  No source edits, no refactors, no dependency changes, no commits touching
  `openlogi_0.9/`.
- Cite evidence for every claim (SHA, path, command output). Label inferences as
  inferences; list what you could not verify rather than guessing.
- Benchmarks/builds may write to `target/` only; scratch worktrees for A/B benches go
  outside the repo and are discarded.
- English throughout; no AI-attribution lines or co-author trailers.
- Commit message: `docs(audit): feature and performance audit vs upstream v0.9.0`.
- Where the clone and the GitHub tag disagree, trust the tag and flag it — but the
  clone is a fresh full fetch, so this should not arise.

---

## Before launching Jules (maintainer checklist — not part of the prompt)

- [ ] `openlogi_0.9/` clone present and at `f3b57974` (`v0.9.0-143`) ✓ verified
      2026-11-12; consider `git -C openlogi_0.9 pull` for a fresher tip.
- [ ] Decide whether to gitignore `openlogi_0.9/` before Jules runs (it is currently
      untracked-but-not-ignored, so a careless `git add -A` would commit ~1,200 files).
- [ ] Prior artifacts committed: `docs/OpenLogi-0.8.15-vs-main-diff.md` ✓; the
      `OpenLogi-0.8.15/` snapshot was removed in `78e7ec6` — the clone supersedes it.
- [ ] Confirm fork facts Jules will restate: version 0.8.3, `PROTOCOL_VERSION = 29`,
      yml locales, `scroll_engine` bench, fork tag `0.8.3` = `859ee94`.
- [ ] Start Jules on a fresh branch off `master`; it should open the audit as a PR.
