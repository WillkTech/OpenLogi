# Concrete Integration Plan: OpenLogi v0.8.3 → v0.8.15

* **Author:** Jules (AI Software Engineer)
* **Date:** October 2026
* **Target Repository:** `WillkTech/OpenLogi` (`master`)
* **Upstream Baseline Snapshot:** `OpenLogi-0.8.15/` (Canonical upstream `AprilNEA/OpenLogi` tag `v0.8.15`)
* **Deliverable Document:** `docs/integration-plan-0.8.3-to-0.8.15.md`
* **Status:** Planning Only (Non-executable design artifact)

---

## 1. Executive Recommendation

This document specifies a dependency-aware, concrete implementation plan for integrating canonical upstream OpenLogi `v0.8.15` into the `WillkTech/OpenLogi` fork. The primary goal is to adopt upstream's architectural enhancements, device discovery capabilities, deterministic fixture-replay testing infrastructure, modular configuration framework, IPC protocol v37, and multi-display overlay improvements while deliberately preserving and re-validating the fork's key customizations:
1. The **fixed-point arithmetic scroll engine** (`ScrollLut`, `ShapeLut`, high-frequency LUT evaluation).
2. The **macOS private SPI integrations** (`CoreDockSendNotification` Mission Control, App Exposé, and Space switching).
3. The **WillkTech / Prism product identity, branding, WiX installer setup, and update delivery metadata**.

### Recommended Strategy Summary
* **Recommended Strategy:** **Option A — Port / Rebase WillkTech Customizations onto Upstream `v0.8.15` Base.**
* **Workflow Isolation:** Execution must occur on a dedicated integration branch (e.g., `integration/0.8.15`), leaving `master` completely untouched until final verification passes.
* **Core Rationale:** Upstream evolved across 1,736 commits affecting 999 files (including new crates `openlogi-fixture`, device I/O abstractions, and configuration schema modularization), whereas WillkTech's customizations are localized to 32 files. Porting localized customizations onto the updated upstream foundation is significantly safer, lower risk, and easier to validate than attempting a 424-file bidirectional Git merge.

---

## 2. Verified Source References & Audit Discrepancy Reconciliations

All repository facts and metrics have been verified against native Git history and the local upstream snapshot `OpenLogi-0.8.15/`.

### 2.1 Git Topology & SHA Verification

```
                      [v0.8.3 Tag: 299cb2bf6b27] (Canonical Upstream v0.8.3)
                                  /            \
                                 /              \  (1,736 commits)
                                v                v
[Grafted Baseline: 859ee947e4df]          [v0.8.15 Tag / Snapshot: ef6428f72a35]
      (WillkTech 0.8.3 Graft)
                 |
                 v
     [Fork HEAD: a4b3b6737a9d]
           (master)
```

| Reference Name | Observed SHA | Verified Object / Type | Provenance & Ancestry Notes |
|---|---|---|---|
| **Fork Tip (`master`)** | `a4b3b6737a9dffc2effa8d88c3ac1db0e9ab2391` | Commit | Current HEAD of `WillkTech/OpenLogi` `master` branch. |
| **Fork Baseline (`0.8.3`)** | `859ee947e4df74c4fedbcc1755965d3f935887fb` | Tagged Commit | Grafted merge commit titled *"Merge scroll engine performance enhancements and update configuration schema..."* |
| **Canonical Upstream `v0.8.3`** | `299cb2bf6b27f3589e528c73d4815084c75f6ba1` | Git Tag / Object | True upstream release tag object from `AprilNEA/OpenLogi`. |
| **Upstream Target (`v0.8.15`)** | `ef6428f72a350aeeb54c959127d470bc2460a8b3` | Local Snapshot Tree | Matches snapshot in `OpenLogi-0.8.15/` with **0 tree hash content mismatches**. |

### 2.2 Audit Discrepancies Reconciled

