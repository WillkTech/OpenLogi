# OpenLogi: Platform Action Dispatch, Key Simulation, & Layered Profile Architecture

## Executive Summary

This document provides a comprehensive architectural breakdown and engineering proposal for **OpenLogi**, a local-first alternative to Logitech Options+ written in Rust. It addresses three main core pillars:
1. **Platform Action Implementation & Dispatch Analysis**: Detailed audit of how actions (`Copy`, `Paste`, `Undo`, `MissionControl`, `LockScreen`, etc.) and key simulations are currently handled across **macOS**, **Windows**, and **Linux**.
2. **Key Combination Customization & UI Proposal**: Proposal for a unified key combo / macro data model and a visual hotkey recorder component in GPUI (`crates/openlogi-desktop`).
3. **Multi-Tier Application Profiles & XMBC-Style Multi-Layer State Machine**: Architectural blueprint for per-application profiles with multi-tier fallback matching, plus an X-Mouse Button Control (XMBC) parity multi-layer state machine preserving strict modifier release invariants.

---

## 1. Phase 1: Platform Action Implementation & Dispatch Analysis

### 1.1 Action Data Models & Catalog Architecture
In OpenLogi, actions are defined as pure, IO-free configuration data within `crates/openlogi-core`:
* **`Action` Enum (`crates/openlogi-core/src/binding/action.rs`)**: Represents all rebindable actions, categorized into System, Mouse, Editing, Browser, Navigation, Media, DPI, Scroll, and Custom payloads (`CustomShortcut(KeyCombo)`, `HoldShortcut(KeyCombo)`, `Workflow(Vec<WorkflowStep>)`, `OpenApplication(ApplicationTarget)`).
* **`Effect` Classification (`crates/openlogi-core/src/binding/effect.rs`)**: Translates high-level `Action` variants into concrete execution targets: `Effect::Click(MouseButton)`, `Effect::Shortcut(Shortcut)`, `Effect::Key(KeyCombo)`, `Effect::HeldKey(KeyCombo)`, `Effect::Scroll`, `Effect::Media`, `Effect::Native`, `Effect::Script`, `Effect::Text`, and `Effect::AgentSide`.
* **Platform Injection Entry Point (`crates/openlogi-inject/src/inject.rs`)**: The `execute(action: &Action)` function acts as the single entry point. It delegates to target-specific modules via `cfg_if!`.

---

### 1.2 Platform-Specific Dispatch Audit

```
                            [ Action Enum / Effect ]
                                       │
                                 execute(&Action)
                                       │
         ┌─────────────────────────────┼─────────────────────────────┐
         ▼                             ▼                             ▼
   [ macOS Inject ]             [ Windows Inject ]            [ Linux Inject ]
 (crates/openlogi-inject/       (crates/openlogi-inject/      (crates/openlogi-inject/
   src/inject/macos.rs)          src/inject/windows.rs)        src/inject/linux.rs)
         │                             │                             │
 ┌───────┴───────┐             ┌───────┴───────┐             ┌───────┴───────┐
 │ CGEventPost   │             │ SendInput     │             │ uinput evdev  │
 │ CoreGraphics  │             │ (VK_* & Scan) │             │ (/dev/uinput) │
 │ AXUIElement   │             └───────────────┘             │ D-Bus / logind│
 │ Private SPIs  │                                           │ MPRIS         │
 └───────────────┘                                           └───────────────┘
```

#### 1. macOS (`crates/openlogi-inject/src/inject/macos.rs`)
* **Keystroke & Mouse Event Synthesis**: Low-level mouse clicks and keyboard shortcuts are posted using `CGEvent::new_keyboard_event` and `CGEvent::new_mouse_event` targeting `CGEventTapLocation::HID`.
* **Synthetic Event Tagging**: Every synthetic event is stamped with `SYNTHETIC_EVENT_USER_DATA = 0x4F4C_4749` (`"OLGI"`) in `EVENT_SOURCE_USER_DATA`. This prevents OpenLogi's own `CGEventTap` input hook from re-capturing injected events and triggering infinite re-entrance loops.
* **Accessibility / AX Navigation**: Browser Back/Forward for WebKit/Safari bypasses key injection and invokes `AXPress` on the toolbar's "Go back" / "Go forward" `AXButton` via Accessibility APIs (`AXUIElementCopyAttributeValue` & `AXUIElementPerformAction`), using process ID (`pid`) captured at press time.
* **Window Management & System SPIs**:
  - `MissionControl`, `AppExpose`, `ShowDesktop`, and `LaunchpadShow` post notifications to the Dock using private SPIs via `dlopen`/`dlsym` (`CoreDockSendNotification` from `ApplicationServices.framework`).
  - Desktop space switching (`PreviousDesktop`, `NextDesktop`) uses `CGSGetSymbolicHotKeyValue` and `CGSSetSymbolicHotKeyEnabled` SPIs to retrieve user hotkey bindings and post events to `CGEventTapLocation::Session`.
  - System sleep invokes `/usr/bin/pmset sleepnow`.
