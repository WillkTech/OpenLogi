# OpenLogi Scroll Pipeline & UI Audit Report

## Executive Summary

This report delivers an in-depth architectural analysis and UI/UX streamlining proposal for **Smooth Scrolling**, **Vertical Scroll Sensitivity**, **Horizontal Scroll Acceleration**, and **Thumbwheel Sensitivity** across the OpenLogi codebase (`openlogi-agent-core`, `openlogi-core`, `openlogi-desktop`, and `openlogi-hidpp`).

### Key Audit Discoveries
1. **Double-Multiplication Artifact on Diverted Thumbwheel Vertical Remaps:** When a user configures an MX Master thumbwheel to perform Vertical Scrolling (`Action::ScrollUp` / `Action::ScrollDown`), the rotation delta is scaled by **Thumbwheel Sensitivity** ($S_{\text{thumb}}$) in `dispatch_scroll` before being enqueued to the scroll runtime worker, where it is scaled **a second time** by **Vertical Scroll Sensitivity** ($S_{\text{vert}}$) in `run_worker`.
2. **Acceleration & Sensitivity Order of Operations:** Acceleration gain $A(v)$ is calculated based on the *unscaled physical rate* (raw ticks/sec), but then applied multiplicatively ($S \times A$) with baseline sensitivity factor $S$. While this avoids distorting velocity estimation, $S \times A$ produces compound gains (up to $100 \times 3.0 = 300\times$ distance) at high scroll speeds.
3. **UI/UX Control Redundancy & Ambiguity:**
   - **Thumbwheel Sensitivity** is presented as a global control in General Settings, but is also stored as an optional per-device override (`devices.<key>.thumbwheel_sensitivity`).
   - **Horizontal Scroll Acceleration** in General Settings operates on horizontal wheel impulses ($x$), but users frequently mistake it for controlling thumbwheel speed. Conversely, diverted thumbwheel horizontal scrolling passes into the scroll runtime with unity gain unless horizontal acceleration is explicitly enabled.
   - **Smooth Scrolling** global toggle affects all standard vertical wheel hook events and diverted HID++ thumbwheel events, but native trackpad/touchpad scrolling bypasses it entirely.

---

## Phase 1: Pipeline & Runtime Audit

### 1. Architectural Dispatch Flowcharts

#### A. Native OS-Hook Scroll Event Flow (Vertical & Horizontal Wheel Input)
```
[ Physical Mouse Wheel Notch / Hook Event ]
                   │
                   ▼
┌────────────────────────────────────────────────────────┐
│ OS Hook Callback (CGEventTap / evdev / WH_MOUSE_LL)    │
│  - Captures WheelDelta::WheelTicks { x, y }             │
└────────────────────────────────────────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────────────────────┐
│ ScrollInputHandle::try_hook_scroll()                   │
│  - Check accepting flag (AtomicBool)                   │
│  - Load ScrollPreferenceSnapshot snapshot               │
│  - Determine Output Mode:                              │
│      • If smooth_scroll == true  => Smooth { at }      │
│      • Else if vertical/horizontal scaled => Direct    │
│      • Else => Reject (Fail open, pass to OS)          │
│  - try_send() via bounded mpsc channel (cap: 128)      │
└────────────────────────────────────────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────────────────────┐
│ ScrollRuntime Worker Loop (run_worker)                 │
│ 1. AccelerationEngine::compute_gains()                 │
│    Calculates velocity v (raw ticks/s over dt)         │
│    Computes gains: gain_x, gain_y                      │
│ 2. Sensitivity & Gain Application:                      │
│    scaled_x = impulse.x * gain_x                       │
│    scaled_y = impulse.y * (S_vert / 14.0) * gain_y     │
└────────────────────────────────────────────────────────┘
         │                                       │
         ▼ (If Smooth)                           ▼ (If Direct)
┌──────────────────────────────┐       ┌──────────────────────────────┐
│ ScrollEngine::impulse()      │       │ emit_direct()                │
│  - ActiveMotion cubic        │       │  - WheelDelta::post()        │
│    smoothstep interpolator   │       │  - openlogi_inject::        │
│  - 100ms duration, 8ms tick  │       │    post_scroll()            │
└──────────────────────────────┘       └──────────────────────────────┘
         │                                       │
         ▼                                       ▼
┌────────────────────────────────────────────────────────┐
│ OS Dispatch Injection Layer (openlogi-inject)          │
│  - macOS: CGEventCreateScrollWheelEvent                │
│  - Windows: SendInput(INPUT_MOUSE, MOUSEEVENTF_WHEEL)  │
│  - Linux: uinput / evdev injection                     │
└────────────────────────────────────────────────────────┘
```

