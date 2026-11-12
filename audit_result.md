# Feature & Performance Audit — fork `master` (0.8.3) vs upstream `v0.9.0+143`

Deliverable per `audit_prompt.md`, written to `audit_result.md` at the maintainer's
direction (the prompt's nominal path was `docs/audit-features-perf-0.9.md`).
Audit date: 2026-11-13. Trees examined:

| Side | Ref | Notes |
|---|---|---|
| Fork | `master` = `b6451aa` (workspace version `0.8.3`) | 17 commits past fork-side tag `0.8.3` (`859ee94`); 61 commits total |
| Upstream | `openlogi_0.9/` clone, `master` = `f3b57974` = `v0.9.0-143` | 2,118 commits total; verified with `git -C openlogi_0.9 rev-parse HEAD` / `describe --tags` |

Method note: all commit-level analysis ran in a throwaway clone under `$TMPDIR/audit/fork`
(fork objects + `upstream` remote fetched from `openlogi_0.9/`); neither tree's `.git` was
modified, no source file was edited, and nothing was committed. Claims are labeled
**MEASURED** (command output quoted) or **INFERRED** (read-and-reasoned).

---

## 1. Executive summary — top 10 findings

1. **The two lineages share no git history at all, and the true fork point is upstream
   `v0.8.0`, not `0.8.3`.** `git merge-base master upstream/master` exits 1 (no common
   ancestor; MEASURED). The fork's root commit `96a2beea` ("Restore UI error fix",
   2026-08-26 11:31 +0800) differs from upstream tag `v0.8.0` (`77a4a9b`, released
   2026-08-25 23:30) by only **83 files, +296/−1,578 lines** (MEASURED:
   `git diff --shortstat 96a2beea v0.8.0`). The fork is a history-rewritten snapshot of
   upstream v0.8.0; its own tags `0.8.2`/`0.8.3` are fork-local version numbers unrelated
   to upstream releases of the same names.
2. **Tag mismatch confirmed:** fork tag `0.8.3` = `859ee94` (tree `49b8ba2b`) vs upstream
   tag `v0.8.3` = `4d0936d` (tree `9db4501e`) — different commits, different trees
   (MEASURED: `git rev-parse 0.8.3^{tree} v0.8.3^{tree}`). Tree distance from the fork tag
   to upstream tags: v0.8.0 = 168 files, v0.8.1 = 193, v0.8.2 = 349, v0.8.3 = 355
   (MEASURED) — the fork drifted *away* from its own version name.
3. **Scale of divergence:** upstream moved **1,033 commits** since v0.8.0 (946 since its
   v0.8.3; 803 to v0.9.0 + 143 after); the fork has **61** commits, 17 since its tag.
   Files changed: upstream 1,383, fork 195, **both sides 146** (conflict hot-zones;
   MEASURED via `comm` over `git diff --name-only v0.8.0 <side>`).
4. **IPC 29 → 46 is purely additive but forces lockstep releases.** Fork
   `PROTOCOL_VERSION = 29`, upstream `46` (MEASURED, `crates/openlogi-ipc/src/ipc.rs:63` /
   `:193`). Across those 17 versions upstream **added 24 wire types and 33 RPC methods and
   deleted nothing** (MEASURED: type/method set diff over both `ipc.rs`). The client
   handshake detects a version mismatch in both directions and demands a relaunch
   (`upstream ipc/src/client.rs:81-92`) — mixing is detected, not silent, but any
   integration must ship GUI+agent+overlay together.
5. **On-disk config is effectively incompatible in both directions.** Fork
   `SCHEMA_VERSION = 7`, upstream `11`, and both sides independently bumped 6→7 for
   *different* changes (fork: `custom_shortcuts`; upstream: thumbwheel native direction).
   `deny_unknown_fields` everywhere means hard load errors, not silent loss — but the fork's
   scroll-curve keys (`vertical_curve_preset`, …), `[[custom_shortcuts]]`, and
   `Action::SavedShortcut` are unknown to upstream, and upstream v11 > fork ceiling 7.
   A one-time translator is mandatory (§4.C).
6. **Flow is upstream's largest new subsystem and the fork has none of it:** ~8,000-LoC
   protocol crate (`openlogi-flow`: frame/negotiation/pairing/SAS/session/QUIC-TLS),
   ~9,150 LoC of agent orchestration, a 557-LoC `[flow]` config section, 7 IPC types +
   7 methods, and a ~2,300-LoC GUI window. It is opt-in (`enabled` defaults false) and
   young (pairing shipped at 0.8.21-beta.58; QUIC keep-alive fixed for 0.9.0, `d288163e`).
7. **The scroll model is a product fork, not just a schema fork.** Fork: velocity→gain
   curves — 13-variant `CurvePreset`, fixed-point `ScrollLut`, measured **9.1 ns** per
   fixed-point gain (MEASURED bench, §5.2), macOS `CVDisplayLink`-synced decay, f32 math.
   Upstream: momentum/glide model (`scroll/glide.rs`, `free_spin.rs`, f64, per-frame `exp`
   + `powf` + a heap `Vec` per frame) with no acceleration-curve concept at all. Neither
   lineage can absorb the other's semantics without a product decision.
8. **GPUI lineage diverged under both apps.** Fork: Zed `gpui 0.2.2` from git rev
   `55007f5` + a fork-only `[patch.crates-io]` for swr-gpui; upstream: longbridge
   `gpui-pre 0.3.10` (crates.io) + `gpui-component v0.8.1` + `gpui-kit`. Any GUI merge is
   also a renderer-stack migration; the fork's patch must be re-homed or dropped.
9. **Upstream perf the fork still needs (and vice versa).** The fork still logs `info!`
   inside the CGEventTap callback into a rolling file appender — exactly what upstream
   `268602eb` removed — and still polls capture state at 1 Hz (`091b7aa2`), builds the
   asset resolver 3× at startup without memoization (`bf2f30c2`/`5145cdb9`), and idles the
   button worker at 10 ms polls (`e2d63f06`). Conversely upstream has no equivalent of the
   fork's LUT gains, display-link decay, tap-thread QoS boost, or allocation-free
   `advance_due`.
10. **Identity collision blocks coexistence.** Both lineages ship identical bundle IDs
    (`org.openlogi.openlogi` / `.agent` / `.overlay`), the same `openlogi://` scheme, the
    same `~/.config/openlogi` + `openlogi-dev` data dirs, and the same socket namespace
    (MEASURED: `crates/openlogi-core/src/brand.rs` and `paths.rs` on both sides). A fork
    build and an upstream build on one machine fight over TCC grants, singleton locks,
    and the user's config file — migration is a flag day, not a dual install.

---

## 2. Topology

### 2.1 True fork point