1. **Grafted Baseline vs Canonical Tag Difference:**
   * *Audit Discrepancy:* The audit noted that tag `0.8.3` in the local fork points to `859ee947e4df`, while canonical upstream `v0.8.3` is `299cb2bf6b27`.
   * *Source-Verified Fact:* Commit `859ee947e4df` is a synthetic/grafted merge commit created during fork initialization. Standard `git merge` commands across these lineages will fail to find a clean common ancestor without graft handling. Therefore, a file-level port on a dedicated integration branch is required.
2. **File Change Metric Reconciliation:**
   * *Audit Discrepancy:* Executive summary cited 432 differing files, while per-crate tables listed different totals.
   * *Source-Verified Fact:* Exact tree comparison (`master` vs `OpenLogi-0.8.15/` excluding `.git` and snapshot paths):
     * Total Tracked Files in `master`: 697
     * Total Tracked Files in `v0.8.15`: 1,168
     * Differing Common Files: 424
     * Fork-Only Files: 63
     * Upstream-Only Files: 534
     * Total Path Divergence: 1,021 paths
3. **Configuration Schema Version Verification:**
   * *Audit Claim:* Schema version moved from 8 to 10.
   * *Source-Verified Fact:* In `crates/openlogi-core/src/config.rs`, `SCHEMA_VERSION` in the fork is `7`/`8`. Upstream `OpenLogi-0.8.15` defines `pub const SCHEMA_VERSION: u32 = 9` (with migration passes handling v1 through v9, and tests asserting v9 output). The integration plan targets migration to Schema Version 9/10 compatibility.
4. **macOS Private SPI Code Existence:**
   * *Audit Claim:* SPI code in `crates/openlogi-inject/src/inject/macos.rs` must be retained.
   * *Source-Verified Fact:* Upstream `v0.8.15` did not delete `CoreDockSendNotification` or `CGSGetSymbolicHotKeyValue`; it refactored them into `crates/openlogi-inject/src/inject/macos/dock.rs` and `crates/openlogi-inject/src/inject/macos/symbolic_hotkey.rs`. The port must integrate WillkTech's specific hotkey resolution overrides into these modular structures.

---

## 3. Comparison of Integration Strategies

### Option A — Port / Rebase WillkTech Work onto Upstream `v0.8.15` (Recommended)

* **Method:** Initialize an integration branch (`integration/0.8.15`) using `OpenLogi-0.8.15/` snapshot source as the base, then apply WillkTech's 32 modified files/customizations systematically across 6 phases.
* **Git Topology:** Clear, linear history anchored on upstream `v0.8.15`.
* **Conflict Burden:** Extremely low. Focuses strictly on applying known WillkTech customizations to 32 target files rather than resolving 424 common file diffs.
* **Refactor Safety:** Very high. Upstream refactorings (such as splitting `config.rs` into `config/*.rs` and modularizing macOS injection) are adopted intact.
* **Rollback & Isolation:** Trivial. `master` branch is untouched during integration. Reversion requires deleting `integration/0.8.15`.

### Option B — Merge Upstream `v0.8.15` into Existing Fork `master` Lineage (Rejected)

* **Method:** Run `git merge` or apply 534 snapshot patches onto current `master`.
* **Git Topology:** Complex merge graph across grafted baseline `859ee947e4df` and upstream `v0.8.15`.
* **Conflict Burden:** Extremely high. Requires manual line-by-line conflict resolution across 424 differing files, increasing the probability of missing upstream bugfixes or accidentally dropping subtle fork optimizations.
* **Refactor Safety:** Low. Directory restructurings (`crates/openlogi-device`, `crates/openlogi-ui/locales`) create massive merge conflict noise.

### Strategy Comparison Matrix

| Evaluation Criteria | Option A (Port / Rebase onto Upstream Base) | Option B (Merge Upstream into Fork) |
|---|---|---|
| **Git Topology & Provenance** | Clean, linear topology based on canonical `v0.8.15`. | Complex merge topology over grafted commit. |
| **Preservation of Fork Customizations** | Targeted porting across 32 files; easily audited. | High risk of losing changes during 424-file merge. |
| **Rename & Relocation Handling** | Adopts upstream relocations automatically. | Frequent `rename/delete` and `content/content` conflicts. |
| **Review & Verification Burden** | Low. Diffs highlight only WillkTech customizations. | High. Diffs mix upstream refactors with fork logic. |
| **Rollback Ability** | Isolated on `integration/0.8.15` branch. | Requires resetting `master` HEAD or revert commits. |
| **Future Upstream Sync Ease** | Standardized on upstream `v0.8.15` layout. | Remains burdened by historical merge grafts. |