#### B. Diverted HID++ Thumbwheel Event Flow
```
[ Physical Thumbwheel Rotation Event (0x2150 Feature) ]
                   │
                   ▼
┌────────────────────────────────────────────────────────┐
│ HID++ Session Receiver (CapturedInput::Scroll)         │
│  - Extract raw increments & WheelResolution            │
└────────────────────────────────────────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────────────────────┐
│ InputDispatcher::dispatch_scroll()                     │
│  - Look up DeviceCapturePlan & WheelConfiguration      │
│  - WheelAccumulators::advance() for physical direction │
└────────────────────────────────────────────────────────┘
         │                                       │
         ▼ (If Bound to Action)                  ▼ (If Continuous Scroll)
┌──────────────────────────────┐       ┌─────────────────────────────────┐
│ Discrete Action Accumulator  │       │ ScrollScale::per_increment()    │
│  - Threshold: T(S_thumb)     │       │   res.native_per_increment()    │
│    = max(1, 28 - S_thumb)   │       │   * (S_thumb / 14.0)            │
│  - Decay: 300ms idle reset   │       │  - Generate ScrollDelta         │
│  - Cooldown: 200ms min gap   │       └─────────────────────────────────┘
│  - Fire Action on threshold  │                        │
└──────────────────────────────┘                        ▼
                                       ┌─────────────────────────────────┐
                                       │ ScrollInputHandle::            │
                                       │   try_hidpp_scroll()            │
                                       │  - Mode: Smooth { at }          │
                                       └─────────────────────────────────┘
                                                        │
                                                        ▼
                                       ┌─────────────────────────────────┐
                                       │ ScrollRuntime Worker Loop       │
                                       │  - Compute Acceleration Gains   │
                                       │  - Apply S_vert (DOUBLE MULT!)  │
                                       │  - Interpolate / Inject         │
                                       └─────────────────────────────────┘
```

---

### 2. Detailed Mathematical Analysis of Sensitivity & Acceleration

#### A. Multiplier Definitions
1. **Vertical Scroll Sensitivity Multiplier ($M_{\text{vert}}$):**
   $$M_{\text{vert}}(S_{\text{vert}}) = \frac{S_{\text{vert}}}{S_{\text{default}}} = \frac{S_{\text{vert}}}{14}$$
   Where $S_{\text{vert}} \in [1, 100]$. Default $S_{\text{vert}} = 14 \implies M_{\text{vert}} = 1.0$.

2. **Thumbwheel Sensitivity Multiplier ($M_{\text{thumb}}$):**
   $$M_{\text{thumb}}(S_{\text{thumb}}) = \frac{S_{\text{thumb}}}{S_{\text{default}}} = \frac{S_{\text{thumb}}}{14}$$
   Where $S_{\text{thumb}} \in [1, 100]$. Default $S_{\text{thumb}} = 14 \implies M_{\text{thumb}} = 1.0$.