```
git merge-base master upstream/master      → exit 1 (no common ancestor)      [MEASURED]
fork root 96a2beea  2026-08-26 11:31  "Restore UI error fix"
upstream v0.8.0    77a4a9b  2026-08-25 23:30  "chore: release v0.8.0 (#991)"
git diff --shortstat 96a2beea v0.8.0  → 83 files, +296/−1,578                  [MEASURED]
```

Nearest-tag tree distances from the fork root: v0.8.0 = 83 files (min), v0.8.1 = 113,
v0.7.7 = 314. **Conclusion: the fork snapshot was taken from upstream v0.8.0 within ~12 h
of its release, then re-rooted into a fresh 61-commit history.** All "fork has X since
0.8.3" statements in older artifacts must be read as fork-local version numbers.

### 2.2 Commit counts (MEASURED, combined object store)

| Direction | Count |
|---|---|
| upstream `v0.8.0..upstream/master` | **1,033** |
| upstream `v0.8.3..upstream/master` | 946 (803 to `v0.9.0`, +143 after) |
| fork `96a2beea..master` (its whole history) | 60 (61 incl. root) |
| fork `0.8.3(tag)..master` | 17 (prompt said 16; `b6451aa` was added after the prompt was written) |

Upstream releases in range: v0.8.4 … v0.8.21-beta.58, v0.9.0 (20 tags).

### 2.3 Conflict hot-zones (files changed on **both** sides since v0.8.0; MEASURED)

146 files. By area: `openlogi-ui` 22, `openlogi-hidpp` 21, `openlogi-desktop` 19,
`openlogi-core` 16, `.github` 15, `openlogi-agent-core` 12, `openlogi-hook` 6,
`openlogi-inject` 5, `openlogi-device` 4, `openlogi-overlay` 3, `openlogi-ipc` 2, plus
packaging/docs/READMEs. Full list recomputable with:
`comm -12 <(git diff --name-only v0.8.0 master|sort) <(git diff --name-only v0.8.0 upstream/master|sort)`.

---

## 3. Feature matrix

Legend — fork/upstream status: ✅ present · ❌ missing · ⚠ diverged.
Cost: S ≤ 1 day, M 1–3 days, L > 3 days or cross-crate. Costs assume rebasing onto the
upstream tree (the recommended direction, §6), not cherry-picks onto the fork.

### 3.1 Upstream-only feature clusters

