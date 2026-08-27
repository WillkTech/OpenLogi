# Audit & Investigation: Debug Output and Live Event Logging

This document provides a comprehensive audit of debug output, trace logging, diagnostic surfaces, and live event monitoring capabilities in the OpenLogi codebase across `openlogi-hid`, `openlogi-hidpp`, `openlogi-hook`, `openlogi-agent-core`, `openlogi-ipc`, `openlogi-core`, and `openlogi-cli`. It covers platform-specific behaviors across macOS, Linux, and Windows, defines the Live Input & Telemetry Capability Matrix, and proposes a zero-allocation, low-latency live debug streaming architecture (`openlogi monitor`).

---

## 1. Logging & Debug Architecture Audit

### 1.1 Tracing & Log Levels (`openlogi-core`, `openlogi-agent-core`)

OpenLogi uses the `tracing` ecosystem for structured diagnostic logging. Subscriber initialization and target filtering are governed by environment variables (`RUST_LOG` / `OPENLOGI_LOG`).

* **Subscriber Initializers:**
  - Agent and desktop binaries configure `tracing_subscriber::fmt` with `EnvFilter::from_default_env()` checking `RUST_LOG` or `OPENLOGI_LOG`.
  - Default filter directive: `info` level across workspace crates (`openlogi_*=info`), suppressing `debug` and `trace` events unless explicitly enabled.

* **Log Detail by Subsystem:**
  - **USB Discovery & Device Inventory (`openlogi-hid`, `openlogi-device`):**
    - `openlogi-hid` logs device enumeration, backend opening, and descriptor matching at `debug` level (`tracing::debug`).
    - Device discovery failures, permissions issues (Input Monitoring / Accessibility), and probe cache misses are logged at `warn` level.
    - Low-level raw USB descriptor parsing and report descriptor bytes are **not logged** even at `trace` level.
  - **HID++ Report Read/Write Loops (`openlogi-hidpp`):**
    - `openlogi-hidpp::channel` emits `trace!(dev, feat, func, "hidpp request")` and `trace!("raw report received")` for every sent request and incoming report when `RUST_LOG=openlogi_hidpp=trace` is enabled.
    - Raw byte payload hex dumps (`payload = {:02x?}`) are capped at 16 bytes for long reports (`0x11` / 20 bytes). Very long reports (e.g., `0x12` / 64 bytes) are truncated in trace output.
  - **OS Input Hook Interception (`openlogi-hook`, `openlogi-agent-core::runtime::hook`):**
    - Successful button remapping and gesture swipes log execution actions at `info` level (`info!(button, action, "gesture click → executing bound action")`).
    - Raw mouse deltas (`MouseEvent::Moved`) and high-frequency scroll ticks (`MouseEvent::Scroll`) are **completely omitted from `tracing` logs** to avoid locking/allocating on the freeze-sensitive hook callback thread.
  - **Scroll Engine & Acceleration Curves (`openlogi-agent-core::runtime::scroll`):**
    - Queue overflows and worker unavailability warn at `warn` level (`warn!("scroll output queue full — input rejected")`).
    - Sub-pixel accumulator remainders, gain multipliers, and step-by-step curve evaluation frames are not logged to tracing.

---

### 1.2 Platform-Specific Hook & Transport Diagnostic Details

#### macOS (`CGEventTap` & `IOKit` / `IOHIDManager`)
* **OS Hook (`crates/openlogi-hook/src/macos.rs`):**
  - Installs a `CGEventTap` on `kCGHIDEventTap` running on a dedicated thread with a high-priority `CFRunLoop`.
  - Callback thread performance is critical: `kCGEventTapDisabledByTimeout` or `kCGEventTapDisabledByUserInput` trigger auto-disabling by macOS if the callback takes longer than ~10ms.
  - To prevent tap eviction, the callback avoids `tracing` log allocations and uses `try_lock()` for `EventMonitor` recording.
  - Device attribution relies on `IOKit` sender IDs (`source_is_remappable`) to prevent intercepting built-in Apple trackpads.