3. **Discrete Action Threshold ($T_{\text{action}}$):**
   $$T_{\text{action}}(S_{\text{thumb}}) = \max\left(1, 2 \times 14 - S_{\text{thumb}}\right) = \max(1, 28 - S_{\text{thumb}})$$
   At default $S_{\text{thumb}} = 14$, $T_{\text{action}} = 14$ rotation units required. At $S_{\text{thumb}} = 27$, $T_{\text{action}} = 1$ unit.

4. **Acceleration Gain ($A_{\text{axis}}$):**
   Calculated in `crates/openlogi-core/src/math.rs` via `gain_at_rate`:
   $$\hat{v}_k = (1 - \alpha)\hat{v}_{k-1} + \alpha v_k \quad (\alpha = 0.4)$$
   $$A(v) = 1.0 + \text{factor} \times \left(\frac{\max(0, \hat{v} - v_{\text{ref}})}{v_{\text{scale}}}\right)^{1.5}$$
   Clamped to $[1.0, \text{max\_gain}]$. ($v_{\text{ref}} = 4.0\text{ ticks/s}$, $v_{\text{scale}} = 10.0$).

#### B. Pipeline Equations & Double-Multiplication Trace

##### Standard Vertical Wheel Impulse ($y_{\text{raw}}$):
$$\text{scaled\_impulse}_y = y_{\text{raw}} \times M_{\text{vert}} \times A_{\text{vert}}(\hat{v}_y) = y_{\text{raw}} \times \left(\frac{S_{\text{vert}}}{14}\right) \times A_{\text{vert}}(\hat{v}_y)$$

##### Standard Horizontal Wheel Impulse ($x_{\text{raw}}$):
$$\text{scaled\_impulse}_x = x_{\text{raw}} \times 1.0 \times A_{\text{horiz}}(\hat{v}_x)$$
*(Note: Horizontal wheel input currently has no independent base sensitivity setting, only acceleration gain $A_{\text{horiz}}$!)*

##### Diverted Thumbwheel to Horizontal Scroll (`Action::HorizontalScrollRight` / `Left`):
In `dispatch_scroll` (`wheel.rs`):
$$\delta_{\text{thumb}} = \text{increments} \times \text{resolution.native\_per\_increment}() \times \left(\frac{S_{\text{thumb}}}{14}\right)$$
In `run_worker` (`worker.rs`):
$$\text{scaled\_impulse}_x = \delta_{\text{thumb}} \times A_{\text{horiz}}(\hat{v}_x) = \text{increments} \times \text{res\_scale} \times \left(\frac{S_{\text{thumb}}}{14}\right) \times A_{\text{horiz}}(\hat{v}_x)$$

##### Diverted Thumbwheel to Vertical Scroll (`Action::ScrollUp` / `Down`) — THE DOUBLE MULTIPLICATION BUG:
In `dispatch_scroll` (`wheel.rs`):
$$\delta_{\text{thumb}} = \text{increments} \times \text{resolution.native\_per\_increment}() \times \left(\frac{S_{\text{thumb}}}{14}\right)$$
When `try_hidpp_scroll` enqueues $\delta_{\text{thumb}}$ as a vertical impulse $y = \delta_{\text{thumb}}$ into `run_worker` (`worker.rs`), line 335 evaluates:
$$\text{scaled\_impulse}_y = y \times M_{\text{vert}} \times A_{\text{vert}}(\hat{v}_y) = \left[\text{increments} \times \text{res\_scale} \times \left(\frac{S_{\text{thumb}}}{14}\right)\right] \times \left(\frac{S_{\text{vert}}}{14}\right) \times A_{\text{vert}}(\hat{v}_y)$$

> ⚠️ **CRITICAL FINDING:** Diverted thumbwheels remapped to vertical scrolling experience compounding scaling:
> $$\text{Total Multiplier} = \left(\frac{S_{\text{thumb}}}{14}\right) \times \left(\frac{S_{\text{vert}}}{14}\right) \times A_{\text{vert}}$$
> If both sensitivities are set to $50$, the static scale factor is $\frac{50}{14} \times \frac{50}{14} = 3.57 \times 3.57 = 12.755\times$ (instead of the expected $3.57\times$).

