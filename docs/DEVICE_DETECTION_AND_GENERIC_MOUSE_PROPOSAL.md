# OpenLogi Device Detection Audit & Generic USB Mouse Architectural Proposal

## Executive Summary

This document presents a comprehensive audit of the OpenLogi codebase (`https://github.com/willearning1/OpenLogi`) regarding device discovery, enumeration pipelines, Logitech G90 support, and generic wired USB mouse compatibility.

OpenLogi currently operates as a specialized HID++ driver daemon and customization utility tailored for Logitech HID++ 1.0/2.0+ wireless and wired peripherals (Unifying, Bolt, Bluetooth Direct, USB HID++) alongside specialized USB lighting devices (Litra Glow/Beam).

Key findings:
1. **Device Discovery:** OpenLogi strictly pre-filters USB/Bluetooth devices during HID node scanning using Logitech's Vendor ID (`VID: 0x046D`) and specific vendor-defined HID Usage Pages (`0xFF00` Usage `0x0002`, `0xFF43` Usage `0x0202`, and `0xFF43` Usage `0x0602`). Standard USB HID Mouse/Pointer devices using generic HID Usage Page `0x0001` (Generic Desktop) and Usage `0x0002` (Mouse) or Usage `0x0001` (Pointer) are explicitly ignored at the transport layer.
2. **Logitech G90 Status:** The Logitech G90 is an optical wired gaming USB mouse (`VID: 0x046D`, `PID: ~0xC07E`) operating purely on standard USB HID boot/mouse protocols. It does not implement Logitech HID++ 1.0/2.0/4.5 feature registers (`0x0000` / `0x0001`). Consequently:
   - The G90's HID nodes do not match OpenLogi's `is_hidpp_candidate` criteria and are dropped during device scanning.
   - Even if forced into the HID++ channel opener, protocol negotiation times out during root feature handshake (`0x0000` / `0x0001`), leading OpenLogi to mark the device node as unhealthy/rejected.
3. **Generic USB Mouse Support:** Non-HID++ USB mice (including generic wired USB mice and legacy Logitech mice like the G90) are completely filtered out in `openlogi-hid` and `openlogi-agent-core`. Furthermore, `openlogi-hook` enforces a strict vendor guard (`source_is_remappable`) that drops non-Logitech input events.

To support the G90 and generic USB mice, this proposal outlines an architectural strategy to introduce a `GenericMouse` device archetype, decouple OS-level input hook processing from HID++ hardware control, and surface standard mouse controls in `openlogi-cli` and `openlogi-desktop`.

---

## Phase 1: Device Discovery & Enumeration Pipeline Audit

### 1. Transport & Enumeration Layer

OpenLogi's hardware discovery is structured across three primary crates:
- `openlogi-hid`: Platform transport abstraction over `async-hid` (wrapping `IOKit`/`IOHIDManager` on macOS, `SetupAPI`/hidapi/Raw Input on Windows, and `udev`/`sysfs`/`hidraw` on Linux).
- `openlogi-hidpp`: HID++ channel framing, request/response multiplexing, and protocol parsing.
- `openlogi-agent-core`: Background orchestrator and inventory watcher (`watchers/inventory.rs`).

#### Platform USB Filtering Matrix

| Platform | Native API / Transport | Discovery Filter Criteria | Secondary Filter / Rejection |
| :--- | :--- | :--- | :--- |
| **macOS** | `IOKit` / `IOHIDManager` via `async-hid` | `VID: 0x046D`<br>`UsagePage` / `UsageID` in `HIDPP_LONG_COLLECTIONS` | Rejects Litra Glow/Beam (`0xFF43/0x0202` PID `0xC900`) via `matches_litra()` |
| **Linux** | `udev` + `/sys/class/hidraw` via `async-hid` | `VID: 0x046D`<br>`UsagePage` / `UsageID` in `HIDPP_LONG_COLLECTIONS` | Filters out receiver child hidraw nodes (`is_receiver_child_node` matching sysfs parent paths) |
| **Windows** | `SetupAPI` / Raw Input via `async-hid` & `WindowsHidppChannel` | `VID: 0x046D`<br>`UsagePage` / `UsageID` in `HIDPP_LONG_COLLECTIONS` | Routes short (0x10) and long (0x11) collections across split device interfaces |

#### Criteria Code Analysis

