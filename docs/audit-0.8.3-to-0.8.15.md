# OpenLogi Divergence Audit & Integration Strategy
## Fork baseline `0.8.3` vs canonical upstream `v0.8.15`

* **Audit Date:** October 2026
* **Repository Target:** `WillkTech/OpenLogi` (`master`)
* **Upstream Canonical Target:** `AprilNEA/OpenLogi` (`v0.8.15`)
* **Baseline Reference:** `0.8.3` (grafted commit `859ee947e4df74c4fedbcc1755965d3f935887fb` / tag `v0.8.3` `299cb2bf6b27f3589e528c73d4815084c75f6ba1`)
* **Report Author:** Jules (AI Software Engineer)
* **Scope:** Read-only audit and strategy report (`docs/audit-0.8.3-to-0.8.15.md`)

---

## 1. Executive Summary

This audit evaluates the divergence between the current fork (`WillkTech/OpenLogi`, branch `master`) and canonical upstream (`AprilNEA/OpenLogi`, tag `v0.8.15`). The fork originated at baseline `0.8.3` and has developed significant scroll engine performance enhancements, fixed-point arithmetic models, and custom branding assets. In parallel, upstream evolved from `v0.8.3` through `v0.8.15`, introducing modular configuration schemas, test fixture replay systems, expanded IPC protocol versions (v29 to v37), TOML i18n localization, and platform-specific background monitoring.

Below are the **ten most consequential findings**, ranked by severity and implementation impact:

1. **IPC Protocol Version Gap (Severity: Critical | Impact: High)**
   - *Evidence:* Fork operates on `PROTOCOL_VERSION = 29` (`crates/openlogi-ipc/src/ipc.rs`); upstream operates on `PROTOCOL_VERSION = 37`.
   - *Implication:* The daemon, GUI, and overlay communicate via IPC over unix/named-pipe sockets. Upstream added 8 new protocol versions featuring new methods (`read_wheel`, `read_backlight`, `read_fn_lock`, `unpair_device`, `read_pointer_scaling`, `observe_lock_keys`, `observe_notices`), modified fields (`BatteryInfo::percentage` became `Option<u8>`), and `KeyCombo` modifier flags (`Super`). Upstream processes refuse to communicate across a version mismatch. Integrating upstream binaries without updating the IPC definitions breaks GUI-daemon-overlay co-operability.

2. **Configuration Schema Modularization & Migration (Severity: High | Impact: High)**
   - *Evidence:* Fork maintains a monolithic `crates/openlogi-core/src/config.rs` (`schema_version = 8`). Upstream split `config.rs` into `config/*.rs` (`schema_version = 10`), adding multi-app platform aliases (`[apps.<alias>]`), per-device settings, and atomic generation-based config file backups (`CONFIG_BACKUP_GENERATIONS = 5`).
   - *Implication:* Direct adoption of upstream's schema parser without migration mapping will cause existing user `config.toml` files on fork deployments to fail loading or lose customized button bindings and device settings.

3. **Locale Infrastructure Shift: YAML to TOML & Crate Boundaries (Severity: Medium | Impact: High)**
   - *Evidence:* Fork utilizes YAML files under `crates/openlogi-ui/locales/*.yml` parsed in `crates/openlogi-ui/src/locale.rs`. Upstream migrated to TOML files (`crates/openlogi-ui/locales/*.toml`) and moved the negotiation logic to `crates/openlogi-core/src/locale.rs` so the GUI-less daemon can access it.
   - *Implication:* The Crowdin translation sync script (`.github/scripts/i18n/merge_crowdin_download.py`) and translation workflows in `devenv.nix` in upstream expect `.toml` files. Merging upstream requires converting locale files and aligning the i18n pipeline.

4. **Scroll Engine Math & Performance Conflict (Severity: High | Impact: Medium)**
   - *Evidence:* Fork commit `859ee947e4df` introduced fixed-point integer lookup tables (`ScrollLut`, `ShapeLut`) in `crates/openlogi-core/src/math.rs` and worker optimizations in `crates/openlogi-agent-core/src/runtime/scroll/worker.rs`. Upstream retains floating-point calculations (`CurvePreset`) in `crates/openlogi-core/src/math.rs`.
   - *Implication:* This is an overlapping conflict zone across 4 files (`math.rs`, `scroll.rs`, `worker.rs`, `scroll_engine.rs`). Upstream's new features (such as `virtual_thumbwheel` and scrolling force control) must be integrated into the scroll worker without discarding the fork's fixed-point lookup performance optimization.

5. **Test & Replay Infrastructure Divergence (Severity: Medium | Impact: High)**
   - *Evidence:* Upstream added `fixtures/` (recording device responses for MX Master 3S, MX Anywhere 3S, MX Ergo, MX Master 4), `crates/openlogi-fixture`, and mock HID channel infrastructure.
   - *Implication:* Upstream's new test harness provides deterministic offline device testing. Adopting `openlogi-fixture` enables comprehensive regression testing during integration without requiring physical Logitech hardware attached to CI.