* **Media Keys**: Volume up/down/mute and media play/pause/track controls are posted via `NSEventType::SystemDefined` auxiliary control buttons (subtype 8, `NX_KEYTYPE_*`) inside an `autoreleasepool`.

#### 2. Windows (`crates/openlogi-inject/src/inject/windows.rs`)
* **Event Synthesis (`SendInput`)**: Mouse and keyboard events map to Win32 `INPUT` structures submitted via `SendInput`.
* **Key Mapping**: USB HID usage codes are mapped to Win32 Virtual-Key (`VK_*`) codes via `hid_usage_to_windows`.
* **Modifiers & Native Actions**:
  - `MissionControl` / `AppExpose` post `Win+Tab`.
  - `ShowDesktop` posts `Win+D`.
  - `PreviousDesktop` / `NextDesktop` post `Win+Ctrl+Left` / `Win+Ctrl+Right`.
  - `Screenshot` / `CaptureRegion` post `Win+Shift+S`.
  - `BrowserBack` / `BrowserForward` send dedicated virtual keys `VK_BROWSER_BACK` (0xA6) / `VK_BROWSER_FORWARD` (0xA7).

#### 3. Linux (`crates/openlogi-inject/src/inject/linux.rs`)
* **Event Synthesis (`uinput` / `evdev`)**: Uses a lazily initialized `uinput` virtual device (`OpenLogi action injector`). Key events emit `EV_KEY` with evdev `KEY_*` codes mapped from USB HID usages via `hid_usage_to_linux`.
* **Scroll Synthesis**: Emits relative axis events `REL_WHEEL`, `REL_HWHEEL`, and high-resolution wheel events `REL_WHEEL_HI_RES`, `REL_HWHEEL_HI_RES`.
* **System & Media Interoperability**:
  - `LockScreen` calls `org.freedesktop.login1.Manager.LockSession($XDG_SESSION_ID)` over D-Bus system bus, falling back to `Super+L`.
  - `Sleep` calls `org.freedesktop.login1.Manager.Suspend(false)` over D-Bus system bus.
  - Media play/pause/next/prev query session D-Bus for `org.mpris.MediaPlayer2.*` services and invoke `PlayPause`/`Next`/`Previous` methods before falling back to `KEY_PLAYPAUSE`, `KEY_NEXTSONG`, etc.

---

### 1.3 Key Press Simulation Verification & Modifier Timing

OpenLogi enforces strict modifier flag tracking and held-state synchronization across all platforms in `crates/openlogi-inject/src/inject.rs`:

```rust
/// Reference counts for physical keyboard outputs across active chords.
struct HeldOutput {
    owners: HashMap<HeldKey, usize>,
}
```

#### Held Output Mechanics & Refcounting
When executing `HoldShortcut(KeyCombo)` or held modifier keys:
1. **Reference Counting (`HeldOutput`)**: Maintains an internal map of physical keys (`Command`, `Control`, `Shift`, `Alt`, `Key(KeyboardUsage)`) and their reference counts (`usize`).
2. **Platform Mapping Differences**:
   - On **macOS**, `Command` and `Control` are distinct physical keys.
   - On **Windows** and **Linux**, `Command` (Super/Meta) and `Control` map to the same physical `Control` output for held chords, deduplicating modifier tracking.
3. **Edge Sequencing (`HoldTransition`)**:
   - On **Press Down**: Modifiers are asserted *before* the key.
   - On **Release Up**: The key is released *before* the modifiers are unasserted (reverse order).
   - **Shared Key Safety**: If Chord A (`Ctrl+A`) and Chord B (`Ctrl+B`) are both held, releasing Chord A drops physical key `A` while incrementing/maintaining the reference count for `Control`. `Control` is only unasserted when the reference count drops to 0.