In `crates/openlogi-hid/src/transport.rs`:
```rust
const HIDPP_LONG_COLLECTIONS: [(u16, u16, bool); 3] = [
    (0xff00, 0x0002, false), // USB, Logi Bolt / Unifying receivers, BT Classic
    (0xff43, 0x0202, true),  // Bluetooth Low Energy (BLE) direct peripherals
    (0xff43, 0x0602, false), // Wired G-series gaming keyboards
];

fn is_hidpp_candidate(
    vendor_id: u16,
    product_id: u16,
    usage_page: u16,
    usage_id: u16,
    receiver_child: bool,
) -> bool {
    vendor_id == LOGITECH_VENDOR_ID // 0x046d
        && is_hidpp_long_collection(usage_page, usage_id)
        && !matches_litra(vendor_id, product_id, usage_page, usage_id)
        && !receiver_child
}
```

Standard USB mouse interfaces expose Usage Page `0x0001` (Generic Desktop) and Usage `0x0002` (Mouse). Because `is_hidpp_candidate` strictly checks for vendor-specific usage pages (`0xFF00` / `0xFF43`), all standard HID mouse endpoints—whether from third-party manufacturers or legacy/wired Logitech mice—are immediately dropped during enumeration.

---

### 2. Logitech G90 Compatibility & HID++ Support

#### Hardware Specifications & Protocol Level
- **Device:** Logitech G90 Optical Gaming Mouse (Wired USB)
- **USB Vendor ID:** `0x046D` (Logitech)
- **USB Product ID:** `0xC07E` (or similar standard gaming USB PID)
- **Protocol:** Standard USB HID Boot Protocol / Standard Report Descriptor (Generic Desktop Mouse).
- **HID++ Status:** **None.** The G90 does not support HID++ 1.0, 2.0, or 4.5. It does not carry HID++ vendor collections (`0xFF00` or `0xFF43`) nor does it respond to HID++ feature discovery (`0x0000` Root Feature).

#### OpenLogi Behavior Analysis with G90
1. **Discovery Phase:**
   When a G90 is plugged in:
   - `enumerate_devices()` lists its `DeviceInfo` (`VID: 0x046D`, `PID: 0xC07E`, `UsagePage: 0x0001`, `UsageID: 0x0002`).
   - `is_hidpp_node()` calls `is_hidpp_candidate()`.
   - `is_hidpp_long_collection(0x0001, 0x0002)` evaluates to `false`.
   - The device is filtered out and never handed to `open_hidpp_channel()`.
2. **Hypothetical Channel Open & Protocol Timeout:**
   If `is_hidpp_candidate` were modified to bypass usage page checks for `VID 0x046D`:
   - `open_hidpp_channel()` would attempt to open the raw HID reader/writer.
   - `probe_direct()` in `crates/openlogi-device/src/inventory/probe.rs` would send HID++ 2.0 Feature Root queries (`0x0000`/`0x0001`) to index `0xFF`.
   - The G90 would ignore these requests.
   - The probe would time out after `DIRECT_DEVICE_PROBE` timeout (~500ms).
   - `probe_direct()` would log: `"feature walk did not complete — transient probe failure, keeping last-known identity"`.
   - The probe returns `NodeProbe::failed()`, marking the node as unhealthy, causing `NodeLedger` to drop the channel after grace expiration.

---

## Phase 2: Generic Wired USB Mouse Support & Fallbacks

### 1. Non-HID++ / Generic Device Recognition

Currently, non-HID++ USB mice (both non-Logitech mice and non-HID++ Logitech mice like the G90) are treated as non-existent by OpenLogi:
- **CLI (`openlogi list`):** Only displays devices inventoried via Bolt, Unifying, or HID++ direct probes (`DeviceInventory`) or Litra lights (`StandaloneDevice`). Generic USB mice are completely omitted.
- **Desktop UI (`openlogi-desktop`):** Device sidebar renders state strictly from `DeviceInventory` and `StandaloneDevice`. Non-HID++ mice have no model representation and cannot be selected.

### 2. Feature Decoupling (OS Hook vs. Direct HID++ Control)

OpenLogi contains two distinct operational subsystems:
1. **Direct Hardware / HID++ Control Channel (`openlogi-hidpp` / `openlogi-device`):** Communicates two-way over raw USB/BLE HID reports.
2. **OS Input Hook Stream (`openlogi-hook`):** Intercepts global OS-level pointer, button, and scroll events (`CGEventTap` on macOS, `evdev` + `uinput` on Linux, `WH_MOUSE_LL` on Windows).

#### Detailed Capability Matrix