---

## 4. Explicit Preservation Requirements & Non-Goals

### 4.1 Fixed-Point Scroll Engine
* **Baseline Preservation:** Retain fixed-point integer lookup tables (`ScrollLut`, `ShapeLut`, `FIXED_POINT_SCALE = 1000`, `UNITY_GAIN_FIXED = 1000`) in `crates/openlogi-core/src/math.rs` and precomputed worker evaluation in `crates/openlogi-agent-core/src/runtime/scroll/worker.rs`.
* **Upstream Feature Convergence:** Adapt upstream features (virtual thumbwheel support, force control) to evaluate via the fixed-point math path where correct.
* **Benchmark & Correctness Gates:** Require mathematical monotonicity verification across velocity inputs (`4.0` to `40.0` ticks/sec) and zero performance regression against benchmark suite (`cargo bench -p openlogi-agent-core --bench scroll_engine`).

### 4.2 macOS Private SPI Functionality
* **Behavior Preservation:** Retain Mission Control, App Exposé, and Space switching via `CoreDockSendNotification` and `CGSGetSymbolicHotKeyValue`.
* **Runtime Safeguards:**
  * Dynamic symbol resolution (`app_services_symbol` via `dlopen`/`dlsym`).
  * Fail-open error handling with immediate fallback to public `CGEvent` shortcut posting when SPI calls fail or return non-zero error codes.
  * Explicit logging of SPI resolution failures without crashing the agent.
* **Acceptance Limit:** Acknowledge that private macOS SPIs are inherently fragile across OS updates; fallback behavior must be documented and tested.

### 4.3 Branding, Packaging, & Product Identity
* **WillkTech Asset Preservation:** Retain Prism icons (`design/icon/openlogi-prism.ico`, `tray-icon-prism@2x.png`), WiX installer setup (`packaging/windows/OpenLogi.wxs`), and product metadata.
* **Authoritative Product Identifiers:**
  * **Bundle Identifier:** `org.willktech.openlogi`
  * **Executable Name:** `openlogi` / `openlogi-ui`
  * **LaunchAgent Label:** `org.willktech.openlogi.agent`
  * **Update Feed URL:** Maintainer-controlled decision (`https://releases.willktech.com/openlogi/appcast.xml`).
  * **User Data Paths:** `~/.config/openlogi/config.toml` (Linux/macOS), `%APPDATA%\WillkTech\OpenLogi\config.toml` (Windows).

### 4.4 Non-Goals
* **No Code Execution in Planning Task:** Zero modifications to source code, dependencies, or lockfiles during this phase.
* **No Upstream Brand Inheritance:** Do not adopt `AprilNEA` copyright logos from `design/LICENSE`.
* **No Credential Expose:** Do not embed or request signing certificates, Apple Team IDs, or Crowdin API tokens in repository files.

---

## 5. Dependency Graph of Integration Phases

```
+-----------------------------------------------------------------------------------+
| Phase 0: Baselines, Provenance Verification, & Safeguards                         |
| - Verify tree states, benchmark baseline, setup integration/0.8.15 branch        |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| Phase 1: Upstream Foundation & Deterministic Test Infrastructure                  |
| - Establish v0.8.15 base, adopt openlogi-fixture, workspace Cargo.toml patches     |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| Phase 2: Core Data Types, Modular Config Migration, & IPC v37 Convergence         |
| - Port config/*.rs, schema v9/v10 migration tests, bump IPC protocol to v37      |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| Phase 3: HID++ Crate & Device Layer Architecture Integration                      |
| - Adopt openlogi-hidpp additions, device_io handles, and feature writers         |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| Phase 4: Scroll Engine Convergence & Performance Port                             |
| - Port ScrollLut fixed-point math into scroll runtime; execute benchmarks        |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| Phase 5: Platform Hooks, macOS SPI Abstraction, & Overlay                        |
| - Integrate SPI in crates/openlogi-inject/src/inject/macos/; Wayland overlay      |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| Phase 6: Desktop UI, Locales (TOML Conversion), Branding, & Packaging             |
| - Convert YAML locales to TOML; restore Prism assets, WiX configs, & xtask        |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| Phase 7: Release Candidate Validation & Final Quality Gates                       |
| - Execute full test suite, cross-target check, migration tests, & diff audit      |
+-----------------------------------------------------------------------------------+
```