---

## 2. Phase 2: Key Combination Customization & UI Proposal

### 2.1 Unified Data Model Proposal

Currently, OpenLogi separates simple key combos (`KeyCombo`) from multi-step sequences (`WorkflowStep`). To support custom chords, hold-and-release mechanics, and multi-step macros seamlessly, we propose a **Unified Shortcut Model** in `openlogi-core`.

#### Proposed TOML Schema Additions

```toml
# Example 1: Standard Key Combination with Hold Mechanics
[devices."receiver:mouse:slot:1".bindings.Back]
type = "Shortcut"
chord = "Ctrl+Shift+V"
mode = "Hold" # Options: "Tap" (default), "Hold"

# Example 2: Extended Sequence / Macro with Delays and Hold Timing
[devices."receiver:mouse:slot:1".bindings.Forward]
type = "Macro"
name = "Paste & Format"
steps = [
  { action = "PressKey", chord = "Ctrl+V" },
  { action = "Delay", millis = 100 },
  { action = "PressKey", chord = "Ctrl+Shift+F" },
  { action = "TypeText", text = "\n// Processed by OpenLogi" }
]

# Example 3: Complex Multi-Layer Shift Binding (XMBC Parity)
[devices."receiver:mouse:slot:1".profiles."com.apple.dt.Xcode".layers.Layer2.bindings.MiddleButton]
type = "Shortcut"
chord = "Cmd+Option+Shift+F"
mode = "Tap"
```

#### Proposed Rust Struct Enhancements (`crates/openlogi-core/src/binding/macro_seq.rs`)

```rust
use serde::{Deserialize, Serialize};
use super::key_combo::KeyCombo;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShortcutMode {
    /// Emit key down immediately followed by key up.
    Tap,
    /// Hold key down for the physical press duration; emit key up on release.
    Hold,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnifiedShortcut {
    pub chord: KeyCombo,
    pub mode: ShortcutMode,
    pub repeat_interval_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MacroStep {
    PressKey { chord: KeyCombo, hold_ms: Option<u64> },
    KeyDown { chord: KeyCombo },
    KeyUp { chord: KeyCombo },
    TypeText { text: String },
    Delay { millis: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MacroSequence {
    pub name: String,
    pub steps: Vec<MacroStep>,
}
```

---

### 2.2 Desktop UI Integration (GPUI Component in `openlogi-desktop`)

We propose a visual key combination recorder component for `crates/openlogi-desktop/src/ui/key_recorder.rs`:

```
┌────────────────────────────────────────────────────────────────────────┐
│ Record Hotkey                                                        │
├────────────────────────────────────────────────────────────────────────┤
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │  ⌘  +  ⇧  +  V                                                    │ │
│ └────────────────────────────────────────────────────────────────────┘ │
│ Mode:  (o) Tap (Instant)   ( ) Hold (Press duration)                  │
│ Options: [x] Capture modifier chords   [ ] Allow sequential multi-keys │
├────────────────────────────────────────────────────────────────────────┤
│ [ Cancel ]                                              [ Save Binding ] │
└────────────────────────────────────────────────────────────────────────┘
```

#### Key Capture Component Design

1. **Physical Key Event Capture**:
   - Captures raw key down/up events using GPUI's `on_key_down` and `on_key_up` event handlers on the element focus state.
   - Converts physical keys into `KeyboardUsage` HID codes for cross-platform portability.

2. **Sequential vs Concurrent Chord Detection**:
   - **Concurrent Chord**: Detects active modifier keys (`Cmd`, `Ctrl`, `Shift`, `Alt`/`Option`, `Super`) currently pressed down simultaneously with a non-modifier key.
   - **Sequential Sequence**: Tracks key press timestamps. If keys are pressed in sequence without overlap within a configurable threshold (e.g. 300ms), records a multi-step macro (`MacroSequence`).

3. **Cross-Platform Shortcut Formatting**:
   Formats key labels dynamically based on host target (`cfg(target_os)`):
   - **macOS**: `⌘` (Command), `⌥` (Option), `⇧` (Shift), `⌃` (Control), e.g., `⌘⇧V`.
   - **Windows/Linux**: `Ctrl`, `Shift`, `Alt`, `Win` / `Super`, e.g., `Ctrl + Shift + V`.

