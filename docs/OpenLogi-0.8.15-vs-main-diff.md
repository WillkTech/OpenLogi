# Diff: OpenLogi-0.8.15 (upstream v0.8.15 snapshot) vs main (WillkTech fork @ 9c40741)

Generated 2026-10-21. Snapshot dir: `OpenLogi-0.8.15/`. Excludes `.git`, `target`, `.DS_Store`.

**Totals: 432 differing files · 58 only in main · 256 only in snapshot**

## Differing files (in both, content differs)

```
.cargo/config.toml ⇄ .cargo/config.toml differ
.cargo/deny.toml ⇄ .cargo/deny.toml differ
.claude/rules/ci.md ⇄ .claude/rules/ci.md differ
.claude/rules/cross-platform.md ⇄ .claude/rules/cross-platform.md differ
.claude/rules/gui.md ⇄ .claude/rules/gui.md differ
.claude/rules/i18n.md ⇄ .claude/rules/i18n.md differ
.claude/rules/ipc-protocol.md ⇄ .claude/rules/ipc-protocol.md differ
.claude/rules/objc-ffi.md ⇄ .claude/rules/objc-ffi.md differ
.claude/rules/rust.md ⇄ .claude/rules/rust.md differ
.claude/rules/xtask.md ⇄ .claude/rules/xtask.md differ
.claude/skills/openlogi-macos-permissions/SKILL.md ⇄ .claude/skills/openlogi-macos-permissions/SKILL.md differ
.config/cliff.toml ⇄ .config/cliff.toml differ
.config/crowdin.yml ⇄ .config/crowdin.yml differ
.config/typos.toml ⇄ .config/typos.toml differ
.github/CODEOWNERS ⇄ .github/CODEOWNERS differ
.github/FUNDING.yml ⇄ .github/FUNDING.yml differ
.github/ISSUE_TEMPLATE/bug_report.yml ⇄ .github/ISSUE_TEMPLATE/bug_report.yml differ
.github/ISSUE_TEMPLATE/config.yml ⇄ .github/ISSUE_TEMPLATE/config.yml differ
.github/ISSUE_TEMPLATE/feature_request.yml ⇄ .github/ISSUE_TEMPLATE/feature_request.yml differ
.github/scripts/i18n/merge_crowdin_download.py ⇄ .github/scripts/i18n/merge_crowdin_download.py differ
.github/workflows/build.yml ⇄ .github/workflows/build.yml differ
.github/workflows/ci.yml ⇄ .github/workflows/ci.yml differ
.github/workflows/crowdin.yml ⇄ .github/workflows/crowdin.yml differ
.github/workflows/nix.yml ⇄ .github/workflows/nix.yml differ
.github/workflows/release-plz.yml ⇄ .github/workflows/release-plz.yml differ
.github/workflows/release.yml ⇄ .github/workflows/release.yml differ
.github/workflows/update-flake-lock.yml ⇄ .github/workflows/update-flake-lock.yml differ
.gitignore ⇄ .gitignore differ
AGENTS.md ⇄ AGENTS.md differ
CHANGELOG.md ⇄ CHANGELOG.md differ
Cargo.lock ⇄ Cargo.lock differ
Cargo.toml ⇄ Cargo.toml differ
README.md ⇄ README.md differ
crates/openlogi/Cargo.toml ⇄ crates/openlogi/Cargo.toml differ
crates/openlogi-agent/Cargo.toml ⇄ crates/openlogi-agent/Cargo.toml differ
crates/openlogi-agent/assets/tray-icon-prism@2x.png ⇄ crates/openlogi-agent/assets/tray-icon-prism@2x.png differ
crates/openlogi-agent/assets/tray-icon-white@2x.png ⇄ crates/openlogi-agent/assets/tray-icon-white@2x.png differ
crates/openlogi-agent/assets/tray-icon@2x.png ⇄ crates/openlogi-agent/assets/tray-icon@2x.png differ
crates/openlogi-agent/build.rs ⇄ crates/openlogi-agent/build.rs differ
crates/openlogi-agent/src/bin/mock_agent.rs ⇄ crates/openlogi-agent/src/bin/mock_agent.rs differ
crates/openlogi-agent/src/binary_watch/relaunch.rs ⇄ crates/openlogi-agent/src/binary_watch/relaunch.rs differ
crates/openlogi-agent/src/binary_watch.rs ⇄ crates/openlogi-agent/src/binary_watch.rs differ
crates/openlogi-agent/src/logging.rs ⇄ crates/openlogi-agent/src/logging.rs differ
crates/openlogi-agent/src/main.rs ⇄ crates/openlogi-agent/src/main.rs differ
crates/openlogi-agent/src/overlay.rs ⇄ crates/openlogi-agent/src/overlay.rs differ
crates/openlogi-agent/src/pairing.rs ⇄ crates/openlogi-agent/src/pairing.rs differ
crates/openlogi-agent/src/resume_windows.rs ⇄ crates/openlogi-agent/src/resume_windows.rs differ
crates/openlogi-agent/src/server.rs ⇄ crates/openlogi-agent/src/server.rs differ
crates/openlogi-agent/src/takeover.rs ⇄ crates/openlogi-agent/src/takeover.rs differ
crates/openlogi-agent/src/tray.rs ⇄ crates/openlogi-agent/src/tray.rs differ
crates/openlogi-agent/src/tray_windows.rs ⇄ crates/openlogi-agent/src/tray_windows.rs differ
crates/openlogi-agent-core/Cargo.toml ⇄ crates/openlogi-agent-core/Cargo.toml differ
crates/openlogi-agent-core/src/action_ring.rs ⇄ crates/openlogi-agent-core/src/action_ring.rs differ
crates/openlogi-agent-core/src/capture_plan.rs ⇄ crates/openlogi-agent-core/src/capture_plan.rs differ
crates/openlogi-agent-core/src/event_monitor.rs ⇄ crates/openlogi-agent-core/src/event_monitor.rs differ
crates/openlogi-agent-core/src/hardware/light.rs ⇄ crates/openlogi-agent-core/src/hardware/light.rs differ
crates/openlogi-agent-core/src/hardware.rs ⇄ crates/openlogi-agent-core/src/hardware.rs differ
crates/openlogi-agent-core/src/lib.rs ⇄ crates/openlogi-agent-core/src/lib.rs differ
crates/openlogi-agent-core/src/observable.rs ⇄ crates/openlogi-agent-core/src/observable.rs differ
crates/openlogi-agent-core/src/orchestrator/tests.rs ⇄ crates/openlogi-agent-core/src/orchestrator/tests.rs differ
crates/openlogi-agent-core/src/orchestrator.rs ⇄ crates/openlogi-agent-core/src/orchestrator.rs differ
crates/openlogi-agent-core/src/receiver_access.rs ⇄ crates/openlogi-agent-core/src/receiver_access.rs differ
crates/openlogi-agent-core/src/runtime/button/tests.rs ⇄ crates/openlogi-agent-core/src/runtime/button/tests.rs differ
crates/openlogi-agent-core/src/runtime/button.rs ⇄ crates/openlogi-agent-core/src/runtime/button.rs differ
crates/openlogi-agent-core/src/runtime/hook/tests.rs ⇄ crates/openlogi-agent-core/src/runtime/hook/tests.rs differ
crates/openlogi-agent-core/src/runtime/hook.rs ⇄ crates/openlogi-agent-core/src/runtime/hook.rs differ
crates/openlogi-agent-core/src/runtime/scroll/tests.rs ⇄ crates/openlogi-agent-core/src/runtime/scroll/tests.rs differ
crates/openlogi-agent-core/src/runtime/scroll/worker.rs ⇄ crates/openlogi-agent-core/src/runtime/scroll/worker.rs differ
crates/openlogi-agent-core/src/runtime/scroll.rs ⇄ crates/openlogi-agent-core/src/runtime/scroll.rs differ
crates/openlogi-agent-core/src/runtime.rs ⇄ crates/openlogi-agent-core/src/runtime.rs differ
crates/openlogi-agent-core/src/watchers/accessibility.rs ⇄ crates/openlogi-agent-core/src/watchers/accessibility.rs differ
crates/openlogi-agent-core/src/watchers/foreground_app.rs ⇄ crates/openlogi-agent-core/src/watchers/foreground_app.rs differ
crates/openlogi-agent-core/src/watchers/gesture/dispatch/tests.rs ⇄ crates/openlogi-agent-core/src/watchers/gesture/dispatch/tests.rs differ
crates/openlogi-agent-core/src/watchers/gesture/dispatch/wheel/tests.rs ⇄ crates/openlogi-agent-core/src/watchers/gesture/dispatch/wheel/tests.rs differ
crates/openlogi-agent-core/src/watchers/gesture/dispatch/wheel.rs ⇄ crates/openlogi-agent-core/src/watchers/gesture/dispatch/wheel.rs differ
crates/openlogi-agent-core/src/watchers/gesture/dispatch.rs ⇄ crates/openlogi-agent-core/src/watchers/gesture/dispatch.rs differ
crates/openlogi-agent-core/src/watchers/gesture/tests.rs ⇄ crates/openlogi-agent-core/src/watchers/gesture/tests.rs differ
crates/openlogi-agent-core/src/watchers/gesture.rs ⇄ crates/openlogi-agent-core/src/watchers/gesture.rs differ
crates/openlogi-agent-core/src/watchers/host_switch.rs ⇄ crates/openlogi-agent-core/src/watchers/host_switch.rs differ
crates/openlogi-agent-core/src/watchers/input_monitoring.rs ⇄ crates/openlogi-agent-core/src/watchers/input_monitoring.rs differ
crates/openlogi-agent-core/src/watchers/inventory.rs ⇄ crates/openlogi-agent-core/src/watchers/inventory.rs differ
crates/openlogi-agent-core/src/watchers/keyboard.rs ⇄ crates/openlogi-agent-core/src/watchers/keyboard.rs differ
crates/openlogi-agent-core/src/watchers/mod.rs ⇄ crates/openlogi-agent-core/src/watchers/mod.rs differ
crates/openlogi-agent-core/src/watchers/pairing.rs ⇄ crates/openlogi-agent-core/src/watchers/pairing.rs differ
crates/openlogi-agent-core/src/watchers/poll.rs ⇄ crates/openlogi-agent-core/src/watchers/poll.rs differ
crates/openlogi-assets/src/http.rs ⇄ crates/openlogi-assets/src/http.rs differ
crates/openlogi-assets/src/metadata.rs ⇄ crates/openlogi-assets/src/metadata.rs differ
crates/openlogi-assets/src/source.rs ⇄ crates/openlogi-assets/src/source.rs differ
crates/openlogi-camera/src/capture_types.rs ⇄ crates/openlogi-camera/src/capture_types.rs differ
crates/openlogi-camera/src/capture_windows.rs ⇄ crates/openlogi-camera/src/capture_windows.rs differ
crates/openlogi-camera/src/controls.rs ⇄ crates/openlogi-camera/src/controls.rs differ
crates/openlogi-camera/src/lib.rs ⇄ crates/openlogi-camera/src/lib.rs differ
crates/openlogi-camera/src/linux.rs ⇄ crates/openlogi-camera/src/linux.rs differ
crates/openlogi-camera/src/uvc_linux.rs ⇄ crates/openlogi-camera/src/uvc_linux.rs differ
crates/openlogi-camera/src/uvc_windows.rs ⇄ crates/openlogi-camera/src/uvc_windows.rs differ
crates/openlogi-cli/Cargo.toml ⇄ crates/openlogi-cli/Cargo.toml differ
crates/openlogi-cli/src/cmd/assets/sync.rs ⇄ crates/openlogi-cli/src/cmd/assets/sync.rs differ
crates/openlogi-cli/src/cmd/backlight.rs ⇄ crates/openlogi-cli/src/cmd/backlight.rs differ
crates/openlogi-cli/src/cmd/camera.rs ⇄ crates/openlogi-cli/src/cmd/camera.rs differ
crates/openlogi-cli/src/cmd/diag/battery.rs ⇄ crates/openlogi-cli/src/cmd/diag/battery.rs differ
crates/openlogi-cli/src/cmd/diag/dpi.rs ⇄ crates/openlogi-cli/src/cmd/diag/dpi.rs differ
crates/openlogi-cli/src/cmd/diag/features.rs ⇄ crates/openlogi-cli/src/cmd/diag/features.rs differ
crates/openlogi-cli/src/cmd/diag/lighting.rs ⇄ crates/openlogi-cli/src/cmd/diag/lighting.rs differ
crates/openlogi-cli/src/cmd/diag/smartshift.rs ⇄ crates/openlogi-cli/src/cmd/diag/smartshift.rs differ
crates/openlogi-cli/src/cmd/diag.rs ⇄ crates/openlogi-cli/src/cmd/diag.rs differ
crates/openlogi-cli/src/cmd/light.rs ⇄ crates/openlogi-cli/src/cmd/light.rs differ
crates/openlogi-cli/src/cmd/list.rs ⇄ crates/openlogi-cli/src/cmd/list.rs differ
crates/openlogi-cli/src/lib.rs ⇄ crates/openlogi-cli/src/lib.rs differ
crates/openlogi-core/Cargo.toml ⇄ crates/openlogi-core/Cargo.toml differ
crates/openlogi-core/src/app.rs ⇄ crates/openlogi-core/src/app.rs differ
crates/openlogi-core/src/binding/action.rs ⇄ crates/openlogi-core/src/binding/action.rs differ
crates/openlogi-core/src/binding/action_ring/icon.rs ⇄ crates/openlogi-core/src/binding/action_ring/icon.rs differ
crates/openlogi-core/src/binding/action_ring.rs ⇄ crates/openlogi-core/src/binding/action_ring.rs differ
crates/openlogi-core/src/binding/button.rs ⇄ crates/openlogi-core/src/binding/button.rs differ
crates/openlogi-core/src/binding/category.rs ⇄ crates/openlogi-core/src/binding/category.rs differ
crates/openlogi-core/src/binding/defaults.rs ⇄ crates/openlogi-core/src/binding/defaults.rs differ
crates/openlogi-core/src/binding/effect.rs ⇄ crates/openlogi-core/src/binding/effect.rs differ
crates/openlogi-core/src/binding/gesture.rs ⇄ crates/openlogi-core/src/binding/gesture.rs differ
crates/openlogi-core/src/binding/key_combo.rs ⇄ crates/openlogi-core/src/binding/key_combo.rs differ
crates/openlogi-core/src/binding/swipe.rs ⇄ crates/openlogi-core/src/binding/swipe.rs differ
crates/openlogi-core/src/binding/tests.rs ⇄ crates/openlogi-core/src/binding/tests.rs differ
crates/openlogi-core/src/binding/value.rs ⇄ crates/openlogi-core/src/binding/value.rs differ
crates/openlogi-core/src/binding.rs ⇄ crates/openlogi-core/src/binding.rs differ
crates/openlogi-core/src/bindings.rs ⇄ crates/openlogi-core/src/bindings.rs differ
crates/openlogi-core/src/brand.rs ⇄ crates/openlogi-core/src/brand.rs differ
crates/openlogi-core/src/config/device.rs ⇄ crates/openlogi-core/src/config/device.rs differ
crates/openlogi-core/src/config/file.rs ⇄ crates/openlogi-core/src/config/file.rs differ
crates/openlogi-core/src/config/identity.rs ⇄ crates/openlogi-core/src/config/identity.rs differ
crates/openlogi-core/src/config/key_trigger.rs ⇄ crates/openlogi-core/src/config/key_trigger.rs differ
crates/openlogi-core/src/config/settings.rs ⇄ crates/openlogi-core/src/config/settings.rs differ
crates/openlogi-core/src/config/tests.rs ⇄ crates/openlogi-core/src/config/tests.rs differ
crates/openlogi-core/src/config.rs ⇄ crates/openlogi-core/src/config.rs differ
crates/openlogi-core/src/device/light.rs ⇄ crates/openlogi-core/src/device/light.rs differ
crates/openlogi-core/src/device.rs ⇄ crates/openlogi-core/src/device.rs differ
crates/openlogi-core/src/device_order.rs ⇄ crates/openlogi-core/src/device_order.rs differ
crates/openlogi-core/src/diagnostics.rs ⇄ crates/openlogi-core/src/diagnostics.rs differ
crates/openlogi-core/src/hid/dpi.rs ⇄ crates/openlogi-core/src/hid/dpi.rs differ
crates/openlogi-core/src/hid/error.rs ⇄ crates/openlogi-core/src/hid/error.rs differ
crates/openlogi-core/src/hid/pairing.rs ⇄ crates/openlogi-core/src/hid/pairing.rs differ
crates/openlogi-core/src/hid/route.rs ⇄ crates/openlogi-core/src/hid/route.rs differ
crates/openlogi-core/src/hid/smartshift.rs ⇄ crates/openlogi-core/src/hid/smartshift.rs differ
crates/openlogi-core/src/hid.rs ⇄ crates/openlogi-core/src/hid.rs differ
crates/openlogi-core/src/lib.rs ⇄ crates/openlogi-core/src/lib.rs differ
crates/openlogi-core/src/paths.rs ⇄ crates/openlogi-core/src/paths.rs differ
crates/openlogi-core/src/scroll.rs ⇄ crates/openlogi-core/src/scroll.rs differ
crates/openlogi-core/src/single_instance.rs ⇄ crates/openlogi-core/src/single_instance.rs differ
crates/openlogi-desktop/Cargo.toml ⇄ crates/openlogi-desktop/Cargo.toml differ
crates/openlogi-desktop/build.rs ⇄ crates/openlogi-desktop/build.rs differ
crates/openlogi-desktop/bundle/desktop-dev/Info.plist ⇄ crates/openlogi-desktop/bundle/desktop-dev/Info.plist differ
crates/openlogi-desktop/icon/AppIcon.icns ⇄ crates/openlogi-desktop/icon/AppIcon.icns differ
crates/openlogi-desktop/src/app/deeplink.rs ⇄ crates/openlogi-desktop/src/app/deeplink.rs differ
crates/openlogi-desktop/src/app/detail.rs ⇄ crates/openlogi-desktop/src/app/detail.rs differ
crates/openlogi-desktop/src/app/home/views.rs ⇄ crates/openlogi-desktop/src/app/home/views.rs differ
crates/openlogi-desktop/src/app/home.rs ⇄ crates/openlogi-desktop/src/app/home.rs differ
crates/openlogi-desktop/src/app/menu.rs ⇄ crates/openlogi-desktop/src/app/menu.rs differ
crates/openlogi-desktop/src/app/status.rs ⇄ crates/openlogi-desktop/src/app/status.rs differ
crates/openlogi-desktop/src/app/widgets.rs ⇄ crates/openlogi-desktop/src/app/widgets.rs differ
crates/openlogi-desktop/src/app.rs ⇄ crates/openlogi-desktop/src/app.rs differ
crates/openlogi-desktop/src/app_assets.rs ⇄ crates/openlogi-desktop/src/app_assets.rs differ
crates/openlogi-desktop/src/features/action_ring/editor.rs ⇄ crates/openlogi-desktop/src/features/action_ring/editor.rs differ
crates/openlogi-desktop/src/features/action_ring.rs ⇄ crates/openlogi-desktop/src/features/action_ring.rs differ
crates/openlogi-desktop/src/features/camera/controls.rs ⇄ crates/openlogi-desktop/src/features/camera/controls.rs differ
crates/openlogi-desktop/src/features/camera/preview.rs ⇄ crates/openlogi-desktop/src/features/camera/preview.rs differ
crates/openlogi-desktop/src/features/keyboard/editors.rs ⇄ crates/openlogi-desktop/src/features/keyboard/editors.rs differ
crates/openlogi-desktop/src/features/keyboard/function_row.rs ⇄ crates/openlogi-desktop/src/features/keyboard/function_row.rs differ
crates/openlogi-desktop/src/features/keyboard/mod.rs ⇄ crates/openlogi-desktop/src/features/keyboard/mod.rs differ
crates/openlogi-desktop/src/features/lighting/mod.rs ⇄ crates/openlogi-desktop/src/features/lighting/mod.rs differ
crates/openlogi-desktop/src/features/lighting/standalone.rs ⇄ crates/openlogi-desktop/src/features/lighting/standalone.rs differ
crates/openlogi-desktop/src/features/lighting/visual.rs ⇄ crates/openlogi-desktop/src/features/lighting/visual.rs differ
crates/openlogi-desktop/src/features/mod.rs ⇄ crates/openlogi-desktop/src/features/mod.rs differ
crates/openlogi-desktop/src/features/mouse/geometry.rs ⇄ crates/openlogi-desktop/src/features/mouse/geometry.rs differ
crates/openlogi-desktop/src/features/mouse/hotspots.rs ⇄ crates/openlogi-desktop/src/features/mouse/hotspots.rs differ
crates/openlogi-desktop/src/features/mouse/inspector.rs ⇄ crates/openlogi-desktop/src/features/mouse/inspector.rs differ
crates/openlogi-desktop/src/features/mouse/leader_lines.rs ⇄ crates/openlogi-desktop/src/features/mouse/leader_lines.rs differ
crates/openlogi-desktop/src/features/mouse/mod.rs ⇄ crates/openlogi-desktop/src/features/mouse/mod.rs differ
crates/openlogi-desktop/src/features/mouse/thumbwheel.rs ⇄ crates/openlogi-desktop/src/features/mouse/thumbwheel.rs differ
crates/openlogi-desktop/src/features/mouse/view.rs ⇄ crates/openlogi-desktop/src/features/mouse/view.rs differ
crates/openlogi-desktop/src/features/pointer/dpi.rs ⇄ crates/openlogi-desktop/src/features/pointer/dpi.rs differ
crates/openlogi-desktop/src/features/pointer/mod.rs ⇄ crates/openlogi-desktop/src/features/pointer/mod.rs differ
crates/openlogi-desktop/src/features/pointer/smartshift.rs ⇄ crates/openlogi-desktop/src/features/pointer/smartshift.rs differ
crates/openlogi-desktop/src/main.rs ⇄ crates/openlogi-desktop/src/main.rs differ
crates/openlogi-desktop/src/platform/app_icon.rs ⇄ crates/openlogi-desktop/src/platform/app_icon.rs differ
crates/openlogi-desktop/src/platform/mod.rs ⇄ crates/openlogi-desktop/src/platform/mod.rs differ
crates/openlogi-desktop/src/platform/os.rs ⇄ crates/openlogi-desktop/src/platform/os.rs differ
crates/openlogi-desktop/src/platform/updater.rs ⇄ crates/openlogi-desktop/src/platform/updater.rs differ
crates/openlogi-desktop/src/runtime.rs ⇄ crates/openlogi-desktop/src/runtime.rs differ
crates/openlogi-desktop/src/services/assets/images.rs ⇄ crates/openlogi-desktop/src/services/assets/images.rs differ
crates/openlogi-desktop/src/services/assets/paths.rs ⇄ crates/openlogi-desktop/src/services/assets/paths.rs differ
crates/openlogi-desktop/src/services/assets/queries.rs ⇄ crates/openlogi-desktop/src/services/assets/queries.rs differ
crates/openlogi-desktop/src/services/assets/sync.rs ⇄ crates/openlogi-desktop/src/services/assets/sync.rs differ
crates/openlogi-desktop/src/services/assets.rs ⇄ crates/openlogi-desktop/src/services/assets.rs differ
crates/openlogi-desktop/src/services/device_reads.rs ⇄ crates/openlogi-desktop/src/services/device_reads.rs differ
crates/openlogi-desktop/src/services/diagnostics.rs ⇄ crates/openlogi-desktop/src/services/diagnostics.rs differ
crates/openlogi-desktop/src/services/i18n.rs ⇄ crates/openlogi-desktop/src/services/i18n.rs differ
crates/openlogi-desktop/src/services/ipc.rs ⇄ crates/openlogi-desktop/src/services/ipc.rs differ
crates/openlogi-desktop/src/state/action_ring.rs ⇄ crates/openlogi-desktop/src/state/action_ring.rs differ
crates/openlogi-desktop/src/state/agent.rs ⇄ crates/openlogi-desktop/src/state/agent.rs differ
crates/openlogi-desktop/src/state/bindings.rs ⇄ crates/openlogi-desktop/src/state/bindings.rs differ
crates/openlogi-desktop/src/state/camera.rs ⇄ crates/openlogi-desktop/src/state/camera.rs differ
crates/openlogi-desktop/src/state/config.rs ⇄ crates/openlogi-desktop/src/state/config.rs differ
crates/openlogi-desktop/src/state/device_key.rs ⇄ crates/openlogi-desktop/src/state/device_key.rs differ
crates/openlogi-desktop/src/state/device_store.rs ⇄ crates/openlogi-desktop/src/state/device_store.rs differ
crates/openlogi-desktop/src/state/devices.rs ⇄ crates/openlogi-desktop/src/state/devices.rs differ
crates/openlogi-desktop/src/state/dpi.rs ⇄ crates/openlogi-desktop/src/state/dpi.rs differ
crates/openlogi-desktop/src/state/inventory.rs ⇄ crates/openlogi-desktop/src/state/inventory.rs differ
crates/openlogi-desktop/src/state/light.rs ⇄ crates/openlogi-desktop/src/state/light.rs differ
crates/openlogi-desktop/src/state/lighting.rs ⇄ crates/openlogi-desktop/src/state/lighting.rs differ
crates/openlogi-desktop/src/state/load.rs ⇄ crates/openlogi-desktop/src/state/load.rs differ
crates/openlogi-desktop/src/state/scroll.rs ⇄ crates/openlogi-desktop/src/state/scroll.rs differ
crates/openlogi-desktop/src/state/settings.rs ⇄ crates/openlogi-desktop/src/state/settings.rs differ
crates/openlogi-desktop/src/state/smartshift.rs ⇄ crates/openlogi-desktop/src/state/smartshift.rs differ
crates/openlogi-desktop/src/state/tests.rs ⇄ crates/openlogi-desktop/src/state/tests.rs differ
crates/openlogi-desktop/src/state.rs ⇄ crates/openlogi-desktop/src/state.rs differ
crates/openlogi-desktop/src/ui/action.rs ⇄ crates/openlogi-desktop/src/ui/action.rs differ
crates/openlogi-desktop/src/ui/battery.rs ⇄ crates/openlogi-desktop/src/ui/battery.rs differ
crates/openlogi-desktop/src/ui/choice_card.rs ⇄ crates/openlogi-desktop/src/ui/choice_card.rs differ
crates/openlogi-desktop/src/ui/components.rs ⇄ crates/openlogi-desktop/src/ui/components.rs differ
crates/openlogi-desktop/src/ui/gallery.rs ⇄ crates/openlogi-desktop/src/ui/gallery.rs differ
crates/openlogi-desktop/src/ui/mod.rs ⇄ crates/openlogi-desktop/src/ui/mod.rs differ
crates/openlogi-desktop/src/ui/theme.rs ⇄ crates/openlogi-desktop/src/ui/theme.rs differ
crates/openlogi-desktop/src/windows/add_device.rs ⇄ crates/openlogi-desktop/src/windows/add_device.rs differ
crates/openlogi-desktop/src/windows/main_window.rs ⇄ crates/openlogi-desktop/src/windows/main_window.rs differ
crates/openlogi-desktop/src/windows/settings/about.rs ⇄ crates/openlogi-desktop/src/windows/settings/about.rs differ
crates/openlogi-desktop/src/windows/settings/appearance.rs ⇄ crates/openlogi-desktop/src/windows/settings/appearance.rs differ
crates/openlogi-desktop/src/windows/settings/assets.rs ⇄ crates/openlogi-desktop/src/windows/settings/assets.rs differ
crates/openlogi-desktop/src/windows/settings/diagnostics.rs ⇄ crates/openlogi-desktop/src/windows/settings/diagnostics.rs differ
crates/openlogi-desktop/src/windows/settings/general.rs ⇄ crates/openlogi-desktop/src/windows/settings/general.rs differ
crates/openlogi-desktop/src/windows/settings/language.rs ⇄ crates/openlogi-desktop/src/windows/settings/language.rs differ
crates/openlogi-desktop/src/windows/settings/permissions.rs ⇄ crates/openlogi-desktop/src/windows/settings/permissions.rs differ
crates/openlogi-desktop/src/windows/settings/updates.rs ⇄ crates/openlogi-desktop/src/windows/settings/updates.rs differ
crates/openlogi-desktop/src/windows/settings.rs ⇄ crates/openlogi-desktop/src/windows/settings.rs differ
crates/openlogi-desktop/src/windows/update_consent.rs ⇄ crates/openlogi-desktop/src/windows/update_consent.rs differ
crates/openlogi-desktop/src/windows.rs ⇄ crates/openlogi-desktop/src/windows.rs differ
crates/openlogi-desktop/themes/openlogi.json ⇄ crates/openlogi-desktop/themes/openlogi.json differ
crates/openlogi-device/Cargo.toml ⇄ crates/openlogi-device/Cargo.toml differ
crates/openlogi-device/src/backend.rs ⇄ crates/openlogi-device/src/backend.rs differ
crates/openlogi-device/src/backlight.rs ⇄ crates/openlogi-device/src/backlight.rs differ
crates/openlogi-device/src/channel/registry.rs ⇄ crates/openlogi-device/src/channel/registry.rs differ
crates/openlogi-device/src/channel/route.rs ⇄ crates/openlogi-device/src/channel/route.rs differ
crates/openlogi-device/src/channel/scripted.rs ⇄ crates/openlogi-device/src/channel/scripted.rs differ
crates/openlogi-device/src/channel.rs ⇄ crates/openlogi-device/src/channel.rs differ
crates/openlogi-device/src/inventory/cache.rs ⇄ crates/openlogi-device/src/inventory/cache.rs differ
crates/openlogi-device/src/inventory/features.rs ⇄ crates/openlogi-device/src/inventory/features.rs differ
crates/openlogi-device/src/inventory/ledger.rs ⇄ crates/openlogi-device/src/inventory/ledger.rs differ
crates/openlogi-device/src/inventory/mappings.rs ⇄ crates/openlogi-device/src/inventory/mappings.rs differ
crates/openlogi-device/src/inventory/persist.rs ⇄ crates/openlogi-device/src/inventory/persist.rs differ
crates/openlogi-device/src/inventory/probe.rs ⇄ crates/openlogi-device/src/inventory/probe.rs differ
crates/openlogi-device/src/inventory/standalone.rs ⇄ crates/openlogi-device/src/inventory/standalone.rs differ
crates/openlogi-device/src/inventory/tests.rs ⇄ crates/openlogi-device/src/inventory/tests.rs differ
crates/openlogi-device/src/inventory.rs ⇄ crates/openlogi-device/src/inventory.rs differ
crates/openlogi-device/src/lib.rs ⇄ crates/openlogi-device/src/lib.rs differ
crates/openlogi-device/src/pairing/registers.rs ⇄ crates/openlogi-device/src/pairing/registers.rs differ
crates/openlogi-device/src/pairing/tests.rs ⇄ crates/openlogi-device/src/pairing/tests.rs differ
crates/openlogi-device/src/pairing.rs ⇄ crates/openlogi-device/src/pairing.rs differ
crates/openlogi-device/src/reprog_controls/event.rs ⇄ crates/openlogi-device/src/reprog_controls/event.rs differ
crates/openlogi-device/src/reprog_controls.rs ⇄ crates/openlogi-device/src/reprog_controls.rs differ
crates/openlogi-device/src/session/gesture/tests.rs ⇄ crates/openlogi-device/src/session/gesture/tests.rs differ
crates/openlogi-device/src/session/gesture.rs ⇄ crates/openlogi-device/src/session/gesture.rs differ
crates/openlogi-device/src/session/host_switch.rs ⇄ crates/openlogi-device/src/session/host_switch.rs differ
crates/openlogi-device/src/session/keyboard.rs ⇄ crates/openlogi-device/src/session/keyboard.rs differ
crates/openlogi-device/src/session.rs ⇄ crates/openlogi-device/src/session.rs differ
crates/openlogi-device/src/thumbwheel.rs ⇄ crates/openlogi-device/src/thumbwheel.rs differ
crates/openlogi-device/src/write/backlight.rs ⇄ crates/openlogi-device/src/write/backlight.rs differ
crates/openlogi-device/src/write/diagnostics.rs ⇄ crates/openlogi-device/src/write/diagnostics.rs differ
crates/openlogi-device/src/write/dpi.rs ⇄ crates/openlogi-device/src/write/dpi.rs differ
crates/openlogi-device/src/write/error.rs ⇄ crates/openlogi-device/src/write/error.rs differ
crates/openlogi-device/src/write/fn_lock.rs ⇄ crates/openlogi-device/src/write/fn_lock.rs differ
crates/openlogi-device/src/write/haptic.rs ⇄ crates/openlogi-device/src/write/haptic.rs differ
crates/openlogi-device/src/write/hires_wheel.rs ⇄ crates/openlogi-device/src/write/hires_wheel.rs differ
crates/openlogi-device/src/write/lighting.rs ⇄ crates/openlogi-device/src/write/lighting.rs differ
crates/openlogi-device/src/write/litra.rs ⇄ crates/openlogi-device/src/write/litra.rs differ
crates/openlogi-device/src/write/smartshift.rs ⇄ crates/openlogi-device/src/write/smartshift.rs differ
crates/openlogi-device/src/write/tests.rs ⇄ crates/openlogi-device/src/write/tests.rs differ
crates/openlogi-device/src/write.rs ⇄ crates/openlogi-device/src/write.rs differ
crates/openlogi-device-registry/src/lib.rs ⇄ crates/openlogi-device-registry/src/lib.rs differ
crates/openlogi-device-registry/src/receiver.rs ⇄ crates/openlogi-device-registry/src/receiver.rs differ
crates/openlogi-hid/Cargo.toml ⇄ crates/openlogi-hid/Cargo.toml differ
crates/openlogi-hid/examples/thumbwheel_trace.rs ⇄ crates/openlogi-hid/examples/thumbwheel_trace.rs differ
crates/openlogi-hid/src/host.rs ⇄ crates/openlogi-hid/src/host.rs differ
crates/openlogi-hid/src/lib.rs ⇄ crates/openlogi-hid/src/lib.rs differ
crates/openlogi-hid/src/permissions.rs ⇄ crates/openlogi-hid/src/permissions.rs differ
crates/openlogi-hid/src/transport/native.rs ⇄ crates/openlogi-hid/src/transport/native.rs differ
crates/openlogi-hid/src/transport/windows.rs ⇄ crates/openlogi-hid/src/transport/windows.rs differ
crates/openlogi-hid/src/transport.rs ⇄ crates/openlogi-hid/src/transport.rs differ
crates/openlogi-hidpp/AGENTS.md ⇄ crates/openlogi-hidpp/AGENTS.md differ
crates/openlogi-hidpp/Cargo.toml ⇄ crates/openlogi-hidpp/Cargo.toml differ
crates/openlogi-hidpp/src/channel/message.rs ⇄ crates/openlogi-hidpp/src/channel/message.rs differ
crates/openlogi-hidpp/src/channel/raw.rs ⇄ crates/openlogi-hidpp/src/channel/raw.rs differ
crates/openlogi-hidpp/src/channel/tests.rs ⇄ crates/openlogi-hidpp/src/channel/tests.rs differ
crates/openlogi-hidpp/src/channel.rs ⇄ crates/openlogi-hidpp/src/channel.rs differ
crates/openlogi-hidpp/src/device.rs ⇄ crates/openlogi-hidpp/src/device.rs differ
crates/openlogi-hidpp/src/feature/adjustable_dpi.rs ⇄ crates/openlogi-hidpp/src/feature/adjustable_dpi.rs differ
crates/openlogi-hidpp/src/feature/backlight.rs ⇄ crates/openlogi-hidpp/src/feature/backlight.rs differ
crates/openlogi-hidpp/src/feature/battery_status.rs ⇄ crates/openlogi-hidpp/src/feature/battery_status.rs differ
crates/openlogi-hidpp/src/feature/battery_voltage.rs ⇄ crates/openlogi-hidpp/src/feature/battery_voltage.rs differ
crates/openlogi-hidpp/src/feature/color_led_effects/tests.rs ⇄ crates/openlogi-hidpp/src/feature/color_led_effects/tests.rs differ
crates/openlogi-hidpp/src/feature/color_led_effects.rs ⇄ crates/openlogi-hidpp/src/feature/color_led_effects.rs differ
crates/openlogi-hidpp/src/feature/crown/tests.rs ⇄ crates/openlogi-hidpp/src/feature/crown/tests.rs differ
crates/openlogi-hidpp/src/feature/crown.rs ⇄ crates/openlogi-hidpp/src/feature/crown.rs differ
crates/openlogi-hidpp/src/feature/device_friendly_name.rs ⇄ crates/openlogi-hidpp/src/feature/device_friendly_name.rs differ
crates/openlogi-hidpp/src/feature/device_information.rs ⇄ crates/openlogi-hidpp/src/feature/device_information.rs differ
crates/openlogi-hidpp/src/feature/dual_platform.rs ⇄ crates/openlogi-hidpp/src/feature/dual_platform.rs differ
crates/openlogi-hidpp/src/feature/extended_dpi/tests.rs ⇄ crates/openlogi-hidpp/src/feature/extended_dpi/tests.rs differ
crates/openlogi-hidpp/src/feature/extended_dpi/types.rs ⇄ crates/openlogi-hidpp/src/feature/extended_dpi/types.rs differ
crates/openlogi-hidpp/src/feature/extended_dpi.rs ⇄ crates/openlogi-hidpp/src/feature/extended_dpi.rs differ
crates/openlogi-hidpp/src/feature/extended_report_rate.rs ⇄ crates/openlogi-hidpp/src/feature/extended_report_rate.rs differ
crates/openlogi-hidpp/src/feature/feature_set.rs ⇄ crates/openlogi-hidpp/src/feature/feature_set.rs differ
crates/openlogi-hidpp/src/feature/fn_inversion.rs ⇄ crates/openlogi-hidpp/src/feature/fn_inversion.rs differ
crates/openlogi-hidpp/src/feature/haptic_feedback.rs ⇄ crates/openlogi-hidpp/src/feature/haptic_feedback.rs differ
crates/openlogi-hidpp/src/feature/hires_wheel.rs ⇄ crates/openlogi-hidpp/src/feature/hires_wheel.rs differ
crates/openlogi-hidpp/src/feature/hosts_info.rs ⇄ crates/openlogi-hidpp/src/feature/hosts_info.rs differ
crates/openlogi-hidpp/src/feature/illumination/tests.rs ⇄ crates/openlogi-hidpp/src/feature/illumination/tests.rs differ
crates/openlogi-hidpp/src/feature/illumination/types.rs ⇄ crates/openlogi-hidpp/src/feature/illumination/types.rs differ
crates/openlogi-hidpp/src/feature/illumination.rs ⇄ crates/openlogi-hidpp/src/feature/illumination.rs differ
crates/openlogi-hidpp/src/feature/multi_platform.rs ⇄ crates/openlogi-hidpp/src/feature/multi_platform.rs differ
crates/openlogi-hidpp/src/feature/per_key_lighting/tests.rs ⇄ crates/openlogi-hidpp/src/feature/per_key_lighting/tests.rs differ
crates/openlogi-hidpp/src/feature/per_key_lighting.rs ⇄ crates/openlogi-hidpp/src/feature/per_key_lighting.rs differ
crates/openlogi-hidpp/src/feature/persistent_remappable_action/tests.rs ⇄ crates/openlogi-hidpp/src/feature/persistent_remappable_action/tests.rs differ
crates/openlogi-hidpp/src/feature/persistent_remappable_action.rs ⇄ crates/openlogi-hidpp/src/feature/persistent_remappable_action.rs differ
crates/openlogi-hidpp/src/feature/registry.rs ⇄ crates/openlogi-hidpp/src/feature/registry.rs differ
crates/openlogi-hidpp/src/feature/report_rate.rs ⇄ crates/openlogi-hidpp/src/feature/report_rate.rs differ
crates/openlogi-hidpp/src/feature/reprog_controls/tests.rs ⇄ crates/openlogi-hidpp/src/feature/reprog_controls/tests.rs differ
crates/openlogi-hidpp/src/feature/reprog_controls.rs ⇄ crates/openlogi-hidpp/src/feature/reprog_controls.rs differ
crates/openlogi-hidpp/src/feature/rgb_effects/tests.rs ⇄ crates/openlogi-hidpp/src/feature/rgb_effects/tests.rs differ
crates/openlogi-hidpp/src/feature/rgb_effects/types.rs ⇄ crates/openlogi-hidpp/src/feature/rgb_effects/types.rs differ
crates/openlogi-hidpp/src/feature/rgb_effects.rs ⇄ crates/openlogi-hidpp/src/feature/rgb_effects.rs differ
crates/openlogi-hidpp/src/feature/smartshift.rs ⇄ crates/openlogi-hidpp/src/feature/smartshift.rs differ
crates/openlogi-hidpp/src/feature/thumbwheel.rs ⇄ crates/openlogi-hidpp/src/feature/thumbwheel.rs differ
crates/openlogi-hidpp/src/feature/unified_battery.rs ⇄ crates/openlogi-hidpp/src/feature/unified_battery.rs differ
crates/openlogi-hidpp/src/feature/wireless_device_status.rs ⇄ crates/openlogi-hidpp/src/feature/wireless_device_status.rs differ
crates/openlogi-hidpp/src/feature.rs ⇄ crates/openlogi-hidpp/src/feature.rs differ
crates/openlogi-hidpp/src/lib.rs ⇄ crates/openlogi-hidpp/src/lib.rs differ
crates/openlogi-hidpp/src/protocol/v10.rs ⇄ crates/openlogi-hidpp/src/protocol/v10.rs differ
crates/openlogi-hidpp/src/protocol/v20.rs ⇄ crates/openlogi-hidpp/src/protocol/v20.rs differ
crates/openlogi-hidpp/src/receiver/bolt/event.rs ⇄ crates/openlogi-hidpp/src/receiver/bolt/event.rs differ
crates/openlogi-hidpp/src/receiver/bolt.rs ⇄ crates/openlogi-hidpp/src/receiver/bolt.rs differ
crates/openlogi-hidpp/src/receiver/unifying.rs ⇄ crates/openlogi-hidpp/src/receiver/unifying.rs differ
crates/openlogi-hidpp/src/receiver.rs ⇄ crates/openlogi-hidpp/src/receiver.rs differ
crates/openlogi-hook/Cargo.toml ⇄ crates/openlogi-hook/Cargo.toml differ
crates/openlogi-hook/gnome-shell-extension/README.md ⇄ crates/openlogi-hook/gnome-shell-extension/README.md differ
crates/openlogi-hook/gnome-shell-extension/openlogi-frontmost@openlogi.dev/extension.js ⇄ crates/openlogi-hook/gnome-shell-extension/openlogi-frontmost@openlogi.dev/extension.js differ
crates/openlogi-hook/gnome-shell-extension/openlogi-frontmost@openlogi.dev/metadata.json ⇄ crates/openlogi-hook/gnome-shell-extension/openlogi-frontmost@openlogi.dev/metadata.json differ
crates/openlogi-hook/src/lib.rs ⇄ crates/openlogi-hook/src/lib.rs differ
crates/openlogi-hook/src/linux.rs ⇄ crates/openlogi-hook/src/linux.rs differ
crates/openlogi-hook/src/macos/watchdog.rs ⇄ crates/openlogi-hook/src/macos/watchdog.rs differ
crates/openlogi-hook/src/macos.rs ⇄ crates/openlogi-hook/src/macos.rs differ
crates/openlogi-hook/src/tests.rs ⇄ crates/openlogi-hook/src/tests.rs differ
crates/openlogi-inject/Cargo.toml ⇄ crates/openlogi-inject/Cargo.toml differ
crates/openlogi-inject/src/inject/linux.rs ⇄ crates/openlogi-inject/src/inject/linux.rs differ
crates/openlogi-inject/src/inject/macos.rs ⇄ crates/openlogi-inject/src/inject/macos.rs differ
crates/openlogi-inject/src/inject/windows.rs ⇄ crates/openlogi-inject/src/inject/windows.rs differ
crates/openlogi-inject/src/inject.rs ⇄ crates/openlogi-inject/src/inject.rs differ
crates/openlogi-inject/src/lib.rs ⇄ crates/openlogi-inject/src/lib.rs differ
crates/openlogi-ipc/Cargo.toml ⇄ crates/openlogi-ipc/Cargo.toml differ
crates/openlogi-ipc/src/client.rs ⇄ crates/openlogi-ipc/src/client.rs differ
crates/openlogi-ipc/src/ipc.rs ⇄ crates/openlogi-ipc/src/ipc.rs differ
crates/openlogi-ipc/src/lib.rs ⇄ crates/openlogi-ipc/src/lib.rs differ
crates/openlogi-ipc/tests/wire_format.rs ⇄ crates/openlogi-ipc/tests/wire_format.rs differ
crates/openlogi-overlay/Cargo.toml ⇄ crates/openlogi-overlay/Cargo.toml differ
crates/openlogi-overlay/build.rs ⇄ crates/openlogi-overlay/build.rs differ
crates/openlogi-overlay/src/main.rs ⇄ crates/openlogi-overlay/src/main.rs differ
crates/openlogi-overlay/src/platform.rs ⇄ crates/openlogi-overlay/src/platform.rs differ
crates/openlogi-overlay/src/ring.rs ⇄ crates/openlogi-overlay/src/ring.rs differ
crates/openlogi-overlay/src/session.rs ⇄ crates/openlogi-overlay/src/session.rs differ
crates/openlogi-permissions/Cargo.toml ⇄ crates/openlogi-permissions/Cargo.toml differ
crates/openlogi-permissions/src/lib.rs ⇄ crates/openlogi-permissions/src/lib.rs differ
crates/openlogi-permissions/src/linux.rs ⇄ crates/openlogi-permissions/src/linux.rs differ
crates/openlogi-permissions/src/macos.rs ⇄ crates/openlogi-permissions/src/macos.rs differ
crates/openlogi-permissions/src/tests.rs ⇄ crates/openlogi-permissions/src/tests.rs differ
crates/openlogi-ui/Cargo.toml ⇄ crates/openlogi-ui/Cargo.toml differ
crates/openlogi-ui/src/action_icons.rs ⇄ crates/openlogi-ui/src/action_icons.rs differ
crates/openlogi-ui/src/lib.rs ⇄ crates/openlogi-ui/src/lib.rs differ
design/icon/openlogi-128.png ⇄ design/icon/openlogi-128.png differ
design/icon/openlogi-16.png ⇄ design/icon/openlogi-16.png differ
design/icon/openlogi-256.png ⇄ design/icon/openlogi-256.png differ
design/icon/openlogi-32.png ⇄ design/icon/openlogi-32.png differ
design/icon/openlogi-48.png ⇄ design/icon/openlogi-48.png differ
design/icon/openlogi-512.png ⇄ design/icon/openlogi-512.png differ
design/icon/openlogi-64.png ⇄ design/icon/openlogi-64.png differ
design/icon/openlogi-prism.ico ⇄ design/icon/openlogi-prism.ico differ
design/icon/openlogi-prism.icon/Assets/OpenLogi-1.png ⇄ design/icon/openlogi-prism.icon/Assets/OpenLogi-1.png differ
design/icon/openlogi.ico ⇄ design/icon/openlogi.ico differ
design/icon/openlogi.icon/Assets/OpenLogi.png ⇄ design/icon/openlogi.icon/Assets/OpenLogi.png differ
design/icon/openlogi.png ⇄ design/icon/openlogi.png differ
devenv.nix ⇄ devenv.nix differ
docs/CONFIGURATION.md ⇄ docs/CONFIGURATION.md differ
docs/DECISIONS.md ⇄ docs/DECISIONS.md differ
docs/DEVELOPMENT.md ⇄ docs/DEVELOPMENT.md differ
docs/INSTALL-linux.md ⇄ docs/INSTALL-linux.md differ
docs/README.de.md ⇄ docs/README.de.md differ
docs/README.fr.md ⇄ docs/README.fr.md differ
docs/README.ja.md ⇄ docs/README.ja.md differ
docs/README.ko.md ⇄ docs/README.ko.md differ
docs/README.zh-CN.md ⇄ docs/README.zh-CN.md differ
docs/USAGE.md ⇄ docs/USAGE.md differ
docs/config.example.toml ⇄ docs/config.example.toml differ
flake.lock ⇄ flake.lock differ
packaging/linux/desktop/openlogi.desktop ⇄ packaging/linux/desktop/openlogi.desktop differ
packaging/linux/install.sh ⇄ packaging/linux/install.sh differ
packaging/linux/nfpm.yaml ⇄ packaging/linux/nfpm.yaml differ
packaging/linux/package.nix ⇄ packaging/linux/package.nix differ
packaging/linux/systemd/openlogi-agent.service ⇄ packaging/linux/systemd/openlogi-agent.service differ
packaging/windows/OpenLogi.wxs ⇄ packaging/windows/OpenLogi.wxs differ
prek.toml ⇄ prek.toml differ
release-plz.toml ⇄ release-plz.toml differ
xtask/Cargo.toml ⇄ xtask/Cargo.toml differ
xtask/README.md ⇄ xtask/README.md differ
xtask/src/commands/ci/jobs/steps.rs ⇄ xtask/src/commands/ci/jobs/steps.rs differ
xtask/src/commands/ci/jobs/tests.rs ⇄ xtask/src/commands/ci/jobs/tests.rs differ
xtask/src/commands/ci/jobs.rs ⇄ xtask/src/commands/ci/jobs.rs differ
xtask/src/commands/ci/list.rs ⇄ xtask/src/commands/ci/list.rs differ
xtask/src/commands/linux/package.rs ⇄ xtask/src/commands/linux/package.rs differ
xtask/src/commands/macos/bundle/embed/tests.rs ⇄ xtask/src/commands/macos/bundle/embed/tests.rs differ
xtask/src/commands/macos/bundle/embed.rs ⇄ xtask/src/commands/macos/bundle/embed.rs differ
xtask/src/commands/macos/bundle/identity/tests.rs ⇄ xtask/src/commands/macos/bundle/identity/tests.rs differ
xtask/src/commands/macos/bundle/identity.rs ⇄ xtask/src/commands/macos/bundle/identity.rs differ
xtask/src/commands/macos/bundle/signing.rs ⇄ xtask/src/commands/macos/bundle/signing.rs differ
xtask/src/commands/macos/bundle.rs ⇄ xtask/src/commands/macos/bundle.rs differ
xtask/src/commands/macos/dev_bundle/processes.rs ⇄ xtask/src/commands/macos/dev_bundle/processes.rs differ
xtask/src/commands/macos/dev_bundle/signing.rs ⇄ xtask/src/commands/macos/dev_bundle/signing.rs differ
xtask/src/commands/macos/dev_bundle/tests.rs ⇄ xtask/src/commands/macos/dev_bundle/tests.rs differ
xtask/src/commands/macos/dev_bundle.rs ⇄ xtask/src/commands/macos/dev_bundle.rs differ
xtask/src/commands/macos/dmg.rs ⇄ xtask/src/commands/macos/dmg.rs differ
xtask/src/commands/macos.rs ⇄ xtask/src/commands/macos.rs differ
xtask/src/commands/mod.rs ⇄ xtask/src/commands/mod.rs differ
xtask/src/commands/release/checkout_version_bump/tests.rs ⇄ xtask/src/commands/release/checkout_version_bump/tests.rs differ
xtask/src/commands/release/latest_json.rs ⇄ xtask/src/commands/release/latest_json.rs differ
xtask/src/commands/release.rs ⇄ xtask/src/commands/release.rs differ
xtask/src/icon/macos.rs ⇄ xtask/src/icon/macos.rs differ
xtask/src/main.rs ⇄ xtask/src/main.rs differ
xtask/src/support/info_plist.rs ⇄ xtask/src/support/info_plist.rs differ
xtask/src/support/mod.rs ⇄ xtask/src/support/mod.rs differ
```