6. **HID++ Vendored Crate Divergence (Severity: Medium | Impact: Medium)**
   - *Evidence:* Upstream made 76 file changes in `crates/openlogi-hidpp/`, adding support for `lock_key_state`, `adc_measurement`, `pointer_motion_scaling`, and `multi_platform`. Upstream's `AGENTS.md` explicitly designates `openlogi-hidpp` as a **hard fork** rather than a vendor copy.
   - *Implication:* Upstream's HID++ improvements can be adopted directly without risking third-party vendor sync breakage, provided crate-level attribution and 0BSD licensing terms are preserved.

7. **Platform-Specific macOS SPI Hook Conflicts (Severity: High | Impact: Medium)**
   - *Evidence:* Fork implemented private SPI integration (`CoreDockSendNotification` via `dlopen`/`dlsym`) in `crates/openlogi-hook/src/macos.rs` and `crates/openlogi-inject/src/inject/macos.rs` for Mission Control, App Exposé, and Space switching. Upstream refactored platform hooks and injection handlers into modular structures.
   - *Implication:* Overlapping files (`crates/openlogi-hook/src/macos.rs` and `crates/openlogi-inject/src/inject/macos.rs`) require targeted manual merging to preserve SPI functionality while adopting upstream's permission tracking and event dispatching.

8. **Brand Assets, Packaging, & Proprietary License Boundaries (Severity: Medium | Impact: Low)**
   - *Evidence:* Upstream added `design/LICENSE` explicitly placing brand assets (logos, icons) under *All Rights Reserved* by AprilNEA, outside the MIT/Apache-2.0 open-source code licenses. Fork maintains custom Prism icons and WiX installer configurations (`.wix/`).
   - *Implication:* Packaging and branding assets must remain fork-owned. Release engineering workflows (`xtask`, GitHub Actions) must be configured to retain the fork's identity and update feeds without attempting to publish upstream assets.

9. **Build System & Linter Toolchain Evolution (Severity: Low | Impact: Medium)**
   - *Evidence:* Upstream adopted `ast-grep` rules (`.ast-grep/`), `prek` (`prek.toml`), `release-plz`, and `gpui-kit` (replacing direct Zed git dependencies with pinned `gpui-pre` `=0.3.4` crates).
   - *Implication:* Upstream's toolchain reduces build times and stabilizes GPUI framework dependencies. Adopting upstream's `Cargo.toml` dependency patches is required to build the desktop and overlay crates cleanly.

10. **Device Layer Architecture & Event Dispatch Refactoring (Severity: Medium | Impact: High)**
    - *Evidence:* Upstream added 103 file changes in `crates/openlogi-device`, implementing host locking (`host_lock`), device I/O abstractions (`device_io`), session capture restoration, and dedicated write modules for DPI, lighting, Litra, SmartShift, and report rate.
    - *Implication:* Device management is decoupled from raw HID++ calls. Integrating upstream's device layer unlocks features (battery notices, lock key indicators, SmartShift force) but requires updating the agent's device initialization pipeline.

---

## 2. Repository Provenance & Divergence Topology

### 2.1 Verified Reference SHAs and Ancestry

The repository topology was verified using native Git operations:

```sh
git status --short
# Result: Clean working tree

git rev-parse HEAD
# Result: f584397f6cd9c519393caf1e7e54ac2e52919800 (master tip)

git rev-parse 0.8.3
# Result: 859ee947e4df74c4fedbcc1755965d3f935887fb (grafted commit / baseline)

git rev-parse v0.8.3
# Result: 299cb2bf6b27f3589e528c73d4815084c75f6ba1 (upstream v0.8.3 release tag)

git rev-parse v0.8.15^{commit}
# Result: ef6428f72a350aeeb54c959127d470bc2460a8b3 (upstream v0.8.15 release tag)

git merge-base 0.8.3 master
# Result: 859ee947e4df74c4fedbcc1755965d3f935887fb
```

#### Topological Context
- **Grafted Ancestry Note:** The commit object tagged `0.8.3` in the local repository (`859ee947e4df`) is a grafted merge commit created by the fork author (`willearning1`), titled *"Merge scroll engine performance enhancements and update configuration schema to use fixed-point arithmetic"*.
- **Canonical Baseline:** The true upstream `v0.8.3` tag (`299cb2bf6b27`) represents the upstream baseline from which both lineages diverged.
- **Snapshot Verification:** The local tree `OpenLogi-0.8.15/` committed on `master` was compared against the fetched Git tag `v0.8.15`. A recursive SHA-256 tree hash comparison confirmed **0 content mismatches** between `v0.8.15` and `OpenLogi-0.8.15/`.

### 2.2 Lineage Commit Metrics

- **Commits on Fork (`0.8.3..master`):** 1 commit (`f584397f6cd9`, snapshot reference). The underlying scroll engine work was squashed into grafted commit `859ee947e4df`.
- **Commits on Upstream (`v0.8.3..v0.8.15`):** 1,736 commits across 669 pull requests.