---

## 3. Phase 3: Application Profiles & XMBC-Style Layer Architecture

### 3.1 Active Window & Process Detection Mechanics

Foreground application detection currently resides in `crates/openlogi-hook` and publishes updates to `openlogi-agent-core` via `watchers::foreground_app`:

```
                     ┌───────────────────────────────┐
                     │   ForegroundApp Watcher Task  │
                     └───────────────┬───────────────┘
                                     │
         ┌───────────────────────────┼───────────────────────────┐
         ▼                           ▼                           ▼
  [ macOS Reader ]           [ Windows Reader ]          [ Linux Readers ]
  `NSWorkspace`              `GetForegroundWindow`       Priority Chain:
  `frontmostApplication`     `GetWindowThreadProcessId`   1. `gnome_shell` D-Bus
  bundle_id (e.g.,           QueryFullProcessImageName   2. `wlr_foreign_toplevel`
  "com.apple.Safari")        exe_name (e.g., "code.exe") 3. X11 `_NET_ACTIVE_WINDOW`
```

#### Multi-Tier Profile Fallback Hierarchy
We propose enhancing profile resolution to support a **3-Tier Fallback Hierarchy**:

$$\text{Global Profile (Default)} \longrightarrow \text{Per-App Profile (Bundle ID / Exe)} \longrightarrow \text{Window Title / Class Overlay}$$

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AppTargetSpec {
    /// Exact bundle ID (macOS) or executable name (Windows/Linux).
    pub app_id: String,
    /// Optional regex or wildcard pattern for window title (e.g., ".* — Neovim").
    pub window_title_pattern: Option<String>,
    /// Optional Linux WM_CLASS or Windows window class name.
    pub window_class_pattern: Option<String>,
}
```

#### Profile Resolution Logic
When evaluating a button press for device $D$ in active window $W$:
1. Check if $W$ matches a **Window Class / Title Overlay** defined under $D$'s profiles. If matched, inspect binding.
2. If unassigned, fall back to $D$'s **Per-Application Profile** matching $W$.app_id.
3. If unassigned, fall back to $D$'s **Global Default Profile**.

---

### 3.2 XMBC-Style Multi-Layer Architecture & State Machine

X-Mouse Button Control (XMBC) parity requires supporting **Multi-Layer Mappings** (e.g., Layer 1 through Layer 10) per profile, triggered via modifier buttons or layer toggle actions.

```
                              [ Physical Input Event ]
                                         │
                                         ▼
                        ┌──────────────────────────────────┐
                        │ Layer State Machine Evaluator    │
                        │ (Active Layer: N, Modifiers: M)  │
                        └────────────────┬─────────────────┘
                                         │
                 ┌───────────────────────┴───────────────────────┐
                 ▼                                               ▼
     [ Button is Layer Modifier ]                    [ Button is Standard Action ]
  - Transition Layer State                        - Query Layer N binding
  - Retain Held State Matrix                        (fallback -> Layer 1 -> Global)
  - Execute Modifier Press                          - Register ActivePress(Layer N)
```

#### State Machine Invariant: Modifier Release State Protection
A critical invariant in XMBC behavior occurs when a user holds a button that switches to **Layer $N$**, presses and holds an action button on **Layer $N$**, and then **releases the layer modifier button first**:

> **Invariant Requirement**:
> 1. The action initiated on **Layer $N$ must remain active** until its physical button is released.
> 2. Reverting to Layer 1 must **NOT** trigger a phantom key-up for Layer $N$'s action, nor emit a duplicate key-down for Layer 1's binding on that same physical button.
> 3. Active key ownership refcounts must be preserved until all involved physical buttons are released.

---

### 3.3 Prototype Implementation of Multi-Layer State Machine

Below is a complete, production-ready Rust prototype for the multi-layer state machine to be integrated into `crates/openlogi-agent-core/src/runtime/layer_state.rs`.

```rust
//! Multi-Layer / Shift-Layer State Machine for OpenLogi (XMBC Parity).

use std::collections::{HashMap, HashSet};
use openlogi_core::binding::{Action, ButtonId};

/// Layer Identifier (Layer 1 is default base layer).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayerId(pub u8);

