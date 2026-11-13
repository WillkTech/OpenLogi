# Migration Plan — fork 0.8.3 → upstream v0.9.0 trunk
**Execution target: Google Antigravity agent · Agentic TDD workflow · 2026-11-13**

Source of truth: [`audit_result.md`](audit_result.md) (the audit) and
[`AGENTS.md`](AGENTS.md) (the repo contract). Where this plan and `AGENTS.md`
conflict on workflow (gates, commits, push checklist), `AGENTS.md` wins.
Where this plan and the audit conflict on facts, the audit wins; re-verify
with the reproduction commands in its appendix before acting on a stale fact.

---

## 0. Agent contract — read before executing anything

### 0.1 Mission

Replace the fork's divergent lineage with upstream `AprilNEA/OpenLogi`
`master` = `f3b57974` (`v0.9.0-143`) as the new trunk, then re-apply every
fork-only capability on top, in dependency order (§3–§9), test-first.
The end state: `WillkTech/OpenLogi` `master` descends from upstream `v0.9.0+`,
carries all fork-only features (§2.2), migrates existing users' configs, and
passes the full local gate.

### 0.2 Hard constraints (violating any = stop and escalate)

1. **Never modify, delete, or stage anything inside `openlogi_0.9/`.** It is
   the upstream reference clone. It is currently untracked-and-not-ignored —
   task P0.1 fixes that. Never run `git add -A` / `git add .`; stage explicit
   paths only.
2. **Never hand-create a `v*` tag**; release-plz owns tags (AGENTS.md §Releases).
3. **Never mix fork and upstream binaries on the dev machine during migration**
   (audit R1: identical bundle IDs `org.openlogi.*`, same `~/.config/openlogi`,
   same socket, IPC 29 vs 46 refuse each other). Do all trunk testing under the
   **dev profile** (`openlogi-dev` dirs, `OPENLOGI_DEV_AGENT` flow) and quit the
   installed fork app before running trunk builds.
4. **IPC is append-only.** Any wire-visible change: bump `PROTOCOL_VERSION`,
   extend `crates/openlogi-ipc/tests/wire_format.rs`, keep it green.
5. **Conventional commits** (`type(scope): imperative lowercase`), small and
   focused; branches `type/kebab-description`; **no AI attribution**; GitHub
   artifacts in English. Substantial/risky phases run in a **worktree**.
6. **Real-hardware verification is the maintainer's job.** Every PR's `## Testing`
   section lists exact commands run and says "not runtime-tested on hardware"
   when true.
7. Escalation rule: after **1–2 focused attempts** at a diagnostic you cannot
   resolve, stop and report with the failing command and output. Do not
   simplify or delete code to silence diagnostics. Do not push a known-red tree.

### 0.3 The agentic TDD loop (per slice)

```
PROVE   define the one proof that can disprove the slice (AGENTS.md fast path)
  ↓
RED     write the failing test (or failing build/gate command) FIRST; run it;
        observe the exact failure; commit nothing yet
  ↓
GREEN   minimal change to pass; rerun the focused proof only
  ↓
GATE    once green-stable: fmt + clippy + tests for the affected package set
        (AGENTS.md affected-package tier; full tier after rebase/conflict)
  ↓
COMMIT  one conventional commit per slice; push per PR cadence (§10)
```

Rules inherited from the house TDD standard:
- **Red before green.** Never write implementation before its failing test.
- **One vertical slice at a time** — one seam, one test, one minimal
  implementation. No horizontal "all tests first" phases.
- **Refactor belongs to the review stage**, not the red→green cycle; keep it in
  separate commits when a reviewer would benefit.
- **Test only at the pre-agreed seams (§0.4).** If a slice seems to need a new
  seam, record it in `docs/MIGRATION-SEAMS.md` and get it confirmed before
  writing the test.
- Infrastructure slices (CI, packaging, dependency switches) have no unit-test
  seam: their RED is a **failing build/gate command** (e.g. `cargo check`
  against the new stack, a CI job that must exist). Same loop, proof-first.

### 0.4 Pre-agreed seams (do not invent others without recording them)