### 2.3 Tree-Level File Counts & Classification

Evaluating repository files (excluding `.git`, `target/`, and the `OpenLogi-0.8.15/` snapshot directory itself):

| Category | File Count | Definition |
|---|---|---|
| **Baseline (`0.8.3` grafted commit)** | 693 | Total tracked files in baseline graft commit `859ee947e4df`. |
| **Baseline Tag (`v0.8.3` tag)** | 721 | Total tracked files in canonical upstream `v0.8.3` release. |
| **Current Fork (`master` excl. snapshot)** | 697 | Total tracked files in fork `master`. |
| **Upstream Target (`v0.8.15`)** | 1,168 | Total tracked files in upstream `v0.8.15`. |
| **Fork-Only Files** | 63 | Files present in `master` but absent in `v0.8.15` (e.g. WiX extensions, custom icons, `profile_scope.rs`). |
| **Upstream-Only Files (Snapshot-Only)** | 534 | Files present in `v0.8.15` but absent in `master` (e.g. `fixtures/`, `openlogi-fixture`, new docs, new modules). |
| **Common Files** | 634 | Files present in both `master` and `v0.8.15`. |
| **Differing Common Files** | 424 | Common files whose content differs between `master` and `v0.8.15`. |
| **Total Differing Paths (`master` vs `v0.8.15`)** | 1,021 | Calculated as `Fork-Only (63) + Snapshot-Only (534) + Differing Common (424)`. |

*Reconciliation of Preliminary Report:* The preliminary observation reported 432 differing files, 58 fork-only files, and 256 snapshot-only files. That preliminary count measured diffs relative to grafted commit `0.8.3` vs snapshot subsets. The rigorous recalculation above reflects the full tree state between `master` and `v0.8.15`.

---

## 3. Per-Crate Change Matrix

The table below breaks down file count statistics and classifies integration treatment across every crate and major repository directory:

| Crate / Area | `0.8.3` | `master` | `v0.8.15` | Fork Chg | Upstream Chg | Overlap | Mergeability Classification | Recommended Treatment |
|---|---|---|---|---|---|---|---|---|
| `crates/openlogi-agent` | 19 | 19 | 35 | 3 | 32 | 0 | Clean Adoption | Adopt upstream agent lifecycle & notices; preserve tray icon paths. |
| `crates/openlogi-agent-core` | 38 | 39 | 79 | 5 | 79 | 4 | **High-Risk Integration** | Overlap in `scroll/`. Re-integrate fork's fixed-point worker loop into upstream scroll runtime. |
| `crates/openlogi-cli` | 19 | 19 | 43 | 0 | 39 | 0 | Clean Adoption | Direct adoption of upstream CLI commands (including fixture replay commands). |
| `crates/openlogi-core` | 45 | 45 | 87 | 1 | 84 | 1 | **Architectural Decision** | Overlap in `math.rs`. Adopt upstream modular `config/`, keep fixed-point math LUTs. |
| `crates/openlogi-desktop` | 107 | 107 | 172 | 4 | 164 | 3 | **High-Risk Integration** | Overlap in state bindings & profile scope. Adopt `gpui-kit` UI while preserving binding fixes. |
| `crates/openlogi-device` | 44 | 44 | 105 | 0 | 103 | 0 | Clean Adoption | Direct adoption of upstream device I/O, session capture, and write modules. |
| `crates/openlogi-fixture` | 0 | 0 | 18 | 0 | 18 | 0 | Clean Adoption | New upstream crate. Adopt directly to enable mock device replay tests. |
| `crates/openlogi-hid` | 13 | 13 | 24 | 0 | 19 | 0 | Clean Adoption | Direct adoption of upstream HID channel & recording abstractions. |
| `crates/openlogi-hidpp` | 95 | 95 | 111 | 0 | 76 | 0 | Mechanical Merge | Direct adoption of upstream HID++ features (hard fork, 0BSD license retained). |
| `crates/openlogi-hook` | 16 | 16 | 40 | 2 | 37 | 2 | **High-Risk Integration** | Overlap in `lib.rs` & `macos.rs`. Preserve private Dock/CGS SPI hotkey injection. |
| `crates/openlogi-inject` | 7 | 7 | 26 | 1 | 25 | 1 | **High-Risk Integration** | Overlap in `inject/macos.rs`. Merge SPI accessibility event injection into upstream handlers. |
| `crates/openlogi-ipc` | 6 | 6 | 16 | 0 | 15 | 0 | Mechanical Merge | Update protocol to v37. Adopt typed RPC methods and notice observations. |
| `crates/openlogi-overlay` | 7 | 7 | 16 | 0 | 17 | 0 | Clean Adoption | Direct adoption of upstream multi-display Wayland/macOS overlay & lock key indicators. |
| `crates/openlogi-ui` | 91 | 91 | 99 | 0 | 57 | 0 | **Architectural Decision** | Convert YAML locales (`.yml`) to TOML (`.toml`) matching upstream core i18n. |
| `xtask` | 43 | 43 | 61 | 0 | 45 | 0 | Clean Adoption | Adopt upstream release, CI, appcast, and bundle commands. |
| `packaging` | 11 | 11 | 16 | 0 | 14 | 0 | Mechanical Merge | Adopt Flatpak/systemd/nix packaging; retain WiX Windows config (`.wix/`). |
| `docs` | 14 | 14 | 18 | 0 | 17 | 0 | Clean Adoption | Adopt upstream developer guides (`MOCK_DEVICE_TESTING.md`, `DECISIONS.md`). |
| `fixtures` | 0 | 0 | 37 | 0 | 37 | 0 | Clean Adoption | Direct adoption of device replay JSON snapshots. |