---

### 3. Smooth Scrolling State Machine & Interpolator

The smooth scroll engine (`ScrollEngine` in `crates/openlogi-agent-core/src/runtime/scroll.rs`) implements a finite cubic smoothstep motion segment over a fixed duration of **100 ms** (`ANIMATION_DURATION`) evaluated at an 8 ms tick cadence (125 Hz).

#### Easing Curve Equation
Given progress ratio $p = \frac{\Delta t}{100\text{ ms}} \in [0.0, 1.0]$:
$$f(p) = p \times (1.2 + p \times (0.6 - 0.8p)) = 1.2p + 0.6p^2 - 0.8p^3$$

#### Kinetic Characteristics:
- **Initial Velocity ($p = 0$):** $f'(0) = 1.2$ — Gives instant non-zero response on Frame 1, avoiding initial lag or deadband.
- **Landing Velocity ($p = 1$):** $f'(1) = 1.2 + 2(0.6)(1) - 3(0.8)(1)^2 = 1.2 + 1.2 - 2.4 = 0.0$ — Ensures smooth deceleration to zero.
- **Total Distance ($p = 1$):** $f(1) = 1.2 + 0.6 - 0.8 = 1.0$ — 100% position conservation.

#### Phase Stream Lifecycle
Foreground dispatch receives a structured stream of phases (`SmoothScrollPhase`):
1. **`Began`**: Emitted on the first non-zero motion frame when transitioning from `OutputStream::Idle` to `Active`.
2. **`Changed`**: Emitted on subsequent frames during active motion or when retargeting occurs.
3. **`Ended`**: Emitted when all active source segments complete.
4. **`Cancelled`**: Emitted when an OS hook generation bump or HID++ session cancellation invalidates the source stream.

#### Retargeting Behavior
When a new wheel tick arrives mid-animation ($t_{\text{new}} < t_{\text{start}} + 100\text{ms}$):
1. The position $P_{\text{current}} = \text{position\_at}(t_{\text{new}})$ is computed.
2. The remaining target is accumulated: $T_{\text{new}} = T_{\text{old}} + \Delta_{\text{new}}$.
3. A new `MotionSegment` is started from $P_{\text{current}}$ to $T_{\text{new}}$ with $t_{\text{start}} = t_{\text{new}}$.
4. This guarantees zero position loss and smooth acceleration under continuous high-frequency wheel ticks.

---

## Phase 2: Desktop UI & Configuration Analysis

### 1. Control Overlaps & User Saliency Issues

#### Analysis of `openlogi-desktop/src/windows/settings/general.rs`
The current General Settings page arranges scroll controls in a flat linear list:
1. `Smooth scrolling` (Toggle)
2. `Vertical Scroll Sensitivity` (Slider 1..100)
3. `Vertical Scroll Acceleration` (Preset Select + Strength Slider + Multiplier Slider + Curve Graph)
4. `Horizontal Scroll Acceleration` (Preset Select + Strength Slider + Multiplier Slider + Curve Graph)
5. `Thumb Wheel Sensitivity` (Slider 1..100)

#### Key Saliency & Confusion Points Identified:
1. **Vertical Scroll Sensitivity vs. Vertical Acceleration:**
   - Both controls affect vertical wheel speed, but users cannot easily discern which control to tweak when scrolling feels "too fast" or "too slow".
   - Vertical Sensitivity scales distance per physical notch linearly, whereas Acceleration increases speed non-linearly during fast scrolling.
2. **Horizontal Scroll Acceleration vs. Thumbwheel Sensitivity:**
   - Users with thumbwheel hardware (MX Master series) routinely adjust *Horizontal Scroll Acceleration* expecting it to change their thumbwheel continuous scroll speed.
   - However, thumbwheel continuous scroll speed is actually controlled by *Thumb Wheel Sensitivity*!
   - Furthermore, horizontal acceleration is **Disabled by default** (`horizontal_acceleration_enabled = false`), leading users to believe horizontal scrolling is unresponsive.
