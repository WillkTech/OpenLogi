# Jules Task — Audit: fork baseline `0.8.3` vs upstream `v0.8.15`, as input to an integration plan

## Prompt (paste to Jules)

You are performing a **read-only divergence audit** in this repository
(WillkTech/OpenLogi, default branch `master`). You will not modify any source file.
Your single deliverable is a new file, **`docs/audit-0.8.3-to-0.8.15.md`**, opened as a
pull request. The audit's purpose is to be the **evidence base for an integration plan**
that merges the best of two lineages:

- **The fork lineage** — this repo, forked from upstream at tag `0.8.3`, with
  fork-only work on `master` since then (branding, `.claude/rules/**`, `prek.toml`,
  `release-plz.toml`, refactors, fixes).
- **The upstream lineage** — AprilNEA/OpenLogi, which evolved from `0.8.3` to
  **`v0.8.15`**. The full v0.8.15 source tree is committed in this repo at
  **`OpenLogi-0.8.15/`** (treat it as canonical ground truth even if you cannot fetch
  from the network).

## Ground truth you can rely on

| Source | What it is | Status |
|---|---|---|
| tag `0.8.3` | the fork point, present on origin | verified ancestor of `master` |
| `master` | current fork tip (all fork-only work) | HEAD |
| `OpenLogi-0.8.15/` | committed copy of upstream v0.8.15 source tree | canonical upstream snapshot |

## Setup

```sh
git fetch origin --tags
git merge-base 0.8.3 master        # must print 0.8.3's SHA — confirm, and report it
# Try to get real upstream history for commit-level attribution:
git remote add upstream https://github.com/AprilNEA/OpenLogi.git
git fetch upstream tag v0.8.15
```

If the upstream fetch fails (no network), **fall back to tree-level diffing against
`OpenLogi-0.8.15/`** and say so in the report — commit-level claims become inferences
from file content in that case. If `0.8.3` is not the merge-base, that is itself a
headline finding; locate the true merge-base before proceeding.

## Method

1. **Upstream evolution (`0.8.3` → `v0.8.15`)** — what the fork is missing:
   `git log --oneline 0.8.3..v0.8.15` and `git diff --stat 0.8.3 v0.8.15` if history is
   available; otherwise `diff -rq` of the tracked tree vs `OpenLogi-0.8.15/` (exclude
   `.git`, `target`, the snapshot dir itself, `.DS_Store`).
2. **Fork-only work (`0.8.3` → `master`)** — what upstream doesn't have:
   `git log --oneline 0.8.3..master` and `git diff --stat 0.8.3 master`.
3. **Conflict hot-zones** — compute the **intersection**: files changed on both sides
   (touched between `0.8.3`→`v0.8.15` AND `0.8.3`→`master`). These predict merge
   conflicts; the per-file list is the most actionable output of this audit.
4. **Per-crate change matrix** — file counts and nature of changes for every crate.
   A prior local tree-diff (snapshot vs fork `master`) found
   **432 differing files · 58 fork-only · 256 snapshot-only**, distributed roughly:

   | Area | Differing | Area | Differing |
   |---|---|---|---|
   | openlogi-desktop | 88 | openlogi-hid | 8 |
   | openlogi-hidpp | 54 | openlogi-camera | 7 |
   | openlogi-device | 40 | openlogi-overlay | 6 |
   | openlogi-core | 38 | openlogi-inject | 6 |
   | openlogi-agent-core | 34 | openlogi-permissions | 5 |
   | openlogi-agent | 17 | openlogi-ipc | 5 |
   | openlogi-cli | 13 | openlogi-ui | 3 |
   | xtask | 27 | openlogi-assets | 3 |
   | .github | 13 | openlogi-device-registry | 2 |
   | design | 12 | openlogi (CLI bin) | 1 |
   | docs | 11 | packaging | 6 |
   | .claude | 9 | .config / .cargo | 3 / 2 |

   Verify these numbers yourself; do not trust them blindly.
5. **Spot-read the actual diffs** (not just `--stat`) for every file in the
   wire-format, config-schema, and platform-FFI areas listed below.

## Focus areas — answer each explicitly

### A. IPC protocol (highest risk)
`crates/openlogi-ipc/src/ipc.rs` and its tests. The wire format is versioned and
**append-only**. Did upstream bump `PROTOCOL_VERSION`, break it, or add fields?
Does the fork's agent/GUI/overlay trio still speak the same dialect after a merge?
List every wire-visible type change.

### B. Config schema & user data
Fork: `crates/openlogi-core/src/config.rs` (single file). Upstream: split into
`config/` with `per_app.rs`, `per_device.rs`, `migrate.rs`, `revision.rs`,
`button_map.rs`, `function_key.rs`, `gestures.rs`, `wheel_hold.rs`, plus a
`config/tests/` tree. Assess: TOML on-disk compatibility, whether upstream added a
migration path from the fork's schema, and what happens to existing user configs
after integration.

### C. Locale system — a genuine fork in the road
Fork: `crates/openlogi-ui/locales/*.yml` (23 languages) + `openlogi-ui/src/locale.rs`.
Upstream: migrated everything to `*.toml` and moved locale logic into
`openlogi-core/src/locale.rs`. Determine which direction is cheaper to converge and
what breaks (Crowdin config, `merge_crowdin_download.py`, i18n CI job,
`services/i18n/`).