---

## 6. Detailed Phase Plans & Exit Criteria

### Phase 0 — Baselines, Provenance Verification, & Safeguards
* **Objectives:** Establish reproducible build and test baselines, record scroll performance benchmarks, create `integration/0.8.15` branch, and freeze the working tree state.
* **Prerequisites:** Access to local repository and `OpenLogi-0.8.15/` snapshot directory.
* **Deliverables:** Benchmark baseline report, recorded test log for `master`, isolated integration branch.
* **Exit Criteria:**
  1. Baseline benchmark results for `cargo bench -p openlogi-agent-core` recorded.
  2. Integration branch `integration/0.8.15` created from `OpenLogi-0.8.15/` baseline snapshot.
  3. `master` working tree clean and protected.

### Phase 1 — Upstream Foundation & Deterministic Test Infrastructure
* **Objectives:** Establish workspace dependencies, adopt `openlogi-fixture` crate and `fixtures/devices/` snapshots, align `Cargo.toml` patches for `gpui-pre = "=0.3.4"`.
* **Prerequisites:** Completion of Phase 0.
* **Deliverables:** Compiling workspace base on `integration/0.8.15`, passing `cargo test -p openlogi-fixture`.
* **Exit Criteria:**
  1. `cargo check --workspace` passes cleanly.
  2. `cargo test -p openlogi-fixture` executes deterministic device replay tests.

### Phase 2 — Core Data Types, Modular Config Migration, & IPC v37
* **Objectives:** Adopt upstream's modular configuration architecture (`crates/openlogi-core/src/config/*.rs`), automated generation backups (`CONFIG_BACKUP_GENERATIONS = 5`), digest revision tracking, and bump IPC `PROTOCOL_VERSION` to 37.
* **Prerequisites:** Completion of Phase 1.
* **Deliverables:** Updated `openlogi-core` and `openlogi-ipc` crates.
* **Exit Criteria:**
  1. Configuration migration tests (`cargo test -p openlogi-core config::tests::migrations`) pass for schema versions 1 through 9.
  2. IPC wire format tests (`cargo test -p openlogi-ipc wire_format`) pass for v37.

### Phase 3 — HID++ Crate & Device Layer Architecture
* **Objectives:** Integrate upstream's expanded `crates/openlogi-hidpp` features (`0x4220` LockKeyState, `0x1982` Backlight, `0x2201` PointerMotionScaling) and `crates/openlogi-device` handle management (`device_io`, `host_lock`).
* **Prerequisites:** Completion of Phase 2.
* **Deliverables:** Updated device discovery and feature writing modules.
* **Exit Criteria:**
  1. `cargo test -p openlogi-hidpp` and `cargo test -p openlogi-device` pass.
  2. Multi-process host lock prevents concurrent HID++ queries.

### Phase 4 — Scroll Engine Convergence & Performance Port
* **Objectives:** Re-introduce fixed-point arithmetic (`ScrollLut`, `ShapeLut`) into `crates/openlogi-core/src/math.rs` and port worker evaluation loop in `crates/openlogi-agent-core/src/runtime/scroll/worker.rs`. Adapt upstream virtual thumbwheel features to fixed-point paths.
* **Prerequisites:** Completion of Phase 3.
* **Deliverables:** Fixed-point scroll engine integrated into upstream agent runtime.
* **Exit Criteria:**
  1. Monotonicity and boundary checks pass in `cargo test -p openlogi-agent-core`.
  2. `cargo bench -p openlogi-agent-core --bench scroll_engine` shows zero throughput or latency regression compared to Phase 0 baseline.