3. **Global vs. Per-Device Configuration Ambiguity:**
   - `ThumbwheelSensitivity` exists in global `AppSettings` *and* as an optional override in `DeviceConfig` (`devices.<key>.thumbwheel_sensitivity`).
   - In `openlogi-desktop/src/features/pointer/smartshift.rs` and `features/mouse/view.rs`, thumbwheel settings appear in device views, but modifying `general.rs` updates `app_settings.thumbwheel_sensitivity`.
   - When a device-level override exists, changing the General Settings slider has **no effect** on that device, creating silent configuration failure.

---

### 2. UI Density & State Duplication Analysis

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ CONFIGURATION HIERARCHY & RESOLUTION PRECEDENCE                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   [ Global AppSettings ]                 [ Device Config Override ]         │
│   - smooth_scroll                        - thumbwheel_sensitivity (Opt)    │
│   - vertical_scroll_sensitivity          - hi_res_wheel_enabled (Opt)      │
│   - vertical_acceleration_*              - smartshift_settings (Opt)        │
│   - horizontal_acceleration_*                                               │
│   - thumbwheel_sensitivity                                                  │
│                                                                             │
│                                      │                                      │
│                                      ▼                                      │
│                         ┌──────────────────────────┐                        │
│                         │ Effective Capture Plan   │                        │
│                         │ (capture_plan.rs)        │                        │
│                         └──────────────────────────┘                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### State Sync Breakdown:
- `openlogi-desktop/src/state/settings.rs` maintains entity sliders:
  - `vertical_scroll_sensitivity_slider`
  - `thumbwheel_sensitivity_slider`
  - `v_accel_slider`, `v_max_gain_slider`
  - `h_accel_slider`, `h_max_gain_slider`
- `set_device_thumbwheel_sensitivity(key, value)` sets the per-device override if `value != app_settings.thumbwheel_sensitivity`. If the value equals global default, it clears the device override back to `None`.
- **Bug/Pitfall:** Setting `general.rs`'s global slider does not notify or refresh active per-device settings sliders if a device-specific window is open simultaneously.

---

## Phase 3: Consolidation Proposal & Deliverables

### 1. Architectural Consolidation Plan

#### Fix for Double-Multiplication Bug
In `crates/openlogi-agent-core/src/runtime/scroll/worker.rs`, update `run_worker` input handling:
- Check `input.source`: if `input.source` is `ScrollSource::Hidpp(_)`, **do not** re-apply `prefs.vertical_sensitivity.scroll_multiplier()`, because HID++ thumbwheel input has already been scaled by `ScrollScale` ($S_{\text{thumb}}$) in `dispatch_scroll`.
- Alternatively, pass an explicit `sensitivity_applied: bool` flag inside `ScrollInput`.

#### Fix for Horizontal Scroll Consistency
Provide clear, unified axis handling so horizontal wheel input and thumbwheel horizontal input share the same consistent baseline sensitivity and acceleration pipeline without double-scaling.

---

### 2. Streamlined UI/UX Hierarchy Recommendations

To eliminate redundant controls and clear up user confusion, we propose reorganizing **General Settings** (`general.rs`) into distinct, logical groups with clear visual separation.

#### ASCII Mockup: Reorganized General Settings (`general.rs`)