impl LayerId {
    pub const BASE: Self = LayerId(1);
}

/// How a button modifies the active layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayerModifierMode {
    /// Active only while the physical button is held down.
    HoldLayer(LayerId),
    /// Toggles active layer on press until pressed again.
    ToggleLayer(LayerId),
    /// Momentarily switches to layer for `timeout_ms` or until next press.
    TimedModifier { target: LayerId, timeout_ms: u64 },
}

/// Binding map for a single layer within a profile.
#[derive(Clone, Debug, Default)]
pub struct LayerBindingMap {
    pub bindings: HashMap<ButtonId, Action>,
    pub modifiers: HashMap<ButtonId, LayerModifierMode>,
}

/// Active press tracking for layer release invariant.
#[derive(Clone, Debug)]
pub struct ActiveLayerPress {
    pub button: ButtonId,
    pub dispatched_layer: LayerId,
    pub action: Action,
}

/// Multi-Layer State Machine per device.
#[derive(Debug)]
pub struct ProfileLayerStateMachine {
    active_profile_id: String,
    /// Profile -> Layer -> Mappings
    profile_layers: HashMap<String, HashMap<LayerId, LayerBindingMap>>,
    /// Currently active layer driven by toggles and held modifier buttons.
    current_layer: LayerId,
    /// Stack of active held layer modifiers: Vec<(ModifierButton, TargetLayer)>
    held_layer_modifiers: Vec<(ButtonId, LayerId)>,
    /// Tracking physical buttons currently held down to enforce release invariant.
    /// Key: ButtonId -> Dispatched Layer & Action
    active_presses: HashMap<ButtonId, ActiveLayerPress>,
}

impl ProfileLayerStateMachine {
    pub fn new() -> Self {
        Self {
            active_profile_id: "global".to_string(),
            profile_layers: HashMap::new(),
            current_layer: LayerId::BASE,
            held_layer_modifiers: Vec::new(),
            active_presses: HashMap::new(),
        }
    }

    pub fn set_active_profile(&mut self, profile_id: &str) {
        if self.active_profile_id != profile_id {
            self.active_profile_id = profile_id.to_string();
            self.reevaluate_active_layer();
        }
    }

    /// Process a physical button DOWN event.
    pub fn handle_button_down(&mut self, button: ButtonId) -> Option<Action> {
        // Check if button is configured as a Layer Modifier in the current profile/layer
        if let Some(modifier_mode) = self.resolve_layer_modifier(button) {
            match modifier_mode {
                LayerModifierMode::HoldLayer(target_layer) => {
                    self.held_layer_modifiers.push((button, target_layer));
                    self.current_layer = target_layer;
                }
                LayerModifierMode::ToggleLayer(target_layer) => {
                    if self.current_layer == target_layer {
                        self.current_layer = LayerId::BASE;
                    } else {
                        self.current_layer = target_layer;
                    }
                }
                LayerModifierMode::TimedModifier { target, .. } => {
                    self.current_layer = target;
                }
            }
            return None; // Layer modifier buttons consume the press event
        }

        // Resolve action for standard button on the current active layer
        let (dispatched_layer, action) = self.resolve_action_for_button(button);

        // Record active press to protect against premature release or layer switches
        self.active_presses.insert(button, ActiveLayerPress {
            button,
            dispatched_layer,
            action: action.clone(),
        });

        Some(action)
    }

    /// Process a physical button UP event.
    pub fn handle_button_up(&mut self, button: ButtonId) -> Option<Action> {
        // 1. Check if this button was a held layer modifier
        if let Some(pos) = self.held_layer_modifiers.iter().position(|(b, _)| *b == button) {
            self.held_layer_modifiers.remove(pos);
            self.reevaluate_active_layer();
            // Note: Even though the active layer reverts, any buttons in `self.active_presses`
            // remain associated with their original `dispatched_layer`.
            return None;
        }

        // 2. Otherwise, terminate the active press for this button
        if let Some(active_press) = self.active_presses.remove(&button) {
            return Some(active_press.action);
        }

        None
    }

    fn reevaluate_active_layer(&mut self) {
        if let Some((_, top_layer)) = self.held_layer_modifiers.last() {
            self.current_layer = *top_layer;
        } else {
            self.current_layer = LayerId::BASE;
        }
    }