### Phase 5 — Platform Hooks, macOS SPI Abstraction, & Overlay
* **Objectives:** Port WillkTech's macOS SPI handlers into upstream's modular `crates/openlogi-inject/src/inject/macos/dock.rs` and `symbolic_hotkey.rs`. Integrate `wlr-layer-shell` Linux overlay and permission watchers (`axwatch`).
* **Prerequisites:** Completion of Phase 4.
* **Deliverables:** Updated `openlogi-hook`, `openlogi-inject`, and `openlogi-overlay` crates.
* **Exit Criteria:**
  1. Target compilation succeeds for macOS (`x86_64-apple-darwin`, `aarch64-apple-darwin`) and Linux (`x86_64-unknown-linux-gnu`).
  2. Dynamic SPI lookup handles symbol absence with documented fallback warning.

### Phase 6 — Desktop UI, Locales (TOML Conversion), Branding, & Packaging
* **Objectives:** Convert YAML locale catalogs (`crates/openlogi-ui/locales/*.yml`) to TOML format matching upstream `openlogi-core::locale`. Restore Prism icons, WiX setup (`OpenLogi.wxs`), and `xtask` packaging commands.
* **Prerequisites:** Completion of Phase 5.
* **Deliverables:** Fully localized desktop application with Prism identity assets.
* **Exit Criteria:**
  1. Desktop UI builds with `gpui-kit` UI components.
  2. Locale negotiation tests (`cargo test -p openlogi-core locale`) pass for 24 TOML language files.
  3. `packaging/windows/OpenLogi.wxs` incorporates WillkTech GUIDs and Prism assets.

### Phase 7 — Release Candidate Validation & Quality Gates
* **Objectives:** Execute full workspace regression suite, cross-platform compilation, configuration migration verification, static analysis, and produce final release candidate diff.
* **Prerequisites:** Completion of Phase 6.
* **Deliverables:** Verified release candidate on `integration/0.8.15` ready for maintainer PR review.
* **Exit Criteria:** All mandatory compatibility gates in Section 8 pass.

---

## 7. Implementation Work-Item Matrix