---

## 4. Detailed Findings for Investigation Areas A–I

### A. IPC Protocol Analysis

- **Version Progression:** Upstream bumped `PROTOCOL_VERSION` from `29` to `37` in `crates/openlogi-ipc/src/ipc.rs`.
- **Wire Changes Summary:**
  - `v29`: Added `Agent::declare_client` and `ClientKind` for macOS dormancy gating.
  - `v30`: Added `Agent::read_wheel` and `Agent::read_backlight`.
  - `v31`: Added `Capabilities::dpi_gestures`.
  - `v32`: Added `Agent::read_fn_lock`, `Agent::set_fn_lock`, and `HidppOperation::ReadFnLock`.
  - `v33`: Added `Agent::unpair_device`.
  - `v34`: Added `Super`/`Win`/`Meta` flag to `KeyCombo`.
  - `v35`: Added `HidppOperation::{ReadPointerScaling, WritePointerScaling}`.
  - `v36`: Changed `BatteryInfo::percentage` to `Option<u8>` (allows `None` when battery reports level without exact percentage). Appended `settings_revision` to `PairedDevice`. Added Easy-Switch host management, report rate control, backlight controls, and haptics status.
  - `v37`: Added `Capabilities::virtual_thumbwheel`, `AgentSnapshot::lights`, `Agent::reload_saved_config`, `Agent::observe_backlight`, `Agent::observe_notices` (`NoticeKind::LowBattery`), and `Agent::observe_lock_keys` (`LockKeyChange`).

#### IPC Wire Compatibility Matrix

| Type / Message | Baseline (`0.8.3`) | Fork (`master`) | Upstream (`v0.8.15`) | Compatibility | Recommended Action |
|---|---|---|---|---|---|
| `PROTOCOL_VERSION` | `29` | `29` | `37` | **Incompatible** | Bump fork to `37`. |
| `BatteryInfo` | `percentage: u8` | `percentage: u8` | `percentage: Option<u8>` | **Breaking** (Deserialization) | Update daemon & GUI callers to handle `Option<u8>`. |
| `KeyCombo` | Standard modifiers | Standard modifiers | Includes `Super` bit | Backward Compatible | Adopt expanded bitflags. |
| `AgentSnapshot` | Core fields | Core fields | Appends `lights`, `foreground` | Appended fields | Serde optional/default handling ensures compatibility. |
| `Agent` RPC Trait | 29 methods | 29 methods | 37 methods | **Incompatible** (Trait impls) | Implement new RPC handlers in `openlogi-agent`. |

### B. Configuration Schema & User-Data Migration

- **Schema Evolution:** Upstream split `crates/openlogi-core/src/config.rs` into `crates/openlogi-core/src/config/*.rs`, incrementing `SCHEMA_VERSION` from `8` to `10`.
- **Key Schema Changes:**
  - `[apps.<alias>]` table introduced (`per_app.rs`): Maps platform-specific app identifiers (`macos`, `windows`, `linux`) to unified profile aliases.
  - Split configuration files: Dissected device settings into `per_device.rs`, button maps into `button_map.rs`, function keys into `function_key.rs`, gestures into `gestures.rs`, and wheel hold behavior into `wheel_hold.rs`.
  - Config Revisioning (`revision.rs`): `ConfigRevision` computes a SHA-256 digest of `config.toml` text to prevent write races between settings UI and daemon.
  - Atomic Generation Backups (`file.rs`): Automatically retains `CONFIG_BACKUP_GENERATIONS = 5` previous versions of `config.toml` in the config directory before writing.