* **Transport (`crates/openlogi-hid/src/transport/native.rs`):**
  - Uses `IOHIDManager` for USB/Bluetooth HID node enumeration.
  - Short HID++ reports (`0x10`, 7 bytes) over Bluetooth LE direct connections are converted to long HID++ reports (`0x11`, 20 bytes) via `normalize_outgoing()` because macOS `IOHIDDeviceSetReport` rejects 7-byte reports on BLE nodes.

#### Linux (`evdev` / `uinput` & `udev` / `hidraw`)
* **OS Hook (`crates/openlogi-hook/src/linux.rs`):**
  - Spawns per-device `evdev` reader threads monitoring `/dev/input/event*` nodes.
  - Synthesized actions are injected via `uinput` virtual input devices (`crates/openlogi-inject/src/inject/linux.rs`).
  - Active window tracking connects to D-Bus (`org.gnome.Shell` or `wlr-foreign-toplevel`).
* **Transport (`crates/openlogi-hid/src/transport/native.rs`):**
  - Uses `/dev/hidraw*` via `udev` for raw HID++ read/write loops.

#### Windows (`WH_MOUSE_LL` / Raw Input & `SetupAPI`)
* **OS Hook (`crates/openlogi-hook/src/windows.rs`):**
  - Installs a low-level mouse hook (`WH_MOUSE_LL`) with a dedicated Win32 message loop (`GetMessage` / `DispatchMessage`).
  - Synthesized actions use `SendInput`.
* **Transport (`crates/openlogi-hid/src/transport/windows_hid.rs`):**
  - Uses `SetupAPI` and `CreateFileW` on `\\?\HID#...` device paths.

---

### 1.3 CLI & IPC Debug Surfaces (`openlogi-cli`, `openlogi-ipc`)

* **Existing CLI Diagnostic Commands (`openlogi-cli`):**
  - `openlogi diag features`: Dumps HID++ feature table (ID, version, index).
  - `openlogi diag controls`: Dumps HID++ 0x1b04 reprogrammable control IDs and flags.
  - `openlogi diag battery`: Reads raw battery status (0x1004 / 0x1000).
  - `openlogi diag dpi`: Reads and tests DPI sensor step adjustments.
  - `openlogi diag smartshift`: Reads and toggles SmartShift ratchet modes.
  - `openlogi diag lighting`: Writes solid RGB color values to wired HID++ devices.
  - `openlogi diag wheel`: Reads/writes HID++ 0x2121 wheel resolution settings.
  - *Gap:* No CLI command currently exists to stream real-time input events or monitor IPC activity (`openlogi monitor` is missing).

* **IPC Protocol Support (`openlogi-ipc`):**
  - Wire protocol version: `PROTOCOL_VERSION = 28`.
  - Built on `tarpc` over local domain/named sockets.
  - Protocol v9 introduced `Agent::poll_event_monitor()` returning `Vec<MonitorEvent>`.
  - Current `MonitorEvent` variants:
    - `MonitorEvent::Button { button: String, pressed: bool }`
    - `MonitorEvent::Scroll { delta_x: f32, delta_y: f32 }`
    - `MonitorEvent::CaptureInterrupted`
  - *Gaps in IPC Monitor:*
    - Device source attribution (`device_id` / `vendor_id:product_id`) is missing.
    - HID++ raw packet headers (CID, feature index, software ID) are missing.
    - Acceleration gain factor and scroll remainder telemetry are missing.
    - Gesture state machine transitions (`SwipeAccumulator` drag vectors, directions) are missing.

---

### 1.4 Desktop UI Diagnostic Utilities (`openlogi-desktop`)

* `openlogi-desktop` features a live curve visualizer for custom scroll acceleration profiles.
* The UI polls `Agent::poll_event_monitor()` while the input settings panel is focused to render live button presses and scroll indicators.
* The monitor buffer is bounded (`CAPACITY = 256`) in `EventMonitor` (`crates/openlogi-agent-core/src/event_monitor.rs`).
* An idle janitor task (`run_idle_janitor`) auto-disables event recording if no IPC poll arrives within 3 seconds, ensuring zero overhead when UI panels are closed.

---

## 2. Live Input & Telemetry Capability Matrix