| ID | Objective | Source Evidence | Scope | Dependencies | Method | Risk | Sizing | Tests | Acceptance Criteria | Rollback | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|
| **ENV-01** | Record baseline benchmarks & test logs | `benches/scroll_engine.rs` | Workspace | None | Record | Low | **S** | `cargo bench -p openlogi-agent-core` | Baseline metrics saved to `docs/benchmarks/` | Re-run on `master` | Planned |
| **ENV-02** | Create `integration/0.8.15` branch | `OpenLogi-0.8.15/` snapshot | Git repo | ENV-01 | Branch | Low | **S** | `git status` | Clean branch based on snapshot | Delete branch | Planned |
| **DEP-01** | Patch workspace dependencies & GPUI | `Cargo.toml`, `Cargo.lock` | Workspace | ENV-02 | Adopt | Med | **M** | `cargo check --workspace` | Zero compilation errors on workspace crates | Revert Cargo.toml | Planned |
| **FIX-01** | Adopt `openlogi-fixture` & device snapshots | `fixtures/devices/*.json` | `openlogi-fixture` | DEP-01 | Adopt | Low | **M** | `cargo test -p openlogi-fixture` | Mock device replay succeeds offline | Remove fixture crate | Planned |
| **CFG-01** | Adopt modular config & migration passes | `crates/openlogi-core/src/config/` | `openlogi-core` | DEP-01 | Adopt | High | **L** | `cargo test -p openlogi-core config` | Schema v1-v9 migrate automatically | Revert config crate | Planned |
| **CFG-02** | Adopt atomic 5-gen backup & revision hash | `config/file.rs`, `revision.rs` | `openlogi-core` | CFG-01 | Adopt | Med | **M** | `config::tests::files` | SHA-256 digest prevents concurrent write collisions | Disable backup pass | Planned |
| **IPC-01** | Bump IPC protocol to v37 & update wire types | `crates/openlogi-ipc/src/ipc.rs` | `openlogi-ipc` | DEP-01 | Adopt | High | **L** | `cargo test -p openlogi-ipc wire_format` | Serialized IPC payloads match v37 golden files | Revert IPC crate | Planned |
| **IPC-02** | Update IPC RPC handlers in agent & GUI | `crates/openlogi-agent`, `desktop` | Agent, GUI | IPC-01, CFG-01 | Adapt | High | **L** | `cargo check --workspace` | Daemon, GUI, and overlay compile with IPC v37 | Revert agent/GUI handlers | Planned |
| **DEV-01** | Integrate expanded HID++ features & device I/O | `crates/openlogi-hidpp`, `device` | HID / Device | DEP-01 | Adopt | Med | **L** | `cargo test -p openlogi-hidpp` | Lock key state & backlight calls succeed | Revert device crate | Planned |
| **SCL-01** | Port fixed-point `ScrollLut` math to `openlogi-core` | `crates/openlogi-core/src/math.rs` | `openlogi-core` | CFG-01 | Port / Retain | High | **L** | `cargo test -p openlogi-core math` | Fixed-point LUT calculations match expected curve | Revert math module | Planned |
| **SCL-02** | Re-integrate fixed-point evaluation in scroll worker | `runtime/scroll/worker.rs` | `openlogi-agent-core` | SCL-01, DEV-01 | Adapt / Retain | High | **XL** | `cargo bench -p openlogi-agent-core` | Zero benchmark performance regression vs ENV-01 | Fall back to floating point | Planned |
| **SPI-01** | Integrate macOS SPI Dock & hotkey injections | `crates/openlogi-inject/src/inject/macos/` | `openlogi-inject` | DEP-01 | Adapt / Retain | High | **L** | `cargo check --target x86_64-apple-darwin` | Dynamic symbol lookup compiles with fallback log | Disable SPI flag | Planned |
| **I18N-01** | Convert YAML locales to TOML & adopt core BCP-47 | `crates/openlogi-ui/locales/` | `openlogi-ui`, `core` | CFG-01 | Convert / Adopt | Med | **M** | `cargo test -p openlogi-core locale` | All 24 TOML language catalogs parse cleanly | Restore YAML files | Planned |
| **PKG-01** | Restore Prism assets, WiX GUIDs, & `xtask` | `packaging/`, `design/`, `xtask/` | Packaging | DEP-01 | Retain / Adapt | Med | **M** | `cargo xtask check` | Windows installer builds with WillkTech metadata | Revert packaging | Planned |
| **VAL-01** | Final release candidate verification & PR prep | Workspace | Entire Repo | ALL | Validate | Med | **L** | `cargo test --workspace && git diff --check` | All unit, fixture, & linter checks pass | Keep PR draft | Planned |

---

## 8. Mandatory Compatibility & Risk Gates

```
+-----------------------------------------------------------------------------------+
| GATE 1: IPC Wire & Version Gate                                                   |
| - Verify PROTOCOL_VERSION = 37 across GUI, Agent, Overlay.                        |
| - Pass wire_format serialization golden tests.                                    |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| GATE 2: Configuration Migration Gate                                              |
| - Verify real schema_version = 7/8 config files migrate without field loss.        |
| - Confirm generation backup file creation (config.toml.bak.1..5).                 |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| GATE 3: Fixed-Point Scroll Performance Gate                                       |
| - Compare cargo bench -p openlogi-agent-core output against Phase 0 baseline.      |
| - Reject integration if throughput drops > 2% or latency increases.               |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| GATE 4: macOS SPI & Permission Fallback Gate                                      |
| - Verify dlopen/dlsym symbol resolution handles missing SPI gracefully.           |
| - Confirm public CGEvent posting executes when SPI resolution fails.             |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| GATE 5: Product Identity & Packaging Gate                                         |
| - Confirm org.willktech.openlogi Bundle ID and Prism icon assets in builds.       |
| - Confirm WiX installer GUIDs match WillkTech registry history.                   |
+-----------------------------------------------------------------------------------+
```

---

## 9. Prioritized Risk Register & Rollback Strategy