| Feature | Hardware / HID++ Dependent? | OS Hook Driven (Universal)? | Functional on Generic Mouse / G90? |
| :--- | :---: | :---: | :---: |
| **DPI Presets (`0x2201`)** | **Yes** (Writes to device MCU) | No | **No** (Controlled via hardware DPI button or OS pointer speed) |
| **SmartShift (`0x2111`)** | **Yes** (Ratchet/Free-spin motor) | No | **No** (Physical wheel mechanical switch) |
| **Onboard RGB (`0x8070`)** | **Yes** (LED controller IC) | No | **No** (Fixed or hardware lighting) |
| **Battery Telemetry (`0x1000`/`0x1004`)** | **Yes** (PMIC/ADC registers) | No | **No** (N/A for wired; OS power for BT) |
| **Global & Per-App Button Remapping** | No | **Yes** (`openlogi-hook`) | **Yes** (Standard Buttons 1–5 / Middle / Side) |
| **Smooth Scrolling & Acceleration** | No | **Yes** (`openlogi-hook`) | **Yes** (Transforms delta ticks into smooth curves) |
| **Actions Ring Overlay** | No | **Yes** (`openlogi-overlay`) | **Yes** (Triggered via mapped button press) |
| **Gestures & Swipe Acceleration** | No | **Yes** (`openlogi-hook`) | **Yes** (Mouse movement while held button) |

#### OS Hook Filter Audit (`crates/openlogi-hook/src/lib.rs`)

Currently, `openlogi-hook` enforces strict vendor filtering before allowing button/scroll remapping:

```rust
pub fn source_is_remappable(device: Option<&EventDevice>) -> bool {
    match device {
        Some(d) if d.is_trackpad_like() => false,
        Some(d) => d.is_logitech(),
        None => false,
    }
}
```

Where `is_logitech()` checks:
```rust
pub fn is_logitech(&self) -> bool {
    if self.vendor_id == Some(LOGITECH_VENDOR_ID) { // 0x046d
        return true;
    }
    self.product_name.as_deref().is_some_and(|n| {
        let n = n.to_ascii_lowercase();
        n.contains("logitech") || n.starts_with("logi ")
    })
}
```

- **Logitech G90 Impact:** Because the G90 has `VID: 0x046D` and product name containing "Logitech", `d.is_logitech()` returns `true`. If OpenLogi detects the G90 as a generic device, the OS hook *already* has the vendor authorization to intercept its buttons and scroll events.
- **Generic Non-Logitech Mouse Impact:** Non-Logitech USB mice (e.g., Razer, SteelSeries, generic OEM) are rejected by `is_logitech()` returning `false`. To enable generic USB mouse support, this check must be decoupled.

---

## Phase 3: Deliverables & Recommendations

### 1. Detection Matrix & Audit Summary

- **HID++ Candidate Criteria:** Strict match on `VID 0x046D` AND `UsagePage/UsageID` in `{(0xFF00, 0x0002), (0xFF43, 0x0202), (0xFF43, 0x0602)}`.
- **Logitech G90 Support Status:**
  - **Currently Detected:** **No.**
  - **Reason:** Filtered out at usage page check; lacks HID++ 1.0/2.0 registers.
  - **Potential Functionality:** Eligible for 100% of OS hook-driven features (Button remapping, Smooth Scrolling, Scroll Acceleration, Actions Ring) if surfaced via a generic fallback scanner.
- **Generic USB Mouse Support Status:**
  - **Currently Supported:** **No.** Filtered out in `openlogi-hid` discovery and `openlogi-hook` event disposition filter.

---

### 2. Generic Mouse Architectural Proposal

We propose introducing a first-class **`GenericMouse`** archetype in OpenLogi. This architecture separates hardware HID++ capability management from universal OS input hook processing.

```
+-----------------------------------------------------------------------------------+
|                                 OS Input Stream                                   |
| (macOS CGEventTap / Linux evdev+uinput / Windows WH_MOUSE_LL)                     |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
|                              openlogi-hook Filter                                 |
|  source_is_remappable():                                                          |
|  - Pass-through trackpads (is_trackpad_like() == true)                            |
|  - Intercept Logitech & Generic USB Mice (allow_generic_mice config)             |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
|                            openlogi-agent-core                                    |
+-----------------------------------------------------------------------------------+
|  +-------------------------------------+  +------------------------------------+  |
|  |     HID++ Discovery Loop            |  |    Generic USB Mouse Scanner       |  |
|  |  (Bolt / Unifying / BT Direct)      |  |  (Usage Page 0x0001 / Usage 0x0002)|  |
|  +-------------------------------------+  +------------------------------------+  |
|                    |                                         |                    |
|                    v                                         v                    |
|  +-------------------------------------+  +------------------------------------+  |
|  |  DeviceKind::Mouse (HID++ Full)     |  |   DeviceKind::GenericMouse         |  |
|  |  - Hardware DPI                     |  |   - OS Button Remapping            |  |
|  |  - SmartShift / Ratchet Mode        |  |   - Smooth Scrolling Curves        |  |
|  |  - Onboard Lighting / Battery       |  |   - Actions Ring Overlay           |  |
|  |  - OS Button Remapping & Scrolling  |  |   - NO Hardware Sliders/Telemetry  |  |
|  +-------------------------------------+  +------------------------------------+  |
+-----------------------------------------------------------------------------------+
```