## Only in main (missing from snapshot)

```
.claude/rules/hidpp.md
.claude/rules/hook.md
Only in ./.opencode
Only in ./.qwen
Only in ./.wix
crates/openlogi-agent/src/launch_agent.rs
crates/openlogi-agent-core/benches
crates/openlogi-agent-core/src/runtime/scroll/acceleration.rs
crates/openlogi-agent-core/src/runtime/scroll/display_link.rs
crates/openlogi-camera/src/capture.rs
crates/openlogi-camera/src/uvc
crates/openlogi-camera/src/uvc.rs
crates/openlogi-cli/src/cmd/assets/mod.rs
crates/openlogi-cli/src/cmd/mod.rs
crates/openlogi-core/src/binding/action_target.rs
crates/openlogi-core/src/math.rs
crates/openlogi-desktop/assets
crates/openlogi-desktop/icon/Assets.car
crates/openlogi-desktop/icon/previews
crates/openlogi-desktop/icon/prism.icns
crates/openlogi-desktop/src/features/lighting/device.rs
crates/openlogi-desktop/src/features/mouse/picker.rs
crates/openlogi-desktop/src/features/profile_scope.rs
crates/openlogi-desktop/src/state/device_runtime.rs
crates/openlogi-desktop/src/ui/action_composer.rs
crates/openlogi-desktop/src/ui/shortcut_builder.rs
crates/openlogi-desktop/src/windows/settings/general
crates/openlogi-device/src/inventory/generic.rs
crates/openlogi-hook/src/linux/gnome_shell.rs
crates/openlogi-hook/src/linux/wlr_foreign_toplevel.rs
crates/openlogi-hook/src/windows.rs
crates/openlogi-overlay/src/agent.rs
crates/openlogi-ui/locales/da.yml
crates/openlogi-ui/locales/de.yml
crates/openlogi-ui/locales/el.yml
crates/openlogi-ui/locales/en.yml
crates/openlogi-ui/locales/es.yml
crates/openlogi-ui/locales/fi.yml
crates/openlogi-ui/locales/fr.yml
crates/openlogi-ui/locales/it.yml
crates/openlogi-ui/locales/ja.yml
crates/openlogi-ui/locales/ko.yml
crates/openlogi-ui/locales/nb.yml
crates/openlogi-ui/locales/nl.yml
crates/openlogi-ui/locales/pl.yml
crates/openlogi-ui/locales/pt-BR.yml
crates/openlogi-ui/locales/pt-PT.yml
crates/openlogi-ui/locales/ru.yml
crates/openlogi-ui/locales/sv.yml
crates/openlogi-ui/locales/tr.yml
crates/openlogi-ui/locales/uk.yml
crates/openlogi-ui/locales/zh-CN.yml
crates/openlogi-ui/locales/zh-HK.yml
crates/openlogi-ui/locales/zh-TW.yml
crates/openlogi-ui/src/locale.rs
design/icon/openlogi.svg
design/icon/prism.svg
docs/scroll01.md
```