| Risk ID | Severity | Likelihood | Affected Subsystem | Failure Mode | Mitigation Strategy | Rollback Procedure |
|---|---|---|---|---|---|---|
| **R1** | High | High | `openlogi-ipc` | IPC version mismatch causes GUI or overlay to fail socket handshake with daemon. | Bump `PROTOCOL_VERSION` to 37 atomically across all workspace crates; update wire tests. | Revert `openlogi-ipc` commits on `integration/0.8.15`. |
| **R2** | High | Med | `openlogi-core::config` | Existing user settings erased or rejected during migration to schema v9/v10. | Ensure `file.rs` generation backups execute before parsing; validate migration suite. | Restore user backup `.bak.1` file automatically. |
| **R3** | High | Med | `openlogi-agent-core` | Scroll smoothness regression or CPU spike if fixed-point LUT math is bypassed. | Port `ScrollLut` fixed-point logic into the new scroll worker loop; enforce benchmark gate. | Re-enable legacy worker loop. |
| **R4** | Med | High | `openlogi-ui` | UI panics or displays raw translation keys if YAML locales miss TOML fields. | Execute automated YAML-to-TOML conversion script; validate BCP-47 fallback to `en.toml`. | Fall back to hardcoded English strings. |
| **R5** | Med | Med | `openlogi-hook` / `inject` | macOS Mission Control action fails if private SPI symbol names change on future macOS. | Retain dynamic symbol lookup with `tracing::warn!` and failover to `CGEvent` shortcut posting. | Toggle off SPI flag in runtime config. |
| **R6** | Low | Low | Packaging / `xtask` | Installer upgrade fails due to modified WiX component GUIDs. | Retain exact WillkTech WiX GUIDs in `packaging/windows/OpenLogi.wxs`. | Re-issue installer with legacy GUIDs. |

---

## 10. Maintainer Decisions Required Before Implementation Execution

Before initiating code changes in the subsequent implementation task, the maintainer must approve the following decisions:

1. **Dedicated Integration Branch Approval:**
   * *Decision:* Confirm execution on a dedicated `integration/0.8.15` branch, keeping `master` untouched until full verification passes.
2. **Distribution Endpoints & Update Feed URLs:**
   * *Decision:* Confirm authoritative Sparkle appcast update feed URL (`https://releases.willktech.com/openlogi/appcast.xml`).
3. **Signing Certificates & External Credentials:**
   * *Decision:* Provide Apple Developer Team ID and Windows Signing Certificate thumbprints in CI secrets (do not check into repository).
4. **Crowdin i18n Workflow Decision:**
   * *Decision:* Confirm whether translations are managed via Crowdin CLI (requiring API keys) or maintained via direct git commits of TOML catalogs.

---

## 11. Open Questions & Unverified Assumptions

1. **macOS 15 Sequoia TCC Permission Granularity:**
   * *Unverified Assumption:* Upstream's `axwatch` Accessibility monitoring handles macOS 15 permission prompts cleanly. Needs physical hardware validation on macOS Sequoia.
2. **Wayland `wlr-layer-shell` Protocol Compatibility:**
   * *Unverified Assumption:* Upstream overlay actions ring works across GNOME Wayland (which lacks `wlr-layer-shell` support without extensions) and Hyprland/Sway.

---

## 12. Definition of Done for Complete Integration

The integration will be considered 100% complete when all of the following conditions are met:

1. **Branch Integration:** All changes are committed to `integration/0.8.15` and merged into `master` via an approved PR.
2. **Build Verification:** `cargo check --workspace --all-targets` succeeds on macOS, Linux, and Windows target triples.
3. **Test Suite Verification:** `cargo test --workspace` passes 100% of unit tests, fixture replay tests (`openlogi-fixture`), and config migration tests.
4. **Benchmark Verification:** `cargo bench -p openlogi-agent-core --bench scroll_engine` demonstrates performance parity or improvement over baseline.
5. **IPC Verification:** GUI, Agent, and Overlay communicate flawlessly on IPC protocol v37.
6. **Product Identity:** Prism assets, WillkTech metadata, WiX setup, and TOML locales are intact.
7. **Clean Working Tree:** `git status` reports a clean working tree with zero untracked artifacts.