```
===============================================================================
 GENERAL SETTINGS - SCROLLING & ACCELERATION
===============================================================================

 ┌───────────────────────────────────────────────────────────────────────────┐
 │ Smooth Scrolling                                                          │
 │ [ON/OFF Toggle]  Animate physical mouse wheel notches for fluid scrolling  │
 └───────────────────────────────────────────────────────────────────────────┘

 ─── VERTICAL SCROLLING ───────────────────────────────────────────────────────

 ┌───────────────────────────────────────────────────────────────────────────┐
 │ Baseline Distance (Sensitivity)                                           │
 │ [═══████───────]  14 (Default)                                            │
 │ Controls distance scrolled per physical wheel notch.                      │
 └───────────────────────────────────────────────────────────────────────────┘

 ┌───────────────────────────────────────────────────────────────────────────┐
 │ Dynamic Speed Acceleration                                                │
 │ Preset: [ Balanced  ▼ ]                                                   │
 │                                                                           │
 │   ┌──────────────────────────┐   Acceleration Factor: [══████───] 100%    │
 │   │  Curve Preview Graph     │   Max Speed Multiplier: [═══████──] 2.5x   │
 │   │  (Velocity vs Gain)      │                                            │
 │   └──────────────────────────┘   [ Reset to Default ]                     │
 └───────────────────────────────────────────────────────────────────────────┘

 ─── HORIZONTAL & THUMBWHEEL SCROLLING ───────────────────────────────────────

 ┌───────────────────────────────────────────────────────────────────────────┐
 │ Thumbwheel & Horizontal Sensitivity                                       │
 │ [═══████───────]  14 (Default)                                            │
 │ Controls scroll speed for side thumbwheels and horizontal wheel tilting.  │
 └───────────────────────────────────────────────────────────────────────────┘

 ┌───────────────────────────────────────────────────────────────────────────┐
 │ Horizontal Speed Acceleration                                             │
 │ Preset: [ None / Disabled ▼ ]                                             │
 │ (Expandable acceleration curve controls for horizontal panning)           │
 └───────────────────────────────────────────────────────────────────────────┘

===============================================================================
```

#### ASCII Mockup: Device Settings (`features/mouse/view.rs` / Thumbwheel Control)

```
===============================================================================
 DEVICE SETTINGS - MX MASTER THUMBWHEEL CONFIGURATION
===============================================================================

 Thumbwheel Mode:
 (•) Horizontal Scroll (Default)
 ( ) Vertical Scroll
 ( ) Discrete Action / Custom Key Assignment

 ┌───────────────────────────────────────────────────────────────────────────┐
 │ Thumbwheel Sensitivity Override                                           │
 │ [ ] Use Global Preference (14)                                            │
 │ [x] Custom Device Sensitivity: [════████─────] 24                         │
 └───────────────────────────────────────────────────────────────────────────┘

 Notice: When mapped to Vertical Scroll, global Vertical Acceleration rules
 apply, but distance scaling is unified to prevent double-acceleration.
===============================================================================
```

---

## Phase 4: Platform & Compatibility Guarantee

### 1. Zero IPC / Wire Format Breaking Changes
- `PROTOCOL_VERSION` in `crates/openlogi-ipc/src/ipc.rs` remains untouched.
- All proposals rely on internal runtime adjustments (`ScrollInput`, `run_worker`, `WheelConfiguration`) and UI presentation adjustments (`general.rs`), keeping IPC serde structs unchanged.

### 2. Platform Invariants & Injection Consistency
- **macOS (`CGEvent` / `CGEventTap`):** Continuous smooth scroll frames post `CGEvent` with `kCGScrollWheelEventIsContinuous` set to 1. Direct mode posts discrete wheel ticks.
- **Windows (`WH_MOUSE_LL` / `SendInput`):** Low-level hook intercepts `WM_MOUSEWHEEL` and `WM_MOUSEHWHEEL`. Direct and smooth frames use `MOUSEEVENTF_WHEEL` / `MOUSEEVENTF_HWHEEL`.
- **Linux (`evdev` / `uinput`):** Relies on `REL_WHEEL` / `REL_HWHEEL` or high-resolution `REL_WHEEL_HI_RES` / `REL_HWHEEL_HI_RES`.
- **Remainder Carryover:** The `ScrollQuantizer` / fractional remainder tracking ensures sub-pixel scroll increments are preserved across frames without drift or deadbands across all platform targets.