## Only in snapshot (missing from main)

```
.agents/rules
.agents/skills
Only in OpenLogi-0.8.15/.ast-grep
.claude/skills/ast-grep
.claude/skills/contributing-device-fixtures
.claude/skills/diagnosing-openlogi-devices
.claude/skills/gpui-kit
.claude/skills/gpui-kit-design-guides
.claude/skills/testing-openlogi-ui
.claude/skills/verifying-openlogi-changes
.github/workflows/stale-allows.yml
crates/openlogi-agent/src/activity_macos.rs
crates/openlogi-agent/src/autostart
crates/openlogi-agent/src/autostart.rs
crates/openlogi-agent/src/bin/mock_agent
crates/openlogi-agent/src/lifecycle
crates/openlogi-agent/src/lifecycle.rs
crates/openlogi-agent/src/pairing
crates/openlogi-agent/src/resume_linux.rs
crates/openlogi-agent/src/shutdown.rs
crates/openlogi-agent/src/startup.rs
crates/openlogi-agent/src/tests.rs
crates/openlogi-agent-core/.ast-grep
crates/openlogi-agent-core/src/backlight_overlay.rs
crates/openlogi-agent-core/src/hardware/context
crates/openlogi-agent-core/src/hardware/context.rs
crates/openlogi-agent-core/src/hardware/rgb_control.rs
crates/openlogi-agent-core/src/hardware/write_order.rs
crates/openlogi-agent-core/src/lock_keys
crates/openlogi-agent-core/src/lock_keys.rs
crates/openlogi-agent-core/src/notices
crates/openlogi-agent-core/src/notices.rs
crates/openlogi-agent-core/src/orchestrator/devices.rs
crates/openlogi-agent-core/src/orchestrator/lights.rs
crates/openlogi-agent-core/src/orchestrator/tests
crates/openlogi-agent-core/src/runtime/button/worker.rs
crates/openlogi-agent-core/src/runtime/pointer.rs
crates/openlogi-agent-core/src/watchers/capture_manager.rs
crates/openlogi-agent-core/src/watchers/capture_session.rs
crates/openlogi-agent-core/src/watchers/inventory
crates/openlogi-agent-core/src/watchers/keyboard
crates/openlogi-agent-core/src/watchers/light
crates/openlogi-agent-core/src/watchers/light.rs
crates/openlogi-agent-core/src/watchers/pairing
crates/openlogi-agent-core/src/watchers/pointer.rs
crates/openlogi-agent-core/src/watchers/retry.rs
crates/openlogi-agent-core/src/watchers/shutdown.rs
crates/openlogi-camera/src/capture_macos.rs
crates/openlogi-camera/src/uvc_macos
crates/openlogi-camera/src/uvc_macos.rs
crates/openlogi-cli/.ast-grep
crates/openlogi-cli/src/agent.rs
crates/openlogi-cli/src/cmd/api
crates/openlogi-cli/src/cmd/api.rs
crates/openlogi-cli/src/cmd/assets.rs
crates/openlogi-cli/src/cmd/diag/pointer_speed.rs
crates/openlogi-cli/src/cmd/fixture
crates/openlogi-cli/src/cmd/fixture.rs
crates/openlogi-cli/src/cmd.rs
crates/openlogi-core/.ast-grep
crates/openlogi-core/src/app
crates/openlogi-core/src/binding/control.rs
crates/openlogi-core/src/config/button_map.rs
crates/openlogi-core/src/config/function_key.rs
crates/openlogi-core/src/config/gestures.rs
crates/openlogi-core/src/config/migrate.rs
crates/openlogi-core/src/config/per_app.rs
crates/openlogi-core/src/config/per_device.rs
crates/openlogi-core/src/config/revision.rs
crates/openlogi-core/src/config/tests
crates/openlogi-core/src/config/wheel_hold.rs
crates/openlogi-core/src/env.rs
crates/openlogi-core/src/hid/backlight.rs
crates/openlogi-core/src/hid/easy_switch.rs
crates/openlogi-core/src/hid/fn_lock.rs
crates/openlogi-core/src/hid/report_rate.rs
crates/openlogi-core/src/hid/scroll.rs
crates/openlogi-core/src/locale.rs
crates/openlogi-core/src/logging.rs
crates/openlogi-core/src/worker.rs
crates/openlogi-desktop/.ast-grep
crates/openlogi-desktop/AGENTS.md
crates/openlogi-desktop/CLAUDE.md
crates/openlogi-desktop/src/app/detail
crates/openlogi-desktop/src/app/tests
crates/openlogi-desktop/src/app/tests.rs
crates/openlogi-desktop/src/features/binding_editor.rs
crates/openlogi-desktop/src/features/camera/controls
crates/openlogi-desktop/src/features/camera/preview
crates/openlogi-desktop/src/features/keyboard/function_row
crates/openlogi-desktop/src/features/lighting/keyboard_rgb.rs
crates/openlogi-desktop/src/features/mouse/view
crates/openlogi-desktop/src/features/pointer/report_rate.rs
crates/openlogi-desktop/src/features/profiles
crates/openlogi-desktop/src/features/profiles.rs
crates/openlogi-desktop/src/platform/installation
crates/openlogi-desktop/src/platform/installation.rs
crates/openlogi-desktop/src/platform/registration
crates/openlogi-desktop/src/platform/registration.rs
crates/openlogi-desktop/src/services/assets/tests.rs
crates/openlogi-desktop/src/services/i18n
crates/openlogi-desktop/src/services/ipc
crates/openlogi-desktop/src/state/backlight.rs
crates/openlogi-desktop/src/state/device_session.rs
crates/openlogi-desktop/src/state/devices
crates/openlogi-desktop/src/state/easy_switch.rs
crates/openlogi-desktop/src/state/events.rs
crates/openlogi-desktop/src/state/fn_lock.rs
crates/openlogi-desktop/src/state/os_layout.rs
crates/openlogi-desktop/src/state/report_rate.rs
crates/openlogi-desktop/src/state/tests
crates/openlogi-desktop/src/ui/commit_slider.rs
crates/openlogi-desktop/src/ui/components
crates/openlogi-device/.ast-grep
crates/openlogi-device/AGENTS.md
crates/openlogi-device/CLAUDE.md
crates/openlogi-device/src/device_io.rs
crates/openlogi-device/src/host_lock.rs
crates/openlogi-device/src/inventory/channel_cache.rs
crates/openlogi-device/src/inventory/events.rs
crates/openlogi-device/src/inventory/probe
crates/openlogi-device/src/inventory/replay_test_support.rs
crates/openlogi-device/src/inventory/replay_tests.rs
crates/openlogi-device/src/inventory/tests
crates/openlogi-device/src/replay
crates/openlogi-device/src/replay.rs
crates/openlogi-device/src/session/capture
crates/openlogi-device/src/session/capture.rs
crates/openlogi-device/src/session/capture_restore.rs
crates/openlogi-device/src/session/gesture/accum.rs
crates/openlogi-device/src/session/gesture/arm.rs
crates/openlogi-device/src/session/gesture/main_wheel.rs
crates/openlogi-device/src/session/gesture/tests
crates/openlogi-device/src/session/host_switch
crates/openlogi-device/src/session/restore.rs
crates/openlogi-device/src/write/dpi
crates/openlogi-device/src/write/easy_switch
crates/openlogi-device/src/write/easy_switch.rs
crates/openlogi-device/src/write/lighting
crates/openlogi-device/src/write/litra
crates/openlogi-device/src/write/os_layout
crates/openlogi-device/src/write/os_layout.rs
crates/openlogi-device/src/write/pointer_scaling.rs
crates/openlogi-device/src/write/report_rate
crates/openlogi-device/src/write/report_rate.rs
crates/openlogi-device/src/write/smartshift
crates/openlogi-device-registry/src/camera.rs
crates/openlogi-device-registry/src/headset.rs
crates/openlogi-fixture
crates/openlogi-hid/AGENTS.md
crates/openlogi-hid/CLAUDE.md
crates/openlogi-hid/src/lighting
crates/openlogi-hid/src/lighting.rs
crates/openlogi-hid/src/recording
crates/openlogi-hid/src/recording.rs
crates/openlogi-hidpp/.ast-grep
crates/openlogi-hidpp/src/channel/mock.rs
crates/openlogi-hidpp/src/channel/observation
crates/openlogi-hidpp/src/channel/observation.rs
crates/openlogi-hidpp/src/channel/pending.rs
crates/openlogi-hidpp/src/channel/tests
crates/openlogi-hidpp/src/feature/adc_measurement.rs
crates/openlogi-hidpp/src/feature/device_information
crates/openlogi-hidpp/src/feature/lock_key_state
crates/openlogi-hidpp/src/feature/lock_key_state.rs
crates/openlogi-hidpp/src/feature/multi_platform
crates/openlogi-hidpp/src/feature/pointer_motion_scaling.rs
crates/openlogi-hook/.ast-grep
crates/openlogi-hook/AGENTS.md
crates/openlogi-hook/CLAUDE.md
crates/openlogi-hook/examples/invert_scroll.rs
crates/openlogi-hook/examples/pointer_context.rs
crates/openlogi-hook/src/linux/foreground
crates/openlogi-hook/src/linux/foreground.rs
crates/openlogi-hook/src/linux/pointer.rs
crates/openlogi-hook/src/linux/supervisor.rs
crates/openlogi-hook/src/macos/foreground.rs
crates/openlogi-hook/src/macos/grant.rs
crates/openlogi-hook/src/macos/pointer.rs
crates/openlogi-hook/src/macos/sender.rs
crates/openlogi-hook/src/macos/translate.rs
crates/openlogi-hook/src/pointer.rs
crates/openlogi-hook/src/windows
crates/openlogi-inject/.ast-grep
crates/openlogi-inject/examples/hold_chord.rs
crates/openlogi-inject/src/inject/macos
crates/openlogi-inject/src/inject/space_switch
crates/openlogi-inject/src/inject/space_switch.rs
crates/openlogi-inject/src/uinput.rs
crates/openlogi-ipc/.ast-grep
crates/openlogi-ipc/AGENTS.md
crates/openlogi-ipc/CLAUDE.md
crates/openlogi-ipc/src/testing.rs
crates/openlogi-overlay/src/backlight.rs
crates/openlogi-overlay/src/ipc
crates/openlogi-overlay/src/ipc.rs
crates/openlogi-overlay/src/lock_key.rs
crates/openlogi-overlay/src/notice
crates/openlogi-overlay/src/notice.rs
crates/openlogi-overlay/src/platform
crates/openlogi-overlay/src/tests.rs
crates/openlogi-ui/action-icons/battery-low.svg
crates/openlogi-ui/action-icons/circle-dot.svg
crates/openlogi-ui/action-icons/dot.svg
crates/openlogi-ui/action-icons/mouse-left.svg
crates/openlogi-ui/action-icons/mouse-right.svg
crates/openlogi-ui/action-icons/x.svg
crates/openlogi-ui/locales/be.toml
crates/openlogi-ui/locales/cs.toml
crates/openlogi-ui/locales/da.toml
crates/openlogi-ui/locales/de.toml
crates/openlogi-ui/locales/el.toml
crates/openlogi-ui/locales/en.toml
crates/openlogi-ui/locales/es.toml
crates/openlogi-ui/locales/fi.toml
crates/openlogi-ui/locales/fr.toml
crates/openlogi-ui/locales/it.toml
crates/openlogi-ui/locales/ja.toml
crates/openlogi-ui/locales/ko.toml
crates/openlogi-ui/locales/nb.toml
crates/openlogi-ui/locales/nl.toml
crates/openlogi-ui/locales/pl.toml
crates/openlogi-ui/locales/pt-BR.toml
crates/openlogi-ui/locales/pt-PT.toml
crates/openlogi-ui/locales/ru.toml
crates/openlogi-ui/locales/sv.toml
crates/openlogi-ui/locales/tr.toml
crates/openlogi-ui/locales/uk.toml
crates/openlogi-ui/locales/zh-CN.toml
crates/openlogi-ui/locales/zh-HK.toml
crates/openlogi-ui/locales/zh-TW.toml
crates/openlogi-ui/tests
design/LICENSE
docs/MOCK_DEVICE_TESTING.md
docs/README.es.md
docs/README.pt-BR.md
docs/README.ru.md
docs/README.tr.md
Only in OpenLogi-0.8.15/fixtures
packaging/linux/flatpak
packaging/linux/tests
packaging/windows/openlogi-installation.json
Only in OpenLogi-0.8.15/sgconfig.yml
Only in OpenLogi-0.8.15/skills-lock.json
xtask/.ast-grep
xtask/AGENTS.md
xtask/CLAUDE.md
xtask/src/commands/lint
xtask/src/commands/lint.rs
xtask/src/commands/macos/bundle/sparkle
xtask/src/commands/macos/bundle/sparkle.rs
xtask/src/commands/release/appcast
xtask/src/commands/release/appcast.rs
xtask/src/support/info_plist
xtask/src/support/update_feed
xtask/src/support/update_feed.rs
```