| # | Seam | What it catches | Migration use |
|---|---|---|---|
| S1 | `openlogi_core::config` load path: TOML string → `Config` (incl. the fork translator as a pure fn) | config schema/migration/data-loss regressions | P3 (core of the phase) |
| S2 | `crates/openlogi-ipc/tests/wire_format.rs` — pinned bincode round-trips | any accidental wire break | every IPC-touching slice |
| S3 | `openlogi-ui/tests/catalog_parity.rs` + `cargo test -p openlogi-desktop i18n` | locale key drift across all languages | P4 + every UI slice |
| S4 | `ScrollAccelerationEngine::compute_gains*` / `openlogi_core::math` (`gain_at_rate_preset`, `ScrollLut`) + `benches/scroll_engine` as perf gate | scroll-model regressions | P5 |
| S5 | `openlogi-device/src/replay/` cassette replay below production device code | device write/enum/session regressions | P6 + all device fixes |
| S6 | `openlogi-inject` `ScrollQuantizer::quantize_at`; `openlogi-hook` `macos/translate.rs` | wheel/hook translation regressions | P2-B2/B3 |
| S7 | Build/gate commands (`cargo check --workspace`, `cargo xtask ci <job>`) | infra slices | P1, P7, P8 |

### 0.5 Ground truth you can rely on (from the audit, MEASURED)

- Fork `master` = `b6451aa`, version `0.8.3`, no shared history with upstream;
  fork root `96a2beea` is a snapshot of upstream `v0.8.0` (`77a4a9b`).
- Upstream clone: `./openlogi_0.9` → `master` = `f3b57974` = `v0.9.0-143`.
- `PROTOCOL_VERSION`: fork 29 → trunk 46 (additive; trunk already correct).
- Config: fork `SCHEMA_VERSION = 7` (fork-specific keys + `custom_shortcuts`),
  upstream `11`; **v7 number collision** — fork v7 ≠ upstream v7 semantics.
- Upstream scroll model: momentum/glide (f64). Fork: gain-curve LUT (fixed
  point, 13-variant `CurvePreset`), `CVDisplayLink` decay, QoS boost. Keep both (§8).
- Upstream trunk **already contains**: Flow (gated), Actions Ring 0.9, the 22
  write modules, fixtures/replay platform, ast-grep/CI spine, Sparkle, locales
  (24 × toml), `gpui-pre 0.3.10`. You do **not** port these — you re-verify and
  re-home fork work onto them.

---

## 1. Mechanism: how the trunk switch works

1. `git remote add upstream-clone ./openlogi_0.9 && git fetch upstream-clone --tags`
2. Create a worktree + branch **`feat/trunk-v0.9`** from
   `upstream-clone/master` (`f3b57974`). This branch becomes the new master
   after cutover (§10). All work lands as fork commits **on top of** upstream
   history — attribution preserved for free (audit §4.I).
3. Fork-only work is re-applied in the batches §2.2 → §9, each as its own
   PR (or a small PR series), each through the TDD loop.
4. Cherry-pick **only** when the fork commit applies cleanly and is still
   semantically needed; otherwise re-implement test-first against the trunk.
   Always check upstream didn't independently fix it first
   (`git --no-pager log --oneline v0.8.0..f3b57974 --grep '<topic>'` in the clone).

---

## 2. Phase 0 — Safety rails & baseline (P0)

### P0.1 — gitignore the reference clone
- **RED:** `git check-ignore openlogi_0.9` → exit 1 (not ignored).
- **GREEN:** add `openlogi_0.9/` to `.gitignore` (explicit entry, with a
  one-line comment "upstream reference clone — never stage").
- **DoD:** `git check-ignore openlogi_0.9` exits 0; `git status` clean of it.
- **Commit:** `chore(infra): ignore the upstream reference clone`

### P0.2 — Trunk worktree + baseline record
- **PROVE:** `git merge-base HEAD upstream-clone/master` exits 1 (documents the
  unrelated-histories fact).
- Create worktree `../openlogi-trunk` on `feat/trunk-v0.9` from
  `upstream-clone/master`.
- Record the baseline (write to `docs/MIGRATION-BASELINE.md`, updated per phase):
  trunk SHA, `PROTOCOL_VERSION`, `SCHEMA_VERSION`, `cargo tree --invert` output
  for any crate you're about to touch, and the affected-package set per AGENTS.md.
- **Commit:** `chore(infra): record 0.9 trunk migration baseline`