    fn resolve_layer_modifier(&self, button: ButtonId) -> Option<LayerModifierMode> {
        let profile = self.profile_layers.get(&self.active_profile_id)?;
        let layer_map = profile.get(&self.current_layer)?;
        layer_map.modifiers.get(&button).cloned()
    }

    fn resolve_action_for_button(&self, button: ButtonId) -> (LayerId, Action) {
        // 1. Check current profile & active layer
        if let Some(profile) = self.profile_layers.get(&self.active_profile_id) {
            if let Some(layer_map) = profile.get(&self.current_layer) {
                if let Some(action) = layer_map.bindings.get(&button) {
                    return (self.current_layer, action.clone());
                }
            }
            // Fallback to Layer 1 of current profile
            if self.current_layer != LayerId::BASE {
                if let Some(base_map) = profile.get(&LayerId::BASE) {
                    if let Some(action) = base_map.bindings.get(&button) {
                        return (LayerId::BASE, action.clone());
                    }
                }
            }
        }

        // 2. Global profile fallback
        if self.active_profile_id != "global" {
            if let Some(global_profile) = self.profile_layers.get("global") {
                if let Some(layer_map) = global_profile.get(&LayerId::BASE) {
                    if let Some(action) = layer_map.bindings.get(&button) {
                        return (LayerId::BASE, action.clone());
                    }
                }
            }
        }

        (LayerId::BASE, Action::None)
    }
}
```

---

## 4. Engineering Implementation & Integration Plan

### Step-by-Step Task Schedule

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Phase 1: Core Configuration & Schema (crates/openlogi-core)                 │
│ - Extend DeviceConfig to support multi-layer profiles & unified shortcut    │
│ - Bump PROTOCOL_VERSION in openlogi-ipc and update wire format tests       │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ Phase 2: Agent Layer State Machine (crates/openlogi-agent-core)            │
│ - Integrate ProfileLayerStateMachine into button runtime worker             │
│ - Implement held-press release invariants and refcounted modifier safety    │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ Phase 3: Desktop UI Layer & Hotkey Capture (crates/openlogi-desktop)        │
│ - Build KeyRecorder GPUI component with live key capture & mode toggles     │
│ - Implement Layer selection tabs (Layer 1..10) in button remapping UI       │
└─────────────────────────────────────────────────────────────────────────────┘
```

1. **Phase 1: `crates/openlogi-core` Data Models**:
   - Add `LayerId`, `LayerBindingMap`, `ProfileConfig`, and `UnifiedShortcut` types.
   - Update `config.toml` serialization/deserialization logic.
   - Bump `PROTOCOL_VERSION` in `crates/openlogi-ipc/src/ipc.rs` and update golden assertions in `crates/openlogi-ipc/tests/wire_format.rs`.

2. **Phase 2: `crates/openlogi-agent-core` Runtime Execution**:
   - Embed `ProfileLayerStateMachine` into `runtime::button::ButtonRuntimeOwner`.
   - Wire foreground application notifications to update `set_active_profile()`.
   - Update action synthesis in `openlogi-inject` to process `UnifiedShortcut` hold/tap modes.

3. **Phase 3: `crates/openlogi-desktop` & GPUI UX**:
   - Create `KeyRecorder` modal component.
   - Add Layer tabs (`Layer 1`, `Layer 2`, `Layer 3`...) to the device control panel.
   - Display active layer indicator in the status bar / device view.

---

## Summary of Key Architecture Decisions

| Architectural Area | Current OpenLogi Design | Proposed XMBC-Parity Architecture |
|---|---|---|
| **Action Dispatch** | Single `execute(&Action)` via CGEvent / SendInput / uinput | Preserved & extended to support unified `HoldShortcut` / `UnifiedShortcut` |
| **Foreground App Detection** | Flat `per_app_bindings` (Bundle ID match) | 3-Tier Fallback: Global -> App -> Window Title/Class Overlay |
| **Button Layers** | Single base binding map per device | Multi-layer per profile (Layer 1..10) with fallback inheritance |
| **Layer Switch Release** | N/A | State Machine tracking `ActiveLayerPress`: held actions complete safely when modifier releases |
| **Shortcut Recording** | String parsing (`KeyCombo::from_str`) | Visual GPUI `KeyRecorder` component capturing physical HID usages |