| Subsystem / Stream | Target Files / Modules | Current Log Level (`trace`/`debug`/`info`) | Live Stream Exists? (CLI / UI / Log) | Gaps & Omissions |
| :--- | :--- | :--- | :--- | :--- |
| **USB Transport / HID Enumeration** | `crates/openlogi-hid`<br>`crates/openlogi-device` | `debug` (enumeration)<br>`warn` (open errors) | **Log Only** (`RUST_LOG=openlogi_hid=debug`) | Raw USB setup packets and HID report descriptor byte dumps are omitted. |
| **Raw HID++ Packets (Receiver / Direct)** | `crates/openlogi-hidpp/src/channel.rs` | `trace` (`openlogi_hidpp=trace`) | **Log Only** (`RUST_LOG=openlogi_hidpp=trace`) | Reports >16 bytes truncated in trace; no IPC/CLI wire monitor. |
| **OS Input Hook (Buttons 1–5, Remap)** | `crates/openlogi-hook`<br>`crates/openlogi-agent-core/src/runtime/hook.rs` | `info` (action dispatch)<br>`warn` (tap errors) | **Partial IPC / UI** (`poll_event_monitor`) | Device ID attribution missing; raw pointer deltas (`Moved`) omitted. |
| **Wheel & Scroll Delta Processing** | `crates/openlogi-agent-core/src/runtime/scroll/worker.rs`<br>`acceleration.rs` | `warn` (queue full) | **Partial IPC / UI** (raw deltas only) | Gain factors, sub-pixel remainders, and curve evaluation metrics omitted. |
| **Gesture State Machine & Drag Vectors** | `crates/openlogi-agent-core/src/watchers/gesture.rs`<br>`openlogi-core/src/binding/swipe.rs` | `debug` (gesture dispatch)<br>`info` (swipe execute) | **Log Only** (`RUST_LOG=openlogi_agent_core=debug`) | Intermediate drag vectors `(dx, dy)` and hold state transitions omitted from IPC/UI. |

---

## 3. Architectural Recommendations & Deliverables

### 3.1 Gaps Analysis & High-Frequency Guardrails

High-frequency input streams (e.g. 1000 Hz USB polling rates, raw pointer deltas, and smooth scroll frames) present severe log-flooding and latency hazards if written directly to stdout or tracing subscribers on the main input path:

1. **Freeze-Sensitive Callbacks:**
   - Operating system hook threads (macOS `CGEventTap`, Windows `WH_MOUSE_LL`) must execute within tight time budgets (<10ms).
   - Allocating memory or acquiring blocking mutexes on every mouse movement event risks OS tap revocation.
2. **Buffer Eviction:**
   - High-rate mouse movement events (1000 Hz) would saturate bounded queues (e.g. `CAPACITY = 256`) in under 250ms, evicting button press/release events before the UI or CLI can poll them.
3. **Guardrail Requirements:**
   - Drop pointer move deltas (`MouseEvent::Moved`) from default IPC monitoring buffers.
   - Use atomic state flags (`AtomicBool`) for `enabled()` checks on hot paths.
   - Use `try_lock()` inside callbacks to drop diagnostic events under lock contention rather than blocking input processing.

---

### 3.2 Proposed Live Event Monitor Architecture (`openlogi monitor`)

To enable native, real-time inspection of low-level device connections, raw HID++ packets, OS hooks, scroll math, and gesture state machines, we propose extending `openlogi-ipc` and `openlogi-cli` with a dedicated telemetry streaming bus.

#### IPC Protocol Extension (`openlogi-ipc`)