#### Step 1: Core Type Extensions (`openlogi-core`)

Extend `DeviceKind` and `DeviceCapabilities` in `crates/openlogi-core/src/device.rs`:

```rust
/// Classification of a discovered device.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    Mouse,
    Keyboard,
    Numpad,
    Trackball,
    Touchpad,
    Light,
    /// Standard USB / Bluetooth pointer device operating without HID++ protocols.
    GenericMouse,
    Unknown,
}

/// Generic USB mouse descriptor representation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericMouseDevice {
    pub address: RawDeviceAddress,
    pub display_name: String,
    pub manufacturer: Option<String>,
    pub serial_number: Option<String>,
    pub vendor_id: u16,
    pub product_id: u16,
    pub online: bool,
}
```

#### Step 2: Generic Device Scanning (`openlogi-device`)

Add a generic mouse scanner module in `crates/openlogi-device/src/inventory/generic.rs`:

```rust
use openlogi_core::device::{GenericMouseDevice, RawDeviceAddress};
use crate::backend::HidBackend;
use crate::inventory::InventoryError;

/// Enumerates connected non-HID++ USB / Bluetooth pointer devices.
pub async fn enumerate_generic_mice(
    backend: &dyn HidBackend,
) -> Result<Vec<GenericMouseDevice>, InventoryError> {
    let devices = backend.enumerate().await?;
    let generic_mice = devices
        .into_iter()
        .filter(|dev| {
            // Match standard HID Mouse usage (Generic Desktop 0x0001, Usage 0x0002)
            // or Pointer usage (Usage 0x0001)
            let is_mouse_usage = dev.usage_page == 0x0001 && (dev.usage_id == 0x0002 || dev.usage_id == 0x0001);

            // Exclude devices that are handled by HID++ or Litra drivers
            let is_hidpp = dev.vendor_id == 0x046d && (
                dev.usage_page == 0xff00 || dev.usage_page == 0xff43
            );

            is_mouse_usage && !is_hidpp
        })
        .map(|dev| {
            let identity = dev.identity();
            GenericMouseDevice {
                address: RawDeviceAddress {
                    vendor_id: dev.vendor_id,
                    product_id: dev.product_id,
                    usage_page: dev.usage_page,
                    usage_id: dev.usage_id,
                    identity,
                },
                display_name: if dev.name.trim().is_empty() {
                    format!("Generic Mouse ({:04x}:{:04x})", dev.vendor_id, dev.product_id)
                } else {
                    dev.name.clone()
                },
                manufacturer: dev.manufacturer.clone(),
                serial_number: dev.serial_number.clone(),
                vendor_id: dev.vendor_id,
                product_id: dev.product_id,
                online: true,
            }
        })
        .collect();

    Ok(generic_mice)
}
```

#### Step 3: Decouple OS Input Hook (`openlogi-hook`)

Update `source_is_remappable` in `crates/openlogi-hook/src/lib.rs` to allow generic mouse interception while preserving trackpad protection:

```rust
/// Configurable filter flag for generic mouse remapping.
pub struct HookConfig {
    pub allow_generic_mice: bool,
}

#[must_use]
pub fn source_is_remappable_with_config(
    device: Option<&EventDevice>,
    allow_generic_mice: bool,
) -> bool {
    match device {
        // CRITICAL GUARD: Trackpads and touchpads are ALWAYS passed through
        // to protect built-in laptop trackpads & Apple Magic Trackpads.
        Some(d) if d.is_trackpad_like() => false,
        Some(d) => {
            if d.is_logitech() {
                true
            } else {
                allow_generic_mice
            }
        }
        None => false,
    }
}
```

#### Step 4: Desktop UI Adjustments (`openlogi-desktop`)

In `openlogi-desktop`:
1. **Device Navigation Sidebar:** Render `GenericMouseDevice` entries with a distinct icon (e.g., standard wired mouse icon).
2. **Settings View Conditioning:**
   - **Show:** Button Remapping View (Middle click, Buttons 4 & 5, Side Buttons, Custom Shortcuts), Smooth Scroll / Acceleration View, Actions Ring Setup.
   - **Hide:** SmartShift Mode Selector, Hardware DPI Slider, RGB / Lighting Customization, Battery Level Indicator.

---

## Conclusion & Next Steps

By implementing this generic fallback architecture:
1. The **Logitech G90** will be seamlessly recognized as a wired USB mouse, enabling full OS button remapping, scroll acceleration, and the Actions Ring overlay.
2. OpenLogi will gain support for **any standard wired or wireless USB mouse**, broadening its utility as a local-first input customizer while protecting built-in trackpads and maintaining robust HID++ support for Logitech's flagship devices.