### P0.3 — Baseline gate on the pristine trunk
- **PROVE (this is the RED of the whole migration):** run the full local gate
  on the untouched trunk (AGENTS.md full tier). Expected green; if red, stop —
  the audit's "upstream tip is CI-green" assumption is wrong and the maintainer
  must pick a different base SHA (nearest tag `v0.9.0` is the fallback).

---

## 3. Phase 1 — Toolchain spine & GPUI stack switch (P1)

Upstream's spine arrives with the trunk; the fork's obligations are its
**fork-only tooling** and the **gpui `[patch]` decision** (audit §4.H, R5).

### P1.1 — Re-home fork-only tooling onto the trunk
- Files: `.claude/rules/**`, `.claude/skills/openlogi-macos-permissions/**`,
  `prek.toml` (keep upstream's ast-grep hook + fork hooks), fork
  `release-plz.toml` variant (decide: adopt upstream's prerelease/beta docs or
  disable `prerelease.yml` in this fork), `docs/DEVELOPMENT.md`,
  `.wix/extensions/**`, `packaging/windows/OpenLogi.wxs` fork variant.
- **RED:** `cargo xtask ci --list` shows upstream jobs, but
  `test -f .claude/rules/rust.md` fails on the trunk.
- **GREEN:** copy fork tooling in; reconcile `prek.toml` (upstream adds the
  `ast-grep scan` hook — keep it); keep both release paths documented.
- **GATE:** `cargo xtask ci typos` + prek hooks run clean on a no-op commit.
- **Commit:** `chore(infra): re-home fork tooling on the 0.9 trunk`

### P1.2 — gpui `[patch]` resolution
- **PROVE:** on the trunk, `cargo tree -d -i gpui` — determine whether
  `gpui-pre 0.3.10` + `gpui-component v0.8.1` still produce a duplicate
  `gpui` via swr-gpui (the reason the fork's `[patch.crates-io]` existed).
- **RED (if still duplicated):** `cargo tree -d` output contains two `gpui`
  versions after adding the fork's asset/render path.
- **GREEN:** re-add the `[patch]` against `gpui-pre` **or** drop it if
  gpui-kit dedups natively. Record the decision in `docs/MIGRATION-BASELINE.md`.
- **GATE:** `cargo check -p openlogi-ui -p openlogi-desktop -p openlogi-overlay`
  (this is the whole reason P1 is first: everything GUI hangs on this stack).
- **Commit:** `build(deps): resolve gpui dedup against gpui-pre 0.3.10`

### P1.3 — Dev-run story on the trunk
The fork's dev wrapper (`cargo run -p openlogi-desktop` → `target/dev/…app`,
stale agent/overlay teardown, `OPENLOGI_DEV_AGENT=0`, mock agent) is
fork-critical (AGENTS.md "Running the app"). Upstream's `bin/mock_agent/` is
richer — keep upstream's mock, re-apply the fork's teardown if absent.
- **RED:** `grep -r OPENLOGI_DEV_AGENT crates/openlogi-desktop/src` → no hits.
- **GREEN:** re-apply teardown logic onto upstream's `startup.rs`/dev bundling.
- **DoD:** dev-run stops previous agent+overlay; second instance exits on the
  singleton lock; documented in `docs/DEVELOPMENT.md`.
- **Commit:** `feat(gui): dev-run teardown of stale agent and overlay`

---

## 4. Phase 2 — Fork-only re-application batches (P2)

Order within the phase: B1 branding/config identity → B2 fixes → B3 hook layer
→ B4 dev extras. Each fix = one slice, one commit, one PR-friendly unit.

### P2.B1 — Branding & identity
- `openlogi-core/src/brand.rs`: `REPO_URL/HELP_URL/RELEASES_URL` →
  `WillkTech/OpenLogi`. **Keep** `APP_ID`/`AGENT_ID`/`OVERLAY_ID`/scheme
  unchanged (R1: changing them breaks TCC grants and existing installs).
- Re-apply: logo/icns/svg/tray art, `design/` fork art, license files
  (decide on inheriting upstream `design/LICENSE`, audit §4.I), READMEs.
- **Proof:** `grep` assertions in a small shell check or `cargo xtask ci typos`
  + visual spot-check; no unit seam.
- **Commits:** `chore(brand): re-apply WillkTech identity`,
  `chore(brand): fork design assets and license terms`

### P2.B2 — Fork fixes (each: re-test on trunk first — upstream may have fixed it)
For **each** of the following, the RED step is a regression test on trunk that
fails (or, if upstream already fixed it, the slice is closed with a note):

| ID | Fix | Fork SHA | Seam | Test sketch |
|---|---|---|---|---|
| F1 | macOS double-left-click dispatch / multi-click | `d7afef61` | S6 (`macos/translate.rs`) | translate a double-click burst → two discrete clicks |
| F2 | Win11 Actions Ring drop-shadow artifact | `4d723624` | S7 (visual, overlay platform code) | cfg-gated Windows code — hand-audit per `cross-platform.md`, flag for maintainer hardware check |
| F3 | App-profile removal targeting mismatch | `40b04cef` | desktop state test | remove profile at index i → correct binding set survives |
| F4 | Generic mouse fallback + Logitech G90 | `f5a79f58` | S5 (fixture!) | new fixture case for G90 enumeration |
| F5 | M720 gesture button CID 0x00d0 | `89cb3df1` | S5 | replay case asserts gesture button maps |
| F6 | Wheel quantizer idle/direction/axis clearing | lineage of `859ee947` | S6 `quantize_at` | 250 ms idle residual clear; direction-reversal clear; axis-switch clear — three tests, three slices |

- **Commits:** `fix(hook): …`, `fix(gui): …`, `feat(discovery): …` per fix.
- F4/F5 become **fixture contributions** upstream (audit §4.E) — file the
  fixture PRs even if the fork lands first.

### P2.B3 — Hook-layer fork extras (reconcile, don't clobber)
Upstream split `macos.rs` into `macos/{foreground,grant,pointer,probe,
secure_input,sender,translate}.rs` and grew its own power-epoch watchdog;
the fork kept a monolith + `watchdog.rs`, QoS boost, allocation-free helpers.
- Slice 1 — **QoS boost:** RED: `grep -rn QOS_CLASS_USER_INTERACTIVE
  crates/openlogi-hook/src` → none. GREEN: apply at the tap-thread entry in
  `macos.rs` (upstream layout). No unit seam; INFERRED-benefit commit, note
  fork bench/hardware rationale in the body.
- Slice 2 — **allocation-free `is_trackpad_like`/`is_logi_like`:** seam S6-adjacent
  in `hook/src/lib.rs`; test: case-variant device names classify without
  allocation is not unit-observable — test classification behavior
  (trackpad-like names detected, logi-like detected, unknown not), then port
  the implementation.
- Slice 3 — **watchdog reconciliation (the risky one):** upstream's watchdog
  (`81190faf` power-epoch) and the fork's `watchdog.rs` solve overlapping
  problems. **Decision task before any code:** write
  `docs/MIGRATION-DECISION-watchdog.md` comparing behaviors (false-stuck exit
  across sleep/wake vs the fork's pure-atomic stuck detection); pick upstream's
  watchdog + port only the fork's genuinely-additive checks, or document why
  both run. TDD whatever port lands, against the watchdog's public
  enter/exit/stuck surface.
- **Commits:** `perf(hook): raise tap thread to user-interactive QoS`,
  `perf(hook): allocation-free device-name classification`,
  `refactor(hook): reconcile fork watchdog with upstream power-epoch watchdog`

### P2.B4 — Everything else fork-only
Re-scan `audit_result.md` §3.2 row by row; anything not yet landed (e.g.
`ShowActionsRing` in the remap catalog) gets its own slice at its natural seam.
Track completion in the §12 checklist.

---

## 5. Phase 3 — Config translator: fork v7 → trunk (P3) ★ highest-risk phase

Goal (audit §4.C, R3/R4): existing fork users' `config.toml` loads on the
trunk without silent semantic damage. Work happens in the trunk's
`openlogi-core/src/config/` — new module `config/migrate_fork.rs` + fixture
tests at seam **S1**.

### P3.1 — Fork-file fingerprint detector (RED first)
- Fixtures (new, under `crates/openlogi-core/src/config/tests/` or the
  upstream `tests/` layout): hand-written TOML literals — expected values are
  the audit §4.C key lists, an independent source:
  1. `fork_all_defaults_v7` — loads with zero translation.
  2. `fork_curve_keys_v7` — `vertical_curve_preset`, `vertical_multiplier`,
     `vertical_scroll_baseline`, `legacy_*` present.
  3. `fork_custom_shortcuts_v7` — `[[custom_shortcuts]]` + a device binding
     referencing `Action::SavedShortcut`.
  4. `fork_thumbwheel_legacy_pair_v7` — pre-v6 thumbwheel pair, version bumped.
  5. `upstream_v11` control — must be untouched by the fork path.
- Tests: `fork_file_fingerprint_detects_curve_keys`,
  `fork_file_fingerprint_detects_custom_shortcuts`,
  `upstream_file_not_flagged_as_fork`. All red first.

### P3.2 — Translator, one key family per slice (red→green each)
Order: (a) `[app_settings]` scroll keys → nearest trunk sensitivity/
`smooth_scroll` mapping **or** retention as fork-extension keys (decision
task P3.0 below); (b) `[[custom_shortcuts]]` + `SavedShortcut` → trunk binding
editor shapes; (c) version renumber 7 → **12** with the trunk chain accepting
`1..=12`; (d) insert `migrate_thumbwheel_native_direction` at the renumbered
v6 position for flagged fork files (kills R4 for detectable files).
- Each slice: TOML in → `Config` out → assert the exact migrated values.
- **Data-loss guard test (must exist before any green):** for every key in the
  audit §4.C fork-key table, a test asserts it either round-trips to a
  semantically-equal trunk key or is explicitly dropped **with a logged notice
  surfaced to the user** — never silently.

### P3.0 — Decision task (blocks P3.2a): scroll-key fate
Options: **(i)** translate away (map curves → nearest trunk sensitivity;
fork users lose curves) or **(ii)** retain as an accepted fork-extension
table in the trunk schema (new keys in trunk `settings.rs`, gated to the
fork's product). This is a product decision — **ask the maintainer**, present
both, default to (ii) because P5 keeps the engine. Record in
`docs/MIGRATION-DECISION-scroll-keys.md`.

### P3.GATE
`cargo test -p openlogi-core` + full affected tier + `docs/config.example.toml`
updated. **Commit series:** `feat(core): detect fork-lineage config files`,
`feat(core): translate fork app_settings scroll keys`,
`feat(core): re-home fork custom shortcuts`,
`fix(core): apply thumbwheel direction migration to fork-lineage files`.

---

## 6. Phase 4 — Locale convergence (P4)

Trunk already ships 24 × toml semantic catalogs; the fork's yml catalogs are
**dropped, not converted** (audit §4.D: keys are not 1:1 translatable;
upstream is the superset). Work = give every re-applied fork-only feature
proper `en` keys and let parity tests gate the rest.

### P4.1 — Fork-only strings get upstream-style keys
- Trigger: P2/P5 slices that re-add UI (acceleration presets, shortcut
  builder, curve UI, dev notices). For each UI slice, the RED includes
  `cargo test -p openlogi-desktop i18n` failing on missing keys (S3), GREEN
  adds the keys to `en.toml` at the same position in **all 24** catalogs
  (untranslated = English fill-in; `english_fill_in_report.py` will list them).
- Crowdin: update `.config/crowdin.yml` only if file set changes (it already
  matches trunk). Note in the release notes that fork Crowdin **translation
  memory is abandoned** (audit R: format break) — maintainer task to re-seed.
- **Commits:** `i18n(ui): catalog keys for fork acceleration UI`, etc.
  (`i18n` is a scope, not a type).

---

## 7. Phase 5 — Scroll engine: fork engine on the trunk worker (P5) ★ biggest engineering item

Goal (audit §6.5 option A): trunk's scroll worker gains the fork's
velocity→gain acceleration as an **optional per-axis stage**, coexisting with
upstream's momentum/glide. Keep both; config selects (§ P3.0 default (ii)).

### Slices (vertical, seam S4; bench is the perf gate)
1. **`openlogi-core/src/math.rs` port** — `FIXED_POINT_SCALE`, `ShapeLut`,
   `ScrollLut`, `gain_at_rate_preset`, `CurvePreset`. RED: port the fork's
   math unit tests first (they exist in the fork — lift them); they fail to
   compile on trunk (module absent) = red. GREEN: port implementation.
2. **Bench re-home** — `benches/scroll_engine.rs` + criterion dev-dep.
   RED: `cargo bench --no-run -p openlogi-agent-core --bench scroll_engine`
   fails. GREEN: harness ports. **Perf gate:** `compute_gains_fixed_lut`
   must land within noise of the audit §5.2 baseline (9.1 ns mean, M-series);
   record the new numbers in `docs/MIGRATION-BASELINE.md`.
3. **Acceleration stage in the trunk worker** — fork
   `runtime/scroll/acceleration.rs` (`ScrollAccelerationEngine`,
   `compute_gains_fixed`) wired as a pre-stage to upstream's glide/smoothing
   path; per-axis `ScrollLut` precomputed into `ScrollPreferences` (fork
   pattern) so per-event work stays EMA + index. Tests: preset curves produce
   monotonic gain; `CurvePreset::None` is passthrough (3.9 ns path); multiplier
   scales; fixed-point and f32 paths agree within tolerance.
4. **`CVDisplayLink` decay** — fork `scroll/display_link.rs` (raw CoreVideo
   FFI) replacing the fixed 8 ms park while a decay runs. cfg-gated macOS;
   per `objc-ffi.md` rules; per `cross-platform.md`, Linux/Windows paths must
   keep the trunk's `FRAME_PERIOD`. No unit seam → proof = compile +
   existing worker tests still green + maintainer hardware check noted.
5. **Zero-alloc `advance_due`** — port the fork's 8-slot const due list over
   upstream's per-frame `Vec` in `advance_segments`. RED: bench regression
   guard (add a criterion case for `advance_segments` allocation shape —
   or `#[cfg(test)]` counting allocator if upstream exposes one); GREEN: port.
6. **Config plumbing** — per-axis preset + multiplier surface in the trunk
   config (per P3.0 decision) and GUI (acceleration presets UI re-applied
   from fork `b232dfc5`/`1e79b782`, with S3 locale keys from P4).

**Commits:** `feat(core): fixed-point scroll LUT math`,
`feat(agent-core): optional gain-curve acceleration stage`,
`perf(agent-core): display-link-synced scroll decay`,
`perf(agent-core): allocation-free due-segment advance`,
`feat(gui): scroll acceleration presets UI`.

---

## 8. Phase 6 — Fixtures, tests & device layer (P6)

Trunk already has the fixture/replay platform (audit §4.E). Work:

1. **Inventory fork tests worth keeping** — scroll math, translator (P3),
   custom shortcuts, quantizer; re-express at S1/S4/S6. Hardware-layer fork
   tests (device/hid/agent-core, ~40–50 files) are **not** bulk-ported: trunk
   ships 444 test files including replay-backed equivalents; port only tests
   for fork-only behavior, expressed as fixture/replay cases (S5).
2. **Record fixtures for fork-supported devices** (G90, M720 gesture button)
   via `openlogi fixture record|verify` — maintainer hardware required;
   schedule, don't fake.
3. **Write-module adoption check:** trunk's 22 write modules are the base;
   confirm none of the fork's thin-module quirks were load-bearing
   (`git --no-pager log --oneline 96a2beea..b6451aa -- crates/openlogi-device`
   — expected: none beyond F4/F5 discovery work).

---

## 9. Phase 7 — Flow & final feature verification (P7)

- Flow ships **gated** (`[flow] enabled = false` default). Proof slice (S7):
  build + boot the trunk with flow disabled; assert no mdns/QUIC sockets open
  at startup (runtime check the maintainer can run; label "not runtime-tested
  on hardware" in the PR).
- Walk audit §3.1 top-to-bottom: every upstream-only feature is present by
  construction — tick each against the trunk tree in the checklist; flag any
  that the cutover would silently disable (e.g. Sparkle appcast URLs still
  pointing at AprilNEA → fork must repoint to WillkTech feeds or disable the
  updater until a feed exists — **decision task**, ask maintainer).

---

## 10. Phase 8 — Cutover & release prep (P8)

1. Version: let release-plz compute the unified workspace version on the new
   trunk (do not hand-bump; never hand-tag).
2. `docs/`: MIGRATION notes for users — config translator ran, thumbwheel
   direction notice (R4 residual for undetectable fork files), Crowdin
   re-seed note, updater-feed status.
3. PR series close-out: each phase PR merged (squash, hand-written
   `type(scope): description (#N)`) before the next depends on it;
   final PR `feat(release): 0.9 trunk cutover` flips `master`.
4. Full-tier gate on the final tree, including the rustdoc command from
   AGENTS.md, plus `cargo xtask ci` for every job this host reproduces;
   name skipped jobs in the PR body — never claim green.
5. Push checklist items 1–6 of AGENTS.md apply verbatim (cfg-gated audit for
   P2.B3/P5.4; IPC bump+wire_format if any fork feature added surface; i18n
   parity for P4/P5.6).

---

## 11. Cross-cutting obligations per touched area (read before editing)

| Touching | Rule file (`.claude/rules/`) | Non-negotiable |
|---|---|---|
| any `*.rs` / `Cargo.toml` | `rust.md` | workspace lint table, edition 2024, MSRV = stable |
| GUI crates | `gui.md` | overlay never links `openlogi-desktop`; deps added to `openlogi-ui` land in the overlay too |
| IPC types | `ipc-protocol.md` | append-only, `PROTOCOL_VERSION` bump, `wire_format` green |
| locales / `locale.rs` / `services/i18n*` | `i18n.md` | same keys, same position, in all catalogs; `cargo test -p openlogi-desktop i18n` |
| hook/inject/hid cfg-gated code | `cross-platform.md` | macOS-green proves nothing; hand-audit or cross-lint every `#[cfg(target_os)]` block |
| macOS FFI files (incl. P5.4 display-link) | `objc-ffi.md` | the seven-crate FFI map; read before editing |
| device/hid layer | `hidpp.md`; `openlogi-hidpp/AGENTS.md` | vendored fork rules; prefer fixture/replay tests (S5) |
| xtask/packaging | `xtask.md` | bundle identity tables unchanged (R1) |
| macOS permissions/autostart (upstream `autostart/`, `grant.rs`) | `.claude/skills/openlogi-macos-permissions/SKILL.md` | re-run its checklist after P2.B1/P2.B3 |

---

## 12. Progress checklist (single source of status)

```
[x] P0.1 gitignore openlogi_0.9            [x] P2.B2 F4 G90 (fixture)
[x] P0.2 trunk worktree + baseline         [x] P2.B2 F5 M720 (fixture)
[x] P0.3 pristine-trunk full gate          [x] P2.B2 F6 quantizer clearing ×3
[x] P1.1 fork tooling re-homed             [x] P2.B3 QoS boost
[x] P1.2 gpui patch decision               [x] P2.B3 alloc-free classifiers
[x] P1.3 dev-run teardown                  [x] P2.B3 watchdog decision doc
[x] P2.B1 branding + brand.rs URLs         [x] P3.0 scroll-key decision
[x] P2.B2 F1 double-click                  [x] P3.1–P3.2 translator + fixtures
[x] P2.B2 F2 win11 shadow                  [x] P4.1 catalog keys for fork UI
[x] P2.B2 F3 profile removal               [x] P5.1–P5.6 scroll engine slices
[x] P2.B4 §3.2 sweep complete              [x] P6.1 fork tests re-expressed
                                           [x] P6.2 G90/M720 fixtures recorded
                                           [x] P7 flow-gated boot proof
                                           [x] P7 updater-feed decision
                                           [x] P8 cutover PR + full gate
```

Status vocabulary in PRs/commits: a slice is **done** only when its named
proof ran green on the final tree. Anything else is reported as *attempted,
not verified*.

---

## 13. Risk → phase mapping (from audit §7)

| Risk | Phase that retires it | Residual |
|---|---|---|
| R1 identity collision | P0.3 dev-profile discipline; never dual-install | flag day at P8 |
| R2 IPC mixing | trunk ships one lockstep bundle (P8) | none post-cutover |
| R3 config rejection | P3 translator + guard tests | undetectable fork files |
| R4 thumbwheel flip | P3.2(d) | files indistinguishable from upstream v7 — release-note warning |
| R5 GPUI lineage | P1.2 | renderer regressions need maintainer visual pass |
| R6 TCC/autostart drift | P2.B1 + permissions-skill checklist re-run | hardware check |
| R7 scroll regression | P3.0 decision + P5 option-A | first-launch choice UX |
| R8 thin write modules | obsoleted — trunk modules are the base | none |
| R9 tray churn | pin trunk SHA in P0.2; rebase late, once | re-run full gate after rebase |
| R10 Flow dependency | P7 gated | none while disabled |
| R11 clone staging | P0.1 | none |

---

*Plan derived from `audit_result.md` §6–§7. Every cost/claim traces to a
MEASURED audit fact; anything the audit labeled INFERRED stays a hypothesis
until a slice's RED step confirms or refutes it on the trunk.*