| Feature | Upstream evidence | Fork | Cost | Rec. |
|---|---|---|---|---|
| **Flow** cross-machine control (crate, `[flow]` config, IPC, GUI window, pairing/SAS, QUIC keep-alive fix `d288163e`) | `openlogi-flow/src/{frame,identity,negotiation,pairing,sas,session,transport/tls}.rs` (~8,015 LoC, `ef577fb2`); `agent-core/src/flow/` (~9,152 LoC, `5925e23c`,`095094d7`); `config/flow.rs` (557 L, `72749479`); `desktop/src/windows/flow/{arrangement,inspector,pairing}.rs` (`6124f8d2`,`6d01ac3a`) | ❌ | L | Defer (§6.1) |
| **Actions Ring 0.9**: floating translucent slots, tailed labels (`de755247`), direction aiming `ActionRingSlot::toward` (`a0cafb1a`,`9c9c1dd7`), per-device `ActionRingSize` (`75f92fd1`), select-on-held-release (`466b3bd4`) | `openlogi-overlay/src/ring.rs` (1,266 L vs fork 326), `binding/action_ring.rs`, agent-core `action_ring.rs` `release()` | ❌ | M | Adopt |
| **Camera**: pan/tilt/gain/backlight (`#1020`), stream modes (`#2185`), USB-link speed (`usb_link.rs`), reset-to-defaults/profile persistence fix (`80cb0d33` #2314), refuse-busy-camera (`c4e63c4c`) | `openlogi-camera/src/{stream_mode,usb_link,mf_formats}.rs`, `macos/formats.rs`, `capture_macos.rs` config-lock | ⚠ base capture+UVC present (`capture.rs`,`uvc.rs` = renamed ancestors) | M–L | Adopt |
| **Device write modules** (upstream 22 vs fork 10): `button_spy` 483L, `disable_keys` 187L, `dpi_led` 228L, `easy_switch` 817L, `force_sensing` 406L, `key_display` 454L, `os_layout` 613L, `pointer_scaling` 289L, `report_rate` 714L, `sidetone` 242L, `onboard` 75L, `unit_id` 52L | `crates/openlogi-device/src/write/` | ❌ (fork hidpp already carries decoders for disable_keys/report_rate/sidetone/force-sensing/lock-keys) | S–M each; `easy_switch` L | Adopt; `easy_switch` is Flow's prerequisite |
| Expanded shared writes (dpi 332→1214, lighting 460→1954, litra 312→2949, smartshift 447→1180, backlight 206→800, haptic 338→1112 L) — reply-confirmation + capability-gating hardening (0.8.14 sweep) | same dir | ⚠ thin bases | L each | Adopt wholesale |
| Registry classes camera/headset/keypad/g600; Unifying slot naming (`ee51ef5b`); DPI-read fix (`22eb8847` #2321) | `openlogi-device-registry/src/{camera,headset,keypad,g600}.rs` | ❌ (fork: lib/litra/receiver only) | S–M | Adopt |
| **Agent-core extras**: `lock_keys` (`#1898`), `notices` low-battery (`#1891`), `backlight_overlay` (`#1695`), `pointer_speed` (`#1922/#1932`), `test_support` | `agent-core/src/{lock_keys,notices,backlight_overlay,pointer_speed}.rs` + overlay consumers `lock_key.rs/notice.rs/backlight.rs` (fork overlay lacks all three) | ❌ | M each | Adopt |
| Virtual HID keyboard injection, macOS (`#1918/#1991`, entitlement-gated) | `agent/src/virtual_keyboard.rs`, `inject/macos/virtual_keyboard.rs` | ❌ | L | Adopt with signing profile |
| Easy-Switch host UI (`#1825`, `60c1abef`); disable-keys UI; wheel-hold config (`#1828`); momentum scrolling (`#1994`); glide shape | `desktop/src/features/{keyboard/host_switch,keyboard/disable_keys,pointer/momentum,pointer/glide_shape}.rs`, `config/wheel_hold.rs` | ❌ | M each | Adopt |
| Bindings: HoldModifier (`74142fba` #1363), keypad keys in shortcuts (`214c9cf1`), G-series G-key/M-profile over 0x8010 (`1d4bf5bf`,`8f55ed03`), custom-shortcut Tap/Hold + press-record (`#1396/#2138`), cross-platform app aliases (`#1907`) | `binding/value.rs`, `features/binding_editor.rs`, `AppAliases` | ❌ (fork has its own separate `custom_shortcuts`/`SavedShortcut` — see §3.2) | S–L | Adopt; reconcile with fork shortcuts (§4.C) |
| Sparkle auto-update + beta channel (`#1744/#1925/#1980`), `prerelease.yml`, appcast in xtask | `xtask/src/support/update_feed.rs`, `release_tag.rs` | ❌ | M | Adopt |
| Linux: pkexec udev install (`60235e87`), grant-input-access in Settings (`bc6a2966`), Hyprland/KWin chords (`36db8cba`,`59ac734a`) | agent + hook linux/ | ❌ | M | Adopt |
| `openlogi://` deep links (`455cb2b0`,`f9b9e0c0`); menu-bar tray rebuild `tray_model` 806 L (post-0.9.0 still settling: `82de2cef`,`e9ef8338`,`f9b31aaf`) | `agent/src/tray_model.rs` | ⚠ fork `tray.rs` 358 L | L | Adopt after settling |
| macOS permission-grant drag panel (`8be1836d` #2492); CLI `diag hidpp` (`b689e88a`), `diag zones` (`3a2b08c0`) | desktop / cli | ❌ | S–M | Adopt |
| Fixture/replay test platform (§4.E) | `fixtures/devices/` (11 devices), `openlogi-fixture`, `device/src/replay/`, `hid/src/recording/`, `openlogi-hidpp/src/channel/{mock,observation}.rs`, `cli cmd/fixture|api`, `docs/MOCK_DEVICE_TESTING.md` | ❌ | M | Adopt |

Already shared (present at the v0.8.0 base, verified both sides): RunShellCommand,
OpenApplication, MiddleClick, power-user actions, gestures, per-app/per-device profiles,
action ring v1, DPI presets, SmartShift, fn-lock, lighting, litra, thumbwheel,
camera base, assets system.

### 3.2 Fork-only features (upstream lacks all of these)

| Feature | Fork evidence | Keep-cost on upstream base | Notes |
|---|---|---|---|
| Scroll acceleration engine: `CurvePreset` (13 variants), fixed-point `ScrollLut`/`ShapeLut`, `gain_at_rate_preset`, EMA rate, `compute_gains_fixed` | `openlogi-core/src/math.rs` (`FIXED_POINT_SCALE=1000`), `agent-core/src/runtime/scroll/acceleration.rs:124`, `9fe59ee1`, `859ee947` | M | The marquee fork feature; bench §5.2 |
| macOS `CVDisplayLink`-synced scroll decay + tap-thread `QOS_CLASS_USER_INTERACTIVE` + allocation-free `is_trackpad_like` | `agent-core/src/runtime/scroll/display_link.rs`, `hook/src/macos.rs:69-73`, `0889b86d` | M | Upstream parks on fixed 8 ms `FRAME_PERIOD`; no QoS call anywhere in its hook (grep, MEASURED) |
| Wheel quantizer idle/direction/axis clearing (single-step responsiveness) | `inject` `ScrollQuantizer::quantize_at` (lineage of `859ee947`; the titled commit `9881e6d1` is itself cosmetic — MEASURED 23+/7−) | S | Upstream's `ScrollQuantizer` (`inject.rs:709`) is a bare residual carry |
| `zero-alloc advance_due` (8-slot const due list) + precomputed curve samples + criterion bench | `agent-core/src/runtime/scroll.rs:336`, `benches/scroll_engine.rs`, `cedd9203` | S | Upstream `advance_segments` allocates a `Vec` per frame |
| `[[custom_shortcuts]]` + `Action::SavedShortcut` + shortcut builder UI + Win/Fn modifiers + ActionComposer custom targets | `config.rs:99` (`CustomShortcutEntry`), `binding/action.rs:196`, `d1310eaa`,`a93eb79d`,`67008723` | M | Upstream's own binding editor (`#1396/#2138`) covers overlapping ground — merge, don't duplicate (§6.4) |
| Generic mouse fallback + Logitech G90 support | `f5a79f58` discovery | S | Pure additive |
| M720 gesture button CID 0x00d0 | `89cb3df1` | S | |
| macOS double-left-click dispatch fix | `d7afef61` | S | Verify upstream didn't independently fix |
| Win11 overlay drop-shadow artifact fix | `4d723624` | S | Overlay `main.rs`/platform |
| App-profile removal targeting fix | `40b04cef` | S | |
| `macos/watchdog.rs` callback watchdog | fork hook | M | Upstream took a different path (`81190faf` power-epoch); reconcile (§4.A platform) |
| Branding: logo/icns/svg/tray, `.wix/` vendored extensions, `design/` fork art, `OpenLogi.wxs` | `c2ce3c5f`…`5c365cc6` | S | Reapply over upstream tree |
| Tooling: `.claude/rules/**` (10), `prek.toml`, `release-plz.toml` (fork variant), `docs/DEVELOPMENT.md`, audit artifacts | repo root | S | Fork-only workflow |

### 3.3 Diverged pairs (both sides rewrote the same file)

- `openlogi-hook/src/macos.rs`: 356+/552− — upstream split it into
  `macos/{foreground,grant,pointer,probe,secure_input,sender,translate}.rs` **and** reworked;
  fork kept the monolith + added `watchdog.rs` (1375+/73− vs upstream's expanded watchdog).
- `openlogi-agent/src/main.rs`: 77+/496− — upstream extracted `startup.rs`/`shutdown.rs`/
  `lifecycle/`; fork added dev-agent teardown + display-link plumbing in main.
- `openlogi-agent/src/tray.rs`: 1024+/141− — upstream grew + split `tray_model/`.
- `openlogi-inject/src/inject.rs`: 1110+/222−.
- Camera `capture.rs`→`capture_macos.rs`: 353+/61− (git-detected rename; additive).
- `crates/openlogi-hidpp`: **101 files differ** (MEASURED) — the 0.8.14/0.8.16 protocol
  fix sweep plus new feature decoders (§4.F).

---

## 4. Focus-area answers

### A. Feature inventory delta
See §3. Verified against implementations, not changelog lines. Flow ships: protocol crate
+ tests (`tests/quic_protocol.rs`), opt-in `[flow]` config, 7 IPC types/methods, GUI
inspector/arrangement/pairing window, agent edge-detection/handoff — gated and young
(`enabled: false` default, `config/flow.rs:394`). Actions Ring 0.9, camera features, and
the 12 upstream-only write modules are each individually verified with paths in §3.1.
Fork-only features enumerated honestly in §3.2.

### B. IPC protocol (highest risk)

- `PROTOCOL_VERSION`: **29 (fork) vs 46 (upstream)** (MEASURED).
- Across 29→46 upstream added **24 wire-visible types** (`FlowIdentity`, `FlowStatus`,
  `FlowPeerStatus`, `FlowLinkState`, `FlowPair{Target,Phase,Failure,CommandError}`,
  `FlowPairingPeer`, `Notice`, `NoticeObservation`, `NoticeKind`, `LockKeyChange`,
  `LockKeyObservation`, `BacklightObservation`, `ObservedLight`, `ChosenLight`,
  `SavedConfig`, `SavedConfigVerdict`, `SecureInput`, `VirtualKeyboardStatus`,
  `ClientKind`) and **33 RPC methods** (Flow pairing ×7, `observe_{notices,lock_keys,
  backlight}`, `read/set_{backlight,fn_lock,disable_keys,force_sensing,pointer_speed,
  report_rate,sidetone}`, `read_{onboard,wheel,easy_switch_hosts}`, `switch_host`,
  `rename_easy_switch_host`, `unpair_device`, `declare_client`,
  `reload_saved_config[_with_verdict]`, `action_ring_unhover`), with **zero deletions**
  (MEASURED: set-diff of `pub enum|struct` decls and trait methods in both `ipc.rs`).
  Append-only discipline held; shared-enum variants were appended (e.g. `ActionRingSize`
  rides `ActionRingInvocation`, `ipc.rs:833` upstream).
- `crates/openlogi-ipc`: 16 files, +2,729/−82 (MEASURED); upstream's
  `tests/wire_format.rs` grew +965 lines pinning the contract; upstream added
  `src/testing.rs` (fork lacks it).
- **Mixing fork and upstream binaries mid-integration:** the client judges the agent's
  `protocol_version` answer and refuses in both directions ("waiting to be replaced" /
  "this process needs a relaunch", upstream `client.rs:81-92`). So a fork GUI + upstream
  agent will not corrupt state — but it will also not work. Because bundle IDs, socket
  names and data dirs are identical (§7), any half-migrated tree on a dev machine will
  fight the installed app. Integration must be whole-tree, whole-bundle.

### C. Config schema & user data

- Layout: fork `config.rs` (1,071 ln) + 6 submodules (6,302 ln total); upstream
  `config.rs` re-export shim (247 ln) + 15 submodules + `tests/` (6,315 ln). The prompt's
  "single file" premise is outdated — the fork is already semi-modular at the same cut
  points (`device/file/identity/key_trigger/settings`).
- `SCHEMA_VERSION`: fork **7**, upstream **11** (MEASURED). The ≤6 migration lineage is
  identical on both sides (v2/v4/v5/v6 semantics match verbatim); both then independently
  bumped 6→7 for different changes — a **version-number collision** that any translator
  must resolve (renumber fork v7 ⇒ ≥12).
- Shared `[devices.*]` keys verbatim: `enabled, custom_name, identity, links, bindings,
  disabled_gestures, per_app_bindings, action_ring, dpi_presets, dpi, lighting, light,
  smartshift, camera_controls, camera_profiles, camera_profile, thumbwheel_sensitivity,
  invert_scroll, scroll_resolution, host_switch_targets, fn_lock`. Upstream-only v8–v10
  additions: `camera_stream_mode, smooth_wheel, momentum_scrolling, keep_os_layout,
  report_rate, pointer_speed, force_sensing, disabled_keys, onboard_memory,
  gshift_bindings, keypad_pages, g_key_owner, gaming_key_mode, g_key_profile_names`;
  top-level `[flow]` and `[apps]` alias table (v9). Fork-only device keys: **none**.
- Fork-only `[app_settings]`: the whole scroll-curve model
  (`vertical/horizontal_scroll_baseline` (with `alias = "vertical_scroll_sensitivity"` /
  `"thumbwheel_sensitivity"`), `vertical/horizontal_curve_preset`,
  `vertical/horizontal_multiplier`, `legacy_*` keys) and top-level `[[custom_shortcuts]]`.
  Upstream-only: `smooth_scroll`, `momentum_scrolling{...}`, `update_channel`,
  `mouse_profile_target`, `low_battery_notifications`, `lock_key_notifications`,
  `animate_desktop_switch`, `update_channel`, app-level `thumbwheel_sensitivity`.
- **Would upstream's `migrate.rs` accept a fork-written config unchanged? No.**
  `deny_unknown_fields` on `Config`/`AppSettings`/`DeviceConfig` (both sides) turns every
  fork-only key into a hard load error; only a fork file with all-default `[app_settings]`,
  no saved shortcuts, and no `SavedShortcut` bindings loads cleanly. Upstream → fork is a
  hard reject too (`11 > 7` → `UnsupportedVersion`). No silent loss at load in either
  direction (both keep `backup_existing_config`), but **one silent semantic risk**:
  a fork file descending from a pre-v6 upstream file carries old thumbwheel pairs;
  upstream's `migrate_thumbwheel_native_direction` is gated `version ≤ 6`
  (`openlogi_0.9 .../config/file.rs:339`) while the fork file claims 7, so upstream would
  skip the swap and flip thumb-wheel direction. (INFERRED from code; fork lacks the
  migration entirely — grep, MEASURED.)
- Upstream-only APIs with no fork counterpart: `config/edit.rs` (`ConfigEdit`/`DeviceEdit`
  surgical on-disk edits), `config/revision.rs` (`ConfigRevision`, reported over IPC
  `45408032`).
- **Verdict: a real merge needs a one-time translator** (fork v7 ⇒ upstream ≥12): rename
  baseline keys back to upstream names, map `CurvePreset`/multiplier to nearest upstream
  sensitivity or retain as a fork-side extension table, re-home `[[custom_shortcuts]]`
  into upstream's binding editor, renumber, and insert the thumbwheel pass at the fork's
  old-v6 position.

### D. Locale system

- Fork: 22 `*.yml` catalogs, **English-text-as-key** (`"Grid": "Grid"`), 490 en keys,
  521 `tr!` call sites; negotiation in `openlogi-ui/src/locale.rs`
  (`fluent_langneg` + `sys_locale`); parity test in-crate.
- Upstream: 24 `*.toml` catalogs (adds `be`, `cs`), **nested semantic keys**
  (`[common] grid = "Grid"`), 1,016 en keys, 977 `tr!` sites; `locale.rs` moved to
  **openlogi-core** so the headless agent tray can translate; parity driven from
  `openlogi-ui/tests/catalog_parity.rs`; `english_fill_in_report.py` CI step.
- Both still use `rust_i18n` 4.2.1, the same `.config/crowdin.yml` (diff is exactly
  `type: yaml→toml`, `en.yml→en.toml`, +`be`/`cs`), and the same
  `.github/scripts/i18n/merge_crowdin_download.py` idea (fork: YAML-regex variant;
  upstream: `tomllib` rewrite + report script).
- **Keys are not 1:1 translatable** — each side has feature vocabulary the other lacks.
- **Cheaper convergence: fork adopts upstream's format + core-resident `locale.rs`.**
  Upstream is the maintained superset; the reverse would force upstream to rewrite 977
  call sites and abandon its tooling. What breaks either way: Crowdin translation memory
  keyed on the old format, and every fork-only string (acceleration UI, shortcuts) needs
  new `en` keys before parity passes. Cost: **L** (521 call sites × 2 binaries, 22
  catalogs, Crowdin project reconfiguration).

### E. Test & fixture infrastructure

- Upstream-only, verified: `fixtures/devices/` — 11 sanitized device corpora
  (`g512-001, g913→g915-001, mx-anywhere-3s, mx-creative-keypad, mx-ergo, mx-keys-s,
  mx-master-2s/3/3s/4, mx-mechanical-mini-mac`), each `manifest.json + profile.json +
  cases/*.json`; crate `openlogi-fixture` (host-free schemas + privacy verification,
  `fs` feature for strict corpus loading); `openlogi-device/src/replay/` (cassette-backed
  `HidBackend` below production code); `openlogi-hid/src/recording/` (bounded capture +
  sanitizer); `openlogi-hidpp/src/channel/{mock,observation}.rs`; profile/control/flow-aware
  `bin/mock_agent/` (fork: single hard-coded `mock_agent.rs`); CLI `fixture
  record|contribute|verify` + `api`; `docs/MOCK_DEVICE_TESTING.md`.
- Fork-only: `benches/scroll_engine.rs` (upstream has **zero** `benches/` dirs, MEASURED
  `ls-tree`) and exactly **1** `tests/*.rs` file workspace-wide (`ipc/tests/wire_format.rs`).
- Surface counts (MEASURED `grep -rl '#[cfg(test)]'`): fork **187** files vs upstream
  **444**; `*/tests/*.rs`: fork 1 vs upstream 92. Per-crate: device 23→54, hid 4→8,
  agent-core 20→61, agent 10→23, hook 6→26, inject 4→13.
- **Rewrite estimate (INFERRED):** the ~47 fork test files touching enumeration/channel/
  session semantics must be re-expressed against `openlogi-device::replay` + fixture
  profiles; pure-logic tests (config, math/scroll, identity, ipc) port nearly as-is. The
  scroll bench survives only if the acceleration engine is kept (§6.5).

### F. HID++ layer

- `crates/openlogi-hidpp`: **101 files differ** between fork `master` and upstream
  `master` (MEASURED `git diff --name-only | wc -l`; up from 54 at the 0.8.15 snapshot —
  upstream's 0.8.14/0.8.16 protocol sweep + new decoders account for the growth).
  Classification (upstream-side): protocol-correctness fixes (reply confirmation,
  capability gating, register event parsing) vs new-feature support
  (`lock_key_state`, `adc_measurement`, `pointer_motion_scaling`, `device_information`
  expansions, disable-keys 0x4521, report-rate 0x8060/0x8061, sidetone 0x8300,
  force-sensing 0x19c0, os_layout 0x4531). Notably the fork's vendored hidpp **already
  carries decoders** for disable_keys/report_rate/sidetone/force-sensing/lock-keys, which
  lowers several §3.1 port costs.
- `openlogi-hidpp-derive`: **byte-identical** between the trees (MEASURED: both tree
  hashes `67633a35`). The fork's copy is a verbatim early copy of upstream's (created
  upstream in `b7a5033a` #625, last touched `4cc8e554` #720, both before the fork point).
  Not a divergent port — zero reconciliation needed.
- Upstream's own fork rules: `openlogi_0.9/crates/openlogi-hidpp/AGENTS.md` (the fork
  keeps the same file, unchanged; upstream additionally maintains per-crate AGENTS.md for
  hook/device/ipc/hid/desktop).

### G. Performance → §5 (co-equal deliverable).

### H. CI, release & toolchain

- `rust-toolchain.toml` identical (`channel = "stable"`). `[workspace.lints]` nearly
  identical; upstream adds `assert_is_empty = "allow"` (clippy 1.99 pedantic) and dev
  `opt-level=3` for `liblzma-sys`/`zstd-sys`. **Fork-only merge hazard:**
  `[patch.crates-io] gpui = { git = zed-industries/zed, version = "=0.2.2" }` (swr-gpui
  dedup) — a real `Cargo.lock` conflict against upstream's `gpui-pre` stack (§5.4).
- Upstream-only machinery: ast-grep lint infra (`sgconfig.yml` + `.ast-grep/` at root and
  in 16 crate dirs, `cargo xtask lint`, CI job `ast-grep`, `stale-allows.yml` sweep, one
  extra `prek.toml` hook); Sparkle/appcast + beta-tag pipeline (`xtask/src/support/
  update_feed.rs`, `prerelease.yml`); CI jobs `ast-grep`, `test-windows`,
  `clippy-macos`, `ci-results`; flatpak + `packaging/windows/openlogi-installation.json`;
  per-crate `AGENTS.md` ×6; skills under `.agents/` (9, incl.
  `contributing-device-fixtures`, `gpui-kit`).
- Fork-only machinery: `.claude/rules/**` ×10 + macOS-permissions skill, vendored
  `.wix/extensions/` (Util + UI wixext 6.0.2 DLLs), fork `OpenLogi.wxs`, `flake.lock`
  in-tree, `release-plz.toml` fork variant (no prerelease path), `docs/DEVELOPMENT.md`,
  prek without ast-grep. **Correction to the prompt's premise:** `prek.toml`,
  `release-plz.toml`, `design/`, and `packaging/` exist on **both** sides (content
  differs); `.claude/rules/**`, `.wix/`, and the gpui `[patch]` are genuinely fork-only.
- Both share devenv/flake/`.envrc` and the same 8 base workflows (`build, ci, crowdin,
  devenv, nix, release, release-plz, update-flake-lock`).
- Recommendation: adopt upstream's CI/lint/release spine wholesale (additive), keep
  `.claude/rules` + prek config, re-derive the WiX patch set.

### I. Licensing & attribution

- Root `LICENSE-MIT` / `LICENSE-APACHE` are byte-identical between trees (MEASURED:
  `git diff --stat` empty). Upstream adds `design/LICENSE` (proprietary brand terms) which
  the fork does not carry — decide whether to inherit it or document the fork's own
  `design/` terms (fork brand art diverged since `c2ce3c5f`).
- No SPDX header conflicts found in the sampled areas; vendored `openlogi-hidpp` remains
  0BSD with its own AGENTS.md on both sides.
- Attribution: upstream history is fully local, so adopted features can land as rebased
  commits preserving original authorship (`Co-authored-by` where re-homed) — consistent
  with the project's contributor-adoption policy. No AI attribution in generated commits.

---

## 5. Performance report

### 5.1 Upstream perf-commit triage (all 9 diffs read; fork counterparts checked)

| SHA | Optimizes | Fork same shape? | Fork benefit |
|---|---|---|---|
| `268602eb` | Moves `info!` logging of captured actions out of the tap callback into workers (rolling-file-appender lock no longer inside the guarded callback) | **Yes** — fork `agent-core/src/runtime/hook.rs` logs at lines 278/298/367/410/478 on the callback path; fork uses `tracing_appender::rolling` | **Yes** — identical pre-fix shape |
| `e2d63f06` | Button worker blocks on `recv`/`recv_timeout(deadline)` instead of 10 ms polling | Partly — fork `runtime/button.rs:25,645` computes deadlines but still polls at 10 ms idle | Yes — idle-block + `Wake` command ports cleanly |
| `81190faf` | Watchdog reads `kern.sleeptime/waketime` sysctls only on entered-callback polls | No — fork watchdog is pure atomics, no sysctl | N/A (but fork also lacks the sleep/wake false-stuck discount) |
| `5145cdb9` | `AssetResolver` memo map (2–4 µs clone vs 5.1 ms opt / 49 ms debug per resolve) | Yes — fork re-resolves per device per snapshot (`state/devices.rs:168,297,374,561,565`) | Yes — worst case per foreground-app change |
| `bf2f30c2` | Build `AssetResolver` once at startup (0.85 ms opt / 8.4 ms debug ×3 sites) | Yes — same three sites (`runtime.rs:82,208`, `app.rs:204`) | Yes |
| `d5e2b880` | `tr!` returns `SharedString::new_static` for borrowed hits (no copy) | Yes — fork `main.rs:28` copies; `new_static` exists in fork's gpui rev | Yes — free, every repaint |
| `091b7aa2` | `tokio::sync::watch` notifications replace 1 Hz capture-state polling + native activation observer | Yes — fork polls at `gesture.rs:46`, `keyboard.rs:87`, `host_switch.rs:56`, `main.rs:507`; no registry watch | Yes, but L — needs device-layer registry watch + hook observer; removes ~4 wakeups/s steady-state |
| `8dbfde34` | One unified `cargo build --release` for GUI+helpers+CLI; skip `cargo bundle` rebuild | Yes — fork xtask builds each helper separately (`embed.rs:88,122`) then rebuilds GUI | Yes — release-pipeline time only |
| `d15c8acc` | Embed 512 px logo (−526,548 B; GPUI samples full-res without mipmaps) | Yes — fork `app_assets.rs:17` embeds the 1024 master; 512 px asset already in fork `design/` | Yes — one-line port |

### 5.2 Benchmarks — MEASURED (fork only)

Machine: Apple M5 Max, 18 cores. Command:
`cargo bench --offline -p openlogi-agent-core --bench scroll_engine -- --warm-up-time 1
--measurement-time 2 --sample-size 10` (criterion quick config; full pass completed,
exit 0). Criterion means:

| Benchmark | Mean | Note |
|---|---|---|
| `gain_at_rate_smooth` | **1.94 ns** | single LUT lookup |
| `gain_at_rate_sigmoid` | 3.44 ns | |
| `compute_gains_none_passthrough` | 3.91 ns | |
| `compute_gains_smooth` (f32 path) | 12.52 ns | per-event f32 curve eval |
| `compute_gains_fixed_lut` | **9.10 ns** | fixed-point LUT — ~27% faster than the f32 path |
| `lut_generation_smooth` | 210.7 ns | one-time per config change |
| `lut_generation_sigmoid` | 596.0 ns | |
| `lut_generation_exponential` | 555.7 ns | |
| `dispatch_latency_end_to_end` | 34.4 ns | |

**Upstream A/B: not measured.** Upstream has no criterion harness (zero `benches/`, no
criterion in its lockfile) and its scroll path is a *different model* (momentum/glide),
so a same-bench A/B is not meaningful; the comparison below is read-and-inferred per the
prompt's fallback.

### 5.3 Hot-path reading (INFERRED unless quoted above)

1. **Scroll engine.** Fork per-event gain = EMA (f32 FMAs) + array-index LUT; per-frame
   easing = f32 cubic polynomial; `advance_due` zero-alloc 8-slot array; 4 ms frames
   unparked by `CVDisplayLink`. Upstream: f64 throughout; shared easing is a cheap
   smoothstep, but the glide path evaluates two `exp()` + two `powf` per frame, launch
   uses `powf`/`powi`, and `advance_segments` heap-allocates a `Vec` per frame; fixed 8 ms
   `FRAME_PERIOD`. **Fork likely faster on the shared smoothing path; upstream's heavier
   math buys the momentum features the fork lacks.** Different features — not a pure
   perf gap.
2. **macOS tap callback.** Shared shape: both resolve device identity via
   `CGEventCopyIOHIDEvent` + sender-ID + thread-local cache, one `Option<String>` clone
   per event (near-identical code, fork `hook/src/macos.rs:174-218` vs upstream
   `macos/sender.rs:120-163`). Fork extra costs: `info!`-in-callback with blocking
   appender lock (§5.1/`268602eb`). Fork extra wins: tap QoS `USER_INTERACTIVE`,
   allocation-free `is_trackpad_like` (upstream lowercases a `String` per
   `source_is_remappable` call, `lib.rs:89/102`, though it short-circuits when scroll
   inversion is off, upstream `macos.rs:364`). **Upstream likely faster in the callback
   body; fork partially compensates via QoS.**
3. **Hook→agent scroll dispatch.** Fork: lock-free `crossbeam ArrayQueue(256)` +
   `thread::unpark`. Upstream: `sync_channel(128)` (mutex + condvar). No serialization
   either side. **Fork likely faster per event; margin small vs the FFI work in (2).**
4. **IPC round-trip.** Identical machinery and cadence: tarpc 0.38 + bincode +
   `interprocess` 2.4.2, long-poll `observe()` held 20 s on both sides (no fixed GUI
   poll). Upstream's polled `AgentSnapshot` is strictly larger (`lights`, `flow`,
   config-revision, `secure_input`, `virtual_keyboard`). **Fork marginally faster per
   snapshot (µs); irrelevant in practice.**
5. **Device probe/inventory caching.** Fork = v0.8.0 shape (scan-count refresh, 2 s poll
   watcher). Upstream: time-based 30 s refresh with 8 s walk budget, battery excluded from
   cache window, separate `channel_cache.rs`, dead-channel stand-in marking (`d29fbc30`),
   warm-cache preservation across failed probes (`c976fa94`), event-driven watcher.
   **Upstream clearly more mature (correctness + fewer wakeups).**
6. **Overlay frame loop.** Both pure-IPC GPUI siblings, no animation timers, redraw on
   IPC/pointer events. **Equivalent** (upstream draws more only when its extra surfaces
   are active).
7. **Agent startup.** Fork serial ladder: consent prompt → orchestrator → watchers →
   **then** IPC bind (first-run socket waits on the user). Upstream binds IPC first
   (`startup.rs:50-53`), seeds permission facts non-prompting, defers prompts/device I/O
   to an arm ladder, event-driven watchers. **Upstream likely better, especially
   first-run connect latency.**

### 5.4 Dependency drift (lockfiles diffed; MEASURED versions)

| Item | Fork | Upstream | Implication |
|---|---|---|---|
| Package count | 1,023 | 1,041 | composition differs more than count |
| async-hid | `openlogi-async-hid 0.5.3-openlogi.1` | same name/version/checksum | none — shared republished fork |
| tokio / tarpc / interprocess / rust-i18n | 1.53.1 / 0.38.0 / 2.4.2 / 4.2.1 | 1.53.3 / 0.38.0 / 2.4.2 / 4.2.1 | patch drift only |
| rustls | 0.23.40 | 0.23.46 | patch drift |
| QUIC (quinn 0.11.9) | pulled by `zed-reqwest` (HTTP/3 asset fetch) | pulled by **`openlogi-flow`** (Flow data plane) + reqwest | same stack, opposite purpose |
| **GPUI** | `gpui 0.2.2` git `zed-industries/zed@55007f5` + gpui-component 0.5.2 (git rev) + fork `[patch]` | **`gpui-pre 0.3.10` (crates.io, longbridge gpui-kit)** + gpui-component v0.8.1 + `gpui-fast` (off by default) | **biggest drift in the workspace** — different renderer lineage; every GUI-merge assumption (e.g. `SharedString::new_static`) lives in a different stack |
| Upstream-only heavy adds | — | flow (quinn+rustls+`mdns-sd`+`if-addrs`+protobuf codegen), fixture, `rcgen`/`x509-parser`, `minisign`, `gpui-updater-*`, `liblzma`/`zstd`, `ar`, `miette`, `scrypt`/`salsa20`, `rpassword` | Flow is upstream's largest new runtime surface |
| Fork-only | criterion+plotters, `zed-reqwest`, `workspace-hack`, `fsevent-sys` | — | fork is the only lineage with executable micro-benchmarks |

### 5.5 Measured vs inferred — explicit split

- **Measured:** bench table (§5.2); all commit-diff readings and line citations in
  §5.1/§5.3 (code facts); lockfile versions/counts; tag/tree/merge-base facts in §2.
- **Inferred:** every "likely faster/slower" in §5.3; fork benefit columns in §5.1;
  test-rewrite estimate (§4.E); Flow→Easy-Switch dependency (§3.1); all runtime-behavior
  claims about hardware, first-run consent latency, and burst behavior.
- **Not measured:** upstream A/B bench (no harness; different model — see §5.2),
  release-size/startup comparison (`cargo build --release` both trees not run — left as
  optional per prompt §G5), tap-callback latency under event storms.

---

## 6. Adoption plan input (propose, do not execute)

Recommended overall direction: **migrate the fork onto the upstream tree** (take upstream
as the new trunk, re-apply fork-only work), *not* cherry-pick upstream into the fork.
Evidence: 1,033 vs 61 commits, 444 vs 187 test files, upstream holds the entire fixture/
CI/release spine, and the fork's 146-file hot zone would otherwise absorb every upstream
rewrite by hand. Per contested cluster:

1. **Flow** — *Option A:* skip for now (fork has zero dependencies on it). *Option B:*
   adopt wholesale with upstream tree; it arrives automatically on an upstream-trunk
   migration, stays opt-in, but drags Easy-Switch writes + mDNS/QUIC/TLS dependency
   surface + a young pairing UX. **Rec:** take upstream's tree (B by default) but keep
   `[flow] enabled=false` and defer user-facing promotion until upstream settles; do not
   port Flow onto the current fork as a separate effort.
2. **Actions Ring 0.9** — *A:* keep fork ring (326-L overlay). *B:* adopt upstream
   ring/selection model (touches core+IPC+GUI+overlay, M). **Rec: B** — it rides the IPC
   append-only path and the fork's ring has no fork-specific behavior worth preserving.
3. **Config split** — *A:* keep fork `config.rs` + hand-port features (each upstream
   feature then re-implements per-key plumbing). *B:* adopt upstream `config/` module +
   write a one-time fork-v7→v12 translator (§4.C). **Rec: B**; the translator is ~1 file
   plus migration tests, and upstream's `edit.rs`/`revision.rs` are prerequisites for
   several upstream features. Decide in the same pass whether the fork's scroll-curve keys
   become an upstream-extension table or are translated away.
4. **Locales** — *A:* keep yml/English-as-key (fork diverges forever from upstream
   locale work). *B:* adopt upstream toml+semantic-keys+core locale (L: 521 call sites,
   22 catalogs, Crowdin remap). **Rec: B**, bundled with the migration PR-series; fork-only
   strings (acceleration UI, shortcut builder) get new `en` keys first.
5. **Scroll model (fork's crown jewel)** — *A:* keep fork engine as the trunk's
   acceleration layer and add upstream's momentum/glide alongside (two models, config
   switch). *B:* keep fork engine, skip upstream momentum features. *C:* adopt upstream
   momentum, drop fork curves (loses the fork's headline differentiator). **Rec: A** —
   port the fork's `math.rs` + `acceleration.rs` + `display_link.rs` + bench onto
   upstream's scroll worker as an optional per-axis acceleration stage; upstream's
   per-frame `Vec` allocation and fixed 8 ms park are then also worth taking the fork's
   `advance_due`/display-link improvements into. This is the one cluster where
   fork→upstream contribution is the right direction.
6. **Fixtures/testing** — *A:* keep fork unit tests (1 integration file). *B:* adopt
   fixture/replay/cassette platform and re-express ~40–50 hardware-layer tests. **Rec: B**;
   it is the only way upstream's device fixes stay verifiable without hardware.
7. **Hook/camera/platform naming** — *A:* keep fork monoliths (every future upstream fix
   becomes a manual rename-port). *B:* adopt upstream's split layout; re-home fork-only
   `watchdog.rs`, QoS boost, allocation-free helpers, camera `capture.rs`→`capture_macos.rs`.
   **Rec: B** — the relocation table (§3.3, §4 of the platform report) makes it mechanical.
8. **CI/lint/release stack** — *A:* keep fork stack. *B:* adopt upstream spine
   (ast-grep, stale-allows, ci-results, test-windows, Sparkle+prerelease) keeping
   `.claude/rules` + prek + release-plz fork variant. **Rec: B**, plus resolve the gpui
   `[patch]` against `gpui-pre` (may become unnecessary if gpui-kit dedups swr-gpui —
   verify with `cargo tree` post-switch).

### Suggested migration order (dependency-driven)

1. Toolchain spine (CI/lint/release, §6.8) + GPUI stack switch — validate the build first.
2. Upstream trunk + fork-only re-application: branding, WiX, dev-agent teardown, hook
   watchdog/QoS, double-click fix, Win11 shadow fix, G90/M720, quantizer clearing.
3. Config translator + upstream `config/` module (§6.3) and locales (§6.4) as one
   user-data cutover release with migration tests.
4. Scroll engine contribution back (§6.5) + Actions Ring 0.9 (§6.2).
5. Fixture platform (§6.6) + device write-module adoption (upstream wholesale).
6. Flow last, behind its existing opt-in gate.

---

## 7. Risk register

| # | Risk | Severity | Evidence / mitigation |
|---|---|---|---|
| R1 | **Identity collision**: identical bundle IDs (`org.openlogi.*`), `openlogi://` scheme, `~/.config/openlogi(-dev)`, socket names → fork & upstream builds fight over TCC grants, singleton locks, config file | High | MEASURED `brand.rs`/`paths.rs` both sides. Mitigation: flag-day cutover; never install both on one machine during migration; dev profile (`openlogi-dev`) is the only safe sandbox |
| R2 | **IPC 29→46 mixing** mid-integration → GUI/agent/overlay refuse each other | High (but loud) | Handshake refuses both directions (`client.rs:81-92`); ship all three binaries together; bump-fork PRs must run `cargo test -p openlogi-ipc --test wire_format` |
| R3 | **User config rejected at first launch after cutover** (deny_unknown_fields + v7/v11 collision) | High | §4.C translator + tests before any upstream-sync release; keep `backup_existing_config`; never let the app silently "start fresh" over an upstream-schema file |
| R4 | **Thumbwheel-direction silent flip** for users whose file descends from pre-v6 upstream | Medium | Insert `migrate_thumbwheel_native_direction` at the renumbered fork v6 position (§4.C); add a fixture test |
| R5 | **GPUI lineage switch** (Zed 0.2.2 → gpui-pre 0.3.10): renderer regressions, Metal shader build needs, `[patch]` obsolescence, `SharedString` API moves | High | Validate in step 1 of the migration order; `cargo tree` the swr-gpui dedup after the switch |
| R6 | **TCC/permission identity drift**: upstream's `input_monitoring.rs`/`grant.rs`/`seed_permission_facts` assume its helper-launch tables; fork's `launch_agent.rs` successor is `autostart/macos.rs` | Medium | Re-run the macOS-permissions skill checklist (`.claude/skills/openlogi-macos-permissions`) after adopting upstream autostart; verify deep links still resolve |
| R7 | **Scroll-model regression for existing fork users** if momentum replaces curves (glide shapes, `FRAME_PERIOD` feel) | Medium | §6.5 option A keeps both; migration must map `CurvePreset`/multiplier keys or explicitly present the choice at first launch |
| R8 | **Fork's thin shared write modules** (dpi/lighting/litra/smartshift) lack upstream's reply-confirmation hardening; porting dependent features first can expose silent write failures on real hardware | Medium | Take upstream write modules wholesale before adopting dpi_led/report_rate/etc. |
| R9 | **Upstream tray_model still mid-rebuild** at master (dense post-0.9.0 race fixes) — adopting it early imports churn | Low-Med | Pin to a post-settle upstream SHA; watch the fix list (`82de2cef`…`280a3fca`) |
| R10 | **Flow dependency chain** (Easy-Switch writes → host-switch IPC → pairing UX) means partial adoption leaves handoff inert | Low (gated off) | Adopt only with its dependency cluster; keep `enabled=false` |
| R11 | **`openlogi_0.9/` is untracked and not gitignored** — a careless `git add -A` commits ~1,200 files | Medium | Maintainer checklist item from the prompt; add to `.gitignore` before further agent runs (not done here — audit-only) |

---

## 8. Not-verified list

- Runtime behavior on real hardware for both lineages (all of §5.3's "likely" claims, tap
  latency under event storms, ArrayQueue-vs-sync_channel burst semantics, first-run
  consent latency).
- Upstream scroll-path performance by measurement (no harness exists; read-and-inferred
  per prompt §G2 fallback).
- Release-size/cold-start comparison (`cargo build --release` both trees not run — the
  optional §G5 step).
- Windows/Linux hot paths (evdev/uinput, SendInput, Windows channel-cache failure mode) —
  unauditable from macOS per the project's own `cross-platform.md` rule.
- GPUI renderer performance delta (Zed gpui 0.2.2 vs gpui-pre 0.3.10 internals, mipmap
  claim in `d15c8acc`).
- Whether upstream independently fixed the fork's double-click dispatch (`d7afef61`) —
  not searched commit-by-commit.
- Attribution of the quantizer-clearing behavior to a specific fork commit (the titled
  commit `9881e6d1` is cosmetic; bisecting the `859ee947` merge's parents was out of
  scope).
- Upstream's `flake.lock` gitignore status; the `gist.github.com` lint-gist identity.
- Maturity of the 143 post-0.9.0 commits beyond the fix-list density signal (tray churn,
  Flow pairing re-typing) — needs runtime soak.
- GitHub-side facts (PR history, Greptile, release-plz runs) — audit used the local clone
  only, per prompt: the clone is a fresh full fetch, and no clone-vs-tag disagreement was
  encountered.

---

### Reproduction commands (all read-only)

```sh
# topology
git clone --local file://$PWD $TMPDIR/audit/fork && cd $TMPDIR/audit/fork
git remote add upstream $PWD/openlogi_0.9 && git fetch upstream --tags
git merge-base master upstream/master                 # → exit 1 (unrelated histories)
git diff --shortstat 96a2beea v0.8.0                  # → 83 files, +296/−1578
git rev-parse 0.8.3^{tree} v0.8.3^{tree}              # → different trees
# hot zones
comm -12 <(git diff --name-only v0.8.0 master|sort) \
         <(git diff --name-only v0.8.0 upstream/master|sort)
# ipc surface delta
git show master:crates/openlogi-ipc/src/ipc.rs | grep -oE 'pub (enum|struct|trait) [A-Za-z0-9_]+' | sort -u
git show upstream/master:crates/openlogi-ipc/src/ipc.rs | grep -oE 'pub (enum|struct|trait) [A-Za-z0-9_]+' | sort -u
# bench (fork, measured 2026-11-13 on Apple M5 Max)
cargo bench --offline -p openlogi-agent-core --bench scroll_engine -- \
  --warm-up-time 1 --measurement-time 2 --sample-size 10
```