1. **Protocol Version:** Bump `PROTOCOL_VERSION` to `29`.
2. **Enhanced Telemetry Event Type (`TelemetryEvent`):**

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TelemetrySubsystem {
    HidTransport,
    Hidpp,
    OsHook,
    ScrollEngine,
    Gesture,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TelemetryEvent {
    /// USB / Transport connection lifecycle
    DeviceConnection {
        vendor_id: u16,
        product_id: u16,
        device_name: String,
        connected: bool,
    },
    /// Raw HID++ packet exchange
    HidppPacket {
        device_route: String,
        software_id: u8,
        feature_index: u8,
        function_id: u8,
        direction: PacketDirection, // Incoming / Outgoing
        payload_hex: String,
    },
    /// OS Mouse Hook Capture
    OsHookInput {
        device_id: Option<String>,
        event_type: HookInputType, // ButtonPress, ButtonRelease, ScrollTick
        button_name: Option<String>,
        delta_x: f32,
        delta_y: f32,
        suppressed: bool,
    },
    /// Scroll worker interpolation frame
    ScrollFrame {
        source: String,
        raw_delta_y: f64,
        accel_gain: f64,
        out_delta_y: f64,
        remainder_y: f64,
    },
    /// Gesture state machine transition
    GestureState {
        control_id: u16,
        phase: String, // Hold, Dragging, Committed, Released
        drag_dx: i32,
        drag_dy: i32,
        resolved_direction: Option<String>,
    },
}
```

3. **IPC Service Method:**
   Add a streaming subscription or ring-buffered poll RPC:
   ```rust
   async fn poll_telemetry_events(subsystems: u8) -> Vec<TelemetryEvent>;
   ```

---

### 3.3 CLI Command Specification (`openlogi monitor`)

Implement subcommand `openlogi monitor` inside `openlogi-cli`:

#### Command Line Arguments
```bash
openlogi monitor [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    --json           Output events as raw JSON objects (one per line)
    --ndjson         Output events formatted as newline-delimited JSON

OPTIONS:
    -s, --subsystem <NAME>   Filter by subsystem: hid, hidpp, hook, scroll, gesture, all [default: all]
    -d, --device <QUERY>     Filter events by device name, slot, or VID:PID
    -l, --level <LEVEL>      Minimum event severity level: trace, debug, info [default: debug]
```

#### Example Formatted Text Output Streams

1. **OS Input Hook Event Stream:**
   ```text
   [00:12:34.102] [HOOK]   [DEVICE: 046d:c07e] BUTTON_DOWN (Button 4 / Side Back) -> SUPPRESSED (Remapped: MouseBack)
   [00:12:34.185] [HOOK]   [DEVICE: 046d:c07e] BUTTON_UP   (Button 4 / Side Back) -> SUPPRESSED
   ```

2. **Wheel & Scroll Processing Stream:**
   ```text
   [00:12:35.010] [SCROLL] [DEVICE: Logi Bolt] RAW: dy=+1.00 | SENS: 1.50x | ACCEL: 1.45x | OUT: +2.17 (rem: +0.17)
   [00:12:35.026] [SCROLL] [DEVICE: Logi Bolt] RAW: dy=+1.00 | SENS: 1.50x | ACCEL: 1.80x | OUT: +2.70 (rem: +0.07)
   ```

3. **HID++ Event & Gesture Tracking Stream:**
   ```text
   [00:12:36.450] [HID++]  [ROUTE: Bolt/Slot1] [SW_ID: 0x02] REQ  [Feat: 0x1b04, Func: 0x01]
   [00:12:36.458] [HID++]  [ROUTE: Bolt/Slot1] [SW_ID: 0x02] RESP [Feat: 0x1b04, Func: 0x01] -> OK
   [00:12:37.100] [GESTURE][CID: 0x00d0] GESTURE_HOLD -> DRAG (dx: +12, dy: -4) -> SWIPE_RIGHT
   ```

---

### 3.4 Zero-Allocation & Latency Guardrails

To preserve OpenLogi's low-latency performance guarantees while streaming real-time telemetry:

1. **Zero-Allocation Sampling on Hot Path:**
   - Subsystem event producers evaluate an `AtomicBool` flag before constructing any string or heap-allocated `TelemetryEvent`.
   - When no CLI or UI client is attached, the overhead cost remains a single atomic load (`Ordering::Relaxed`).

2. **Ring Buffer & Channel Backpressure:**
   - Telemetry event queues utilize a fixed-capacity lock-free or try-lock ring buffer (`CAPACITY = 512`).
   - If a CLI or UI subscriber falls behind consumption, the oldest telemetry events are dropped without blocking or slowing down input forwarding.

3. **Idle Janitor Teardown:**
   - The agent retains the auto-disable janitor pattern (`run_idle_janitor`), stopping telemetry collection automatically if no subscriber polls within 3 seconds.