### D. Test & fixture infrastructure (biggest upstream win)
Upstream-only: `fixtures/`, `crates/openlogi-fixture`, the replay system
(`openlogi-device/src/replay*`, `openlogi-hid/src/recording*`,
`openlogi-hidpp/src/channel/{mock,observation,pending}.rs`),
`openlogi-cli` `fixture`/`api` commands, `docs/MOCK_DEVICE_TESTING.md`.
Fork-only: `crates/openlogi-agent-core/benches/`. Evaluate how much of the fork's
current test surface would need rewriting on top of the fixture system.

### E. Vendored HID++ fork
`crates/openlogi-hidpp` (54 differing files, 0BSD, hard fork of the `hidpp` crate).
Classify upstream changes: protocol fixes, new feature support
(`lock_key_state`, `adc_measurement`, `pointer_motion_scaling`, `multi_platform`,
`device_information`), test scaffolding. Check `OpenLogi-0.8.15/crates/openlogi-hidpp/AGENTS.md`
for upstream's own rules about the fork.

### F. Device layer & registry
Upstream additions: `openlogi-device/src/{device_io,host_lock,replay}.rs`,
`inventory/{channel_cache,events,probe}/`, `session/{capture,capture_restore,restore}.rs`,
`write/{dpi,easy_switch,lighting,litra,os_layout,report_rate,smartshift}/`,
`openlogi-device-registry/src/{camera,headset}.rs`. Map these against the fork's
`inventory/generic.rs` and any fork-specific device handling.

### G. Platform / cfg-gated code
The macOS FFI surface spans seven crates. Upstream added `openlogi-hook/src/{macos,linux,windows}/`
splits (`foreground`, `pointer`, `grant`, `sender`, `translate`, `supervisor`),
`openlogi-inject/src/{uinput,inject/macos,inject/space_switch}.rs`,
`openlogi-agent/src/{activity_macos,autostart,lifecycle,shutdown,startup,resume_linux}.rs`.
Fork added `openlogi-hook/src/windows.rs`, `openlogi-agent/src/launch_agent.rs`,
`openlogi-camera/src/{capture,uvc}.rs` (upstream names these
`capture_macos`/`uvc_macos`). Flag every rename/relocation pair — these will produce
add/add conflicts.

### H. CI, release & toolchain
- Upstream: ast-grep lint infra (`.ast-grep/`, `sgconfig.yml`, `skills-lock.json`,
  `xtask/src/commands/lint*`), Sparkle/appcast in `xtask` (`sparkle/`, `appcast/`,
  `update_feed/`), `stale-allows.yml` workflow, `packaging/linux/flatpak`,
  `packaging/windows/openlogi-installation.json`, per-crate `AGENTS.md`/`CLAUDE.md`.
- Fork: its own `.claude/rules/**` (including `hidpp.md`, `hook.md` which upstream
  lacks), `prek.toml`, `release-plz.toml` contract, `.wix/`, `design/icon/*.svg`
  branding, `crates/openlogi-desktop/{assets,icon,icon/previews}`.
- Diff `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `clippy.toml`, `rustfmt.toml`,
  `devenv.*`, `flake.*`, `prek.toml`, `release-plz.toml` and explain every meaningful
  delta (MSRV, lint table, dependency additions).

### I. Licensing & attribution
Project is dual MIT/Apache-2.0; `design/` assets are proprietary; upstream adds
`design/LICENSE`. Confirm no license headers/SPDX markers conflict, and note any
authorship/attribution implications of a merge direction choice.

## Deliverable — `docs/audit-0.8.3-to-0.8.15.md`

1. **Executive summary** — top 10 findings, each with evidence (commit SHA / file path).
2. **Merge-base & divergence topology** — the actual graph, with numbers from the
   Method section (both directions: upstream-only, fork-only, both-sides).
3. **Per-crate change matrix** — for each crate: upstream-only changes, fork-only
   changes, and **conflict hot-zones** (files changed on both sides — compute the
   intersection; do not estimate).
4. **Focus-area answers** (A–I above).
5. **Mergeability assessment** per area: clean cherry-pick / mechanical merge /
   architectural decision required.
6. **Integration plan input** — for each contested decision (locales yml vs toml,
   config single-file vs split, hook/camera file naming, which CI/lint stack wins,
   merge direction: upstream-into-fork vs fork-rebase-onto-upstream), state both
   options, the switching cost, and a recommendation with rationale.
   Do **not** execute the plan; propose it.
7. **Risk register** — wire-format breaks, user-config migration, TCC/permission
   identity changes, packaging identity (bundle IDs, helper-launch tables), anything
   that could silently break the three-process bundle (GUI/agent/overlay).
8. **Not-verified list** — everything you could not confirm from the repo alone.

## Constraints

- Audit only: the deliverable file is the **only** file you may create. No source
  edits, no refactors, no dependency changes.
- Cite evidence for every claim (SHA, path, command output). Label inferences as
  inferences; list what you could not verify rather than guessing.
- English throughout; no AI-attribution lines or co-author trailers in the commit.
- Commit message: `docs(audit): divergence audit 0.8.3 → upstream v0.8.15`.
- The v0.8.15 GitHub tag is canonical; `OpenLogi-0.8.15/` in this repo is a committed
  copy of it — if they ever disagree, trust the tag and flag the discrepancy.

---

## Before launching Jules (maintainer checklist — not part of the prompt)

- [ ] `OpenLogi-0.8.15/` is committed and pushed ✓ (1,168 tracked files).
- [ ] Tag `0.8.3` is on origin ✓ (verified: `859ee94`, and it is the merge-base with `master`).
- [ ] Optional: commit `docs/OpenLogi-0.8.15-vs-main-diff.md` (the local tree-diff list)
      so Jules can cross-check its counts instead of regenerating them.
- [ ] Start Jules on a fresh branch off `master`; it should open the audit as a PR.