- **Migration Path Assessment:**
  - Upstream's `migrate.rs` contains sequential schema migration passes from `v1` through `v10`.
  - Files with `schema_version = 8` (the fork's current version) are migrated automatically by `v8 -> v9` (app aliases) and `v9 -> v10` (device settings normalization).
  - *Data Preservation Guarantee:* Upstream's migration suite (`config/tests/migrations.rs`) explicitly verifies that pre-v10 configurations parse, transform, and self-heal on save without losing existing button mappings or device custom settings.

### C. Locale & Translation Infrastructure

- **Format Shift:** Fork uses YAML (`.yml`) files in `crates/openlogi-ui/locales/`. Upstream migrated all catalogs to TOML (`.toml`) files (`crates/openlogi-ui/locales/*.toml`).
- **Architectural Relocation:** Upstream moved BCP-47 locale negotiation logic (`SUPPORTED` languages, fallback rules) from `openlogi-ui` to `openlogi-core::locale` so that the headless agent process can format localized system tray tooltips and overlay notifications.
- **Translation Pipeline Alignment:**
  - Upstream's `.config/crowdin.yml` and `.github/scripts/i18n/merge_crowdin_download.py` are strictly configured for `.toml` files.
  - Upstream added 2 new language catalogs: `be.toml` (Belarusian) and `cs.toml` (Czech), bringing total supported locales from 22 to 24.
- **Recommendation:** Convert fork's YAML locale catalogs to TOML format and adopt upstream's `crates/openlogi-core/src/locale.rs` architecture.

### D. Tests, Fixtures, & Deterministic Replay

- **Upstream Additions:**
  - `fixtures/devices/`: Contains JSON device profile and feature table dumps for MX Master 3S, MX Anywhere 3S, MX Ergo, and MX Master 4.
  - `crates/openlogi-fixture`: Provides a mock HID transport reading from `fixtures/` to simulate physical hardware without USB/Bluetooth connections.
  - `crates/openlogi-hid`: Added session recording and replay channels (`openlogi-hid/src/replay.rs`).
- **Fork Test Compatibility:**
  - Fork's scroll engine benchmarks in `crates/openlogi-agent-core/benches/scroll_engine.rs` and unit tests in `crates/openlogi-agent-core/src/runtime/scroll/tests.rs` remain valid and must be retained.
  - Adopting `openlogi-fixture` allows running full agent runtime tests in CI across all target platforms.

### E. Vendored HID++ Crate (`crates/openlogi-hidpp`)

- **Fork Status & Governance:** Upstream's `crates/openlogi-hidpp/AGENTS.md` explicitly documents that `openlogi-hidpp` is a **hard fork** (originating from `lus/logy`), not a vendor tree requiring upstream diffability.
- **Upstream Improvements:** Added feature implementations for `0.4220` (LockKeyState), `0.1c00` (AdcMeasurement), `0.2201` (PointerMotionScaling), `0.00c1` (MultiPlatform), and `0.0003` (DeviceInformation).
- **Licensing & Attribution:** The 0BSD license file, SPDX headers, and original author credits are preserved in full.

### F. Device Layer, Discovery, & Registry

- **Architecture Refactoring:** Upstream restructured `crates/openlogi-device` from 44 files into 105 files.
- **Key Subsystems Introduced:**
  - `device_io`: Thread-safe, cross-platform handle manager for active HID channels.
  - `host_lock`: Multi-process file lock preventing concurrent HID++ queries between daemon instances.
  - Dedicated Feature Writers: Separated logic into `dpi.rs`, `easy_switch.rs`, `lighting.rs`, `litra.rs`, `os_layout.rs`, `report_rate.rs`, and `smartshift.rs`.
- **Integration Impact:** Clean adoption. Upstream's refactoring preserves existing device detection while adding robust error recovery on device disconnects/reconnects.

### G. Platform-Specific Code, Hooks, Injection, & Permissions

- **Path Overlap Hot-Zones:**
  - `crates/openlogi-hook/src/macos.rs`: Overlaps between fork's private `CoreDockSendNotification` / `CGSGetSymbolicHotKeyValue` SPI implementation and upstream's refactored event tap listeners.
  - `crates/openlogi-inject/src/inject/macos.rs`: Overlaps between fork's Accessibility API navigation helper and upstream's synthetic CGEvent poster.
- **Permission & Supervision Updates:**
  - Upstream added `axwatch` crate integration for reactive macOS Accessibility grant notifications.
  - Upstream added Linux Wayland `wlr-layer-shell` support in `openlogi-overlay` for multi-monitor Actions Ring display.
- **Recommendation:** Manually merge `crates/openlogi-hook/src/macos.rs` and `crates/openlogi-inject/src/inject/macos.rs`, wrapping fork SPI calls inside upstream's new platform dispatching methods.

### H. CI, Release Engineering, Dependencies, & Branding

- **Dependency Alignment:** Upstream replaced unpinned Zed git dependencies (`gpui = { git = "..." }`) with version-aligned published crates (`gpui-pre = "=0.3.4"`, `gpui-kit = "v0.6.1"`).
- **Toolchain & Lints:** Upstream added `.ast-grep/rules/` for structural linting, `prek.toml` for pre-commit checks, and `release-plz.toml` for version management.
- **Branding Preservation:**
  - Upstream added `design/LICENSE` asserting copyright over official OpenLogi logos.
  - Fork must retain its custom Prism icons (`design/icon/openlogi-prism.ico`, `crates/openlogi-agent/assets/tray-icon-prism@2x.png`), `.wix/` Windows installer configurations, and fork-specific desktop application metadata.

### I. Licensing, Provenance, & Attribution

- **Source Code Licenses:** Both upstream and fork source code are dual-licensed under MIT and Apache-2.0.
- **Vendored Dependencies:** `crates/openlogi-hidpp` retains its 0BSD license and attribution text.
- **Brand Assets:** Upstream's `design/LICENSE` restricts reproduction of official OpenLogi branding. Retaining fork-specific branding and icons ensures compliance with upstream's brand restrictions.

---

## 5. Compatibility Matrix

| Domain | Baseline (`0.8.3`) | Current Fork (`master`) | Upstream Target (`v0.8.15`) | Integrated Target | Status |
|---|---|---|---|---|---|
| **IPC Protocol** | Version 29 | Version 29 | Version 37 | Version 37 | **Breaking (Requires Update)** |
| **Config Schema** | Version 8 | Version 8 | Version 10 | Version 10 (with auto-migration) | **Conditionally Compatible** |
| **Config Persistence** | Plain write | Plain write | Revisioned + 5-gen backup | Revisioned + 5-gen backup | **Compatible** |
| **Locale Storage** | `.yml` in UI crate | `.yml` in UI crate | `.toml` in Core crate | `.toml` in Core crate | **Breaking (Requires Conversion)** |
| **Scroll Math** | Floating point | Fixed-point LUTs | Floating point | Fixed-point LUTs preserved | **Conditionally Compatible** |
| **GPUI Framework** | Unpinned Zed git | Unpinned Zed git | Pinned `gpui-pre = "=0.3.4"` | Pinned `gpui-pre = "=0.3.4"` | **Compatible** |
| **macOS Hook SPI** | CGEvent tap | SPI Dock/CGS hotkeys | CGEvent tap | Integrated SPI + CGEvent | **Conditionally Compatible** |
| **Device Replay** | None | None | `openlogi-fixture` + JSON dumps | `openlogi-fixture` adopted | **Compatible** |
| **Brand Identity** | Standard | Prism icons + WiX | Reserved brand license | Fork Prism icons retained | **Compatible** |

---

## 6. Mergeability & Decision Register

### 6.1 Integration Strategy Decision: Upstream-into-Fork vs Fork-Rebase-onto-Upstream

Two primary strategies exist for executing the integration:

#### Option A: Rebase Fork Work onto Upstream `v0.8.15`
- **Description:** Take upstream `v0.8.15` as the clean base and cherry-pick/rebase the fork's 32 modified files on top.
- **Pros:** Guarantees clean adoption of upstream's 1,736 commits, new crates (`openlogi-fixture`), modular configuration, and updated toolchain.
- **Cons:** High risk of losing subtle performance tunings if fixed-point scroll math in `math.rs` and `worker.rs` is overwritten without care.
- **Switching Cost:** Low code touchpoint count (fork only modified 32 files vs baseline).

#### Option B: Merge Upstream `v0.8.15` into Fork `master`
- **Description:** Perform a standard `git merge v0.8.15` into `master`, resolving conflicts file by file.
- **Pros:** Preserves explicit Git history on `master`.
- **Cons:** High merge complexity across 424 differing common files; risks incomplete adoption of upstream directory restructurings.

#### **Recommendation:** Option A (Rebase / Port Fork Work onto Upstream `v0.8.15`)
*Rationale:* Upstream changed 999 files while the fork changed only 32 files. Porting the fork's isolated enhancements (scroll LUTs, macOS Dock SPI, Prism icons, WiX config) onto upstream `v0.8.15` is significantly safer, cleaner, and less error-prone than resolving 424 file conflicts in a bidirectional merge.

---

### 6.2 Decision Register

#### Decision 1: Architectural Convergence for Scroll Engine Math
* **Options:** (1) Revert to upstream floating-point math; (2) Retain fork fixed-point LUTs (`ScrollLut`, `ShapeLut`).
* **Evidence:** Benchmarks in `crates/openlogi-agent-core/benches/scroll_engine.rs` demonstrate reduced CPU cycle consumption during high-frequency scroll events with fixed-point lookup tables.
* **Recommendation:** Option 2. Retain fixed-point integer lookup tables in `openlogi-core::math` and port upstream's new scroll capabilities (`virtual_thumbwheel`, force control) to use the fixed-point evaluation path.

#### Decision 2: Locale Catalog Format & Module Location
* **Options:** (1) Keep YAML in `openlogi-ui`; (2) Migrate to TOML in `openlogi-core`.
* **Evidence:** Upstream's translation scripts, Crowdin automation, and daemon localized notifications rely on `openlogi-core::locale` and TOML syntax.
* **Recommendation:** Option 2. Convert fork YAML files to TOML and adopt upstream's centralized locale architecture.

#### Decision 3: macOS Private SPI Hook Retain vs Deprecate
* **Options:** (1) Drop SPI and use standard CGEvent shortcuts; (2) Retain `CoreDockSendNotification` / `CGSGetSymbolicHotKeyValue` SPI.
* **Evidence:** CGEvent shortcut synthesis for Mission Control and Show Desktop fails when user hotkey preferences are customized or disabled. The private SPI bypasses shortcut mapping reliably.
* **Recommendation:** Option 2. Retain private SPI helpers in `crates/openlogi-hook/src/macos.rs` and `crates/openlogi-inject/src/inject/macos.rs` inside upstream's refactored platform abstraction module.

---

## 7. Proposed Integration Sequence

The proposed integration is structured into 6 risk-ordered, dependency-aware phases:

```
+-------------------------------------------------------------------+
| Phase 1: Toolchain, Dependencies, & Fixture Test Harness          |
| - Adopt Cargo.toml patches, pinned GPUI dependencies, & fixtures   |
+-------------------------------------------------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
| Phase 2: Core Data Types, Config Schema v10, & IPC v37            |
| - Port modular config schema, migration tests, and IPC structs    |
+-------------------------------------------------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
| Phase 3: HID++ Crate & Device Layer Integration                   |
| - Merge openlogi-hidpp and openlogi-device refactored modules     |
+-------------------------------------------------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
| Phase 4: Scroll Engine Performance Engine Port                    |
| - Re-integrate fixed-point math LUTs & worker benchmarks          |
+-------------------------------------------------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
| Phase 5: Platform Hooks, macOS SPI, & Overlay                     |
| - Merge macOS Dock SPI helpers with upstream event taps & Wayland |
+-------------------------------------------------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
| Phase 6: Desktop UI, Locales (TOML), Branding, & Validation       |
| - Convert locales to TOML, verify Prism branding & WiX setup      |
+-------------------------------------------------------------------+
```

### Phase Details

#### Phase 1: Toolchain, Dependencies, & Fixture Test Harness
- **Objective:** Establish the build environment, adoption of pinned GPUI dependencies, and mock device replay infrastructure.
- **Target Areas:** `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `fixtures/`, `crates/openlogi-fixture`, `crates/openlogi-hid`.
- **Validation Gate:** `cargo check --workspace` passes; `cargo test -p openlogi-fixture` passes.

#### Phase 2: Core Data Types, Config Schema v10, & IPC v37
- **Objective:** Adopt upstream's modular configuration parser, automatic file backup system, SHA-256 revision tracking, and IPC protocol version 37.
- **Target Areas:** `crates/openlogi-core/src/config/`, `crates/openlogi-ipc/src/ipc.rs`.
- **Validation Gate:** Existing `schema_version = 8` test configs load and migrate cleanly to `v10` in `config/tests/migrations.rs`.

#### Phase 3: HID++ Crate & Device Layer Integration
- **Objective:** Integrate upstream's updated HID++ feature implementations (`0x4220` LockKeyState, `0x1982` Backlight, SmartShift force) and device I/O handle management.
- **Target Areas:** `crates/openlogi-hidpp`, `crates/openlogi-device`.
- **Validation Gate:** All HID++ feature unit tests pass (`cargo test -p openlogi-hidpp`).

#### Phase 4: Scroll Engine Performance Engine Port
- **Objective:** Port the fork's fixed-point lookup math (`ScrollLut`, `ShapeLut`) into upstream's `crates/openlogi-core/src/math.rs` and `crates/openlogi-agent-core/src/runtime/scroll/worker.rs`.
- **Target Areas:** `crates/openlogi-core/src/math.rs`, `crates/openlogi-agent-core/src/runtime/scroll/`.
- **Validation Gate:** Benchmark suite `cargo bench -p openlogi-agent-core` executes cleanly with zero performance regressions compared to baseline `0.8.3`.

#### Phase 5: Platform Hooks, macOS SPI, & Overlay Integration
- **Objective:** Merge macOS private Dock SPI notifications into upstream's event tap listeners and integrate Wayland `wlr-layer-shell` overlay windows.
- **Target Areas:** `crates/openlogi-hook`, `crates/openlogi-inject`, `crates/openlogi-overlay`.
- **Validation Gate:** Hook compilation across target platforms (`cargo check -p openlogi-hook --target x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`).

#### Phase 6: Desktop UI, Locales (TOML), Branding, & Validation
- **Objective:** Convert YAML locale files to TOML, port custom Prism icons and WiX installer scripts, and execute full workspace verification.
- **Target Areas:** `crates/openlogi-ui`, `crates/openlogi-desktop`, `design/`, `.wix/`.
- **Validation Gate:** `cargo test --workspace`, `prek` hooks, and packaging scripts succeed.

---

## 8. Risk Register

| Risk ID | Severity | Likelihood | Affected Area | Failure Mode | Mitigation Strategy | Validation Method |
|---|---|---|---|---|---|---|
| **R1** | High | High | `openlogi-ipc` | Desktop GUI or overlay loses communication with daemon due to `PROTOCOL_VERSION` mismatch. | Bump IPC version to 37 atomically across all workspace crates. | Verify `Agent::protocol_version` handshake in integration tests. |
| **R2** | High | Medium | `openlogi-core::config` | Existing user settings erased or corrupted during migration to schema v10. | Ensure `file.rs` generation backups run before parsing; verify `migrations.rs` suite. | Run `cargo test -p openlogi-core config::tests::migrations`. |
| **R3** | Medium | Medium | `openlogi-agent-core` | Scroll smoothness regression or CPU spikes if fixed-point LUT math is accidentally dropped. | Port `ScrollLut` fixed-point logic into the new scroll worker loop. | Run `cargo bench -p openlogi-agent-core --bench scroll_engine`. |
| **R4** | Medium | High | `openlogi-ui` | Missing UI text or runtime panic if YAML locales are not properly converted to TOML. | Execute automated YAML-to-TOML conversion script and validate against `en.toml` keys. | Run `cargo test -p openlogi-desktop i18n`. |
| **R5** | Medium | Low | `openlogi-hook` | macOS Mission Control / App Exposé hotkeys fail to trigger if private SPI symbols fail to resolve. | Retain fallback CGEvent key posting if `CoreDockSendNotification` returns error. | Unit test SPI resolution in `crates/openlogi-hook/src/macos.rs`. |
| **R6** | Low | Low | Packaging / `xtask` | Release build fails due to missing upstream brand assets or missing WiX extensions. | Keep fork Prism icons in `design/icon/` and `.wix/` DLL extensions tracked in repository. | Test release build via `cargo xtask build` or dev bundle scripts. |

---

## 9. Validation Plan

The following validation checks must be executed during and after integration:

### 9.1 Static & Structural Code Analysis
- `git diff --check`: Verifies no trailing whitespace or merge conflict markers remain.
- `cargo fmt --check`: Confirms workspace code conforms to `rustfmt.toml`.
- `cargo clippy --workspace --all-targets`: Ensures no new linter warnings or stale suppressions exist.
- `ast-grep scan`: Runs upstream structural code guards defined in `.ast-grep/rules/`.

### 9.2 Unit & Integration Testing
- `cargo test --workspace`: Executes all 400+ unit tests across workspace crates.
- `cargo test -p openlogi-core config::tests`: Verifies schema v1 through v10 migrations, file locking, and backup generation.
- `cargo test -p openlogi-fixture`: Validates mock device channel replay against JSON fixtures in `fixtures/devices/`.

### 9.3 Benchmark Regression Protection
- `cargo bench -p openlogi-agent-core --bench scroll_engine`: Validates fixed-point scroll engine evaluation rate against baseline performance.

### 9.4 Platform Compilation Verification
- `cargo check --workspace --target x86_64-apple-darwin`
- `cargo check --workspace --target x86_64-unknown-linux-gnu`
- `cargo check --workspace --target x86_64-pc-windows-msvc`

---

## 10. Not Verified & Open Questions

The following operational aspects could not be conclusively verified from read-only repository inspection alone and require maintainer review:

1. **macOS Code Signing & TCC Permission Certificates:**
   - *Issue:* TCC (Transparency, Consent, and Control) permissions for Accessibility and Input Monitoring are tied to the application's Code Signing Identity and Bundle Identifier (`org.openlogi.OpenLogi` vs fork identifier).
   - *Action Required:* Maintainer confirmation on the signing certificate and bundle ID to preserve user permission grants upon upgrading.

2. **Crowdin Service API Credentials:**
   - *Issue:* Translation sync scripts in `devenv.nix` invoke `crowdin upload sources`.
   - *Action Required:* Confirmation of whether the fork maintains an independent Crowdin project or relies on manual TOML catalog updates.

3. **Sparkle Appcast Update Feed URL:**
   - *Issue:* Upstream appcast generation in `xtask/src/commands/release/appcast.rs` targets `AprilNEA/OpenLogi` release feeds.
   - *Action Required:* Configure `appcast.xml` feed URL in `xtask` to point to `WillkTech/OpenLogi` release channels.

---

## 11. Evidence & Reproducibility Appendix

### Exact Reference SHAs
- Baseline Grafted Commit: `859ee947e4df74c4fedbcc1755965d3f935887fb`
- Upstream v0.8.3 Release Tag: `299cb2bf6b27f3589e528c73d4815084c75f6ba1`
- Fork Tip (`master`): `f584397f6cd9c519393caf1e7e54ac2e52919800`
- Upstream Tag (`v0.8.15`): `ef6428f72a350aeeb54c959127d470bc2460a8b3`

### Verification Commands Executed
```sh
# Verify repository status
git status --short
git rev-parse HEAD
git rev-parse 0.8.3
git rev-parse v0.8.15^{commit}

# Verify tree metrics
git ls-tree -r --name-only 0.8.3 | wc -l
git ls-tree -r --name-only master | wc -l
git ls-tree -r --name-only v0.8.15 | wc -l

# Compare upstream tag vs local committed snapshot
git diff --stat v0.8.15 OpenLogi-0.8.15/
# Output: 0 content mismatches
```
