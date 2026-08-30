### Task: Scroll Engine & Acceleration Architecture Overhaul

#### Objective
Refactor and optimize the entire scroll acceleration and smoothing pipeline across `crates/openlogi-core`, `crates/openlogi-agent-core`, and `crates/openlogi-desktop`. Eliminate input lag on single ratchet clicks, migrate hot-path calculations to zero-allocation fixed-point arithmetic (`i32`), unify smooth scrolling into curve presets (removing standalone toggles), and modernize the configuration UI.

---

### Phase 1: Core Mathematical Model & Data Architecture (`openlogi-core`)

1. **Fixed-Point Arithmetic & Parameter Renaming:**
   - Implement fixed-point scaling (`i32` with a `1_000` scale factor, where `1000 = 1.000x`) for all scroll baselines, multipliers, and gain caps in `math.rs` and `config/settings.rs`.
   - **Rename Sensitivity Fields (with Serde Backwards Compatibility):**
     - Rename `VerticalScrollSensitivity` -> `VerticalScrollBaseline` (Display: `"Vertical Scroll baseline"`).
     - Rename `ThumbwheelSensitivity` -> `HorizontalScrollBaseline` (Display: `"Horizontal Scroll baseline"`).
     - Add serde aliases (`#[serde(alias = "vertical_scroll_sensitivity")]`, `#[serde(alias = "thumbwheel_sensitivity")]`) to maintain configuration deserialization compatibility without bumping the wire version.
   - Store and compute baseline values natively as fixed-point integers to avoid `f64 -> i32` conversion overhead on hot paths.

2. **Unified Curve Presets & LUTs:**
   - Define `CurvePreset`:
     - `None` (1:1 direct passthrough, disables dynamic curve processing).
     - `Smooth` (Low-latency easing decay with immediate Frame 0 impulse).
     - `Linear`
     - `Exponential`
     - `Logarithmic`
     - `Quadratic` / `Cubic`
     - `Custom` (Configured via Multiplier and Cap parameters).
   - Precompute curve functions into 256-slot static Lookup Tables (LUTs) keyed by input velocity bins on config load/update to enable $O(1)$ integer gain evaluation.

---

### Phase 2: High-Performance Scroll Engine Runtime (`openlogi-agent-core`)

1. **Frame 0 Fast-Path & Immediate Impulse:**
   - In `runtime/scroll/worker.rs` and `runtime/scroll.rs`:
     - If the engine is idle (>250ms pause) or a discrete single notch arrives, dispatch the full quantized baseline step immediately on Frame 0 (bypassing the timer loop).
     - For rapid continuous scrolling within the active window under `CurvePreset::Smooth`, transition seamlessly into sub-pixel easing decay.

2. **Inter-Tick Velocity Seeding & State Resets:**
   - Seed velocity instantly on Tick 2 based directly on inter-tick timestamp difference ($\Delta t$), eliminating Exponential Moving Average (EMA) ramp-up lag.
   - Enforce a strict minimum base multiplier floor ($gain \ge 1000$ / $1.000\times$).
   - Maintain fixed-point sub-pixel remainders and clear accumulators instantly on:
     - Direction reversal ($\text{sign}(\Delta_t) \ne \text{sign}(\Delta_{t-1})$).
     - Axis switch ($dx \leftrightarrow dy$).
     - Idle timeout (>250ms).

3. **Zero-Allocation Hot Path & Adaptive Thread Parking:**
   - Replace heap/Vec allocations in `advance_due` with stack/array buffers.
   - Replace lock contention on `preferences.load()` with atomic/snapshot reads.
   - Dynamically park the worker thread when active decay falls below visible thresholds to eliminate idle CPU wakeups.

---

### Phase 3: Desktop UI & Visualizer Overhaul (`openlogi-desktop`)

1. **General Settings View (`windows/settings/general.rs`):**
   - **Remove Standalone Toggles:** Remove the legacy "Smooth scrolling", "Vertical Scroll Acceleration", and "Horizontal Scroll Acceleration" switch toggles.
   - **Controls & Sliders:**
     - Render `CurvePreset` selector dropdown.
     - Display native fixed-point sliders:
       - **Vertical Scroll baseline** / **Horizontal baseline**
       - **Multiplier** (Base gain scalar)
       - **Cap** (Maximum gain ceiling)
     - Adjusting Multiplier or Cap automatically sets the preset to `Custom`.
     - Selecting `None` collapses and hides the curve visualizer and fine-tuning sliders for that axis.

2. **Live Acceleration Curve Graph (`windows/settings/general/curve.rs`):**
   - **Horizontal Axis ($X$):** Input Speed / Velocity ($\text{ticks/sec}$).
   - **Vertical Axis ($Y$):** Gain Multiplier (dynamic ceiling mapped to the active Cap).
   - Use cached point samples derived from pure `openlogi-core::math` routines to prevent window redraw churn.

3. **Layout Padding & Info Tooltips:**
   - Wrap settings content in `Scrollable<Div>` with right gutter padding to prevent control cropping.
   - Use `IconName::Info` hover tooltips for concise helper text:
     - *Vertical Scroll baseline:* `"Controls baseline scrolling distance per vertical wheel notch."`
     - *Horizontal baseline:* `"Controls baseline scrolling distance per horizontal thumbwheel notch."`
     - *Multiplier:* `"Base acceleration gain factor applied to scroll velocity."`
     - *Cap:* `"Maximum multiplier ceiling limiting peak scroll speed."`

---

### Phase 4: Verification & Quality Gate

1. **Unit & Integration Tests:**
   - Fixed-point quantization, LUT lookup accuracy, single-tick Frame 0 impulse, accumulator resets, and preset switching.
2. **Workspace Verification Suite:**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -D warnings
   cargo test --workspace
