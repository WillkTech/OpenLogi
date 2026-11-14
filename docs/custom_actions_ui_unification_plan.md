# Custom Actions — Unified Library, Everywhere-Available, Restorable-to-Off

## UI Investigation and Redevelopment Plan (Consolidated & Locked-In v4)

Status: **approved for phased execution**.
Supersedes and consolidates:

- [`shortcuts_power_user_ui_unification_plan.md`](shortcuts_power_user_ui_unification_plan.md) — parent plan (surface audit S1–S5, duplication D1–D6, consistency gaps C1–C7, data risks R1–R3, architecture §2, backlog §4).
- [`keys_tab_shortcuts_power_user_plan.md`](keys_tab_shortcuts_power_user_plan.md) — Keys-first plan (platform-gating finding §2.4, target-model options §5.2, "Target Key vs Type Text" overlap analysis §3).
- [`keys_tab_shortcuts_power_user_plan2.md`](keys_tab_shortcuts_power_user_plan2.md) — decision log for Q1–Q6.

This document adds two core requirements and five engineering refinements (R1–R5):

- **R-UNION** — a true *union* of Custom Shortcuts and the four Power User actions (Type Text / Run AppleScript / Run Shell Command / Workflow) into one concept — **Custom Actions** — creatable, editable, and assignable across all action-assignment surfaces (mouse buttons, Actions Ring, Keys tab, K270 Media keys).
- **R-OFF** — an explicit, scope-aware **Off / Reset to default / Clear slot** affordance on every surface, distinct from **"Do Nothing"** (`Action::None`).

---

## 1. Ground Truth & Verified Gaps

### 1.1 The Five Action-Assignment Surfaces

| # | Surface | List Builder | Search | + New | Edit Pencil | Delete X | Power User | Reset / Clear |
|---|---|---|---|---|---|---|---|---|
| S1 | Mouse popover picker (`features/mouse/inspector.rs` + `picker.rs`) | `action_rows_matching` | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| S2 | Mouse inline `action_library` (`inspector.rs` L616–677, button L650) | `action_rows_matching` | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| S3 | Actions Ring slot editor (`features/action_ring/editor.rs`) | own sections + inline `composer_section` | ❌ | ❌ | ✅ | own list | ❌ | ❌ **(no Clear Slot button)** |
| S4 | Keys tab panel (`features/keyboard/function_row.rs` `config_panel` L783) | `action_rows` (`on_edit: None`) | ❌ | ❌ | ❌ | ✅ unconditional | ✅ (only here) | ❌ **(cannot return to Off)** |
| S5 | K270 Media keys (`features/k270/view.rs`) | `action_rows_matching` | ✅ | ❌ | ❌ | ✅ unconditional | ❌ | ✅ **"Reset to default" exists** |

### 1.2 Five Newly Verified Refinements (R1–R5)

- **R1 — Two-Way Modifier Sync + Record Button (`ActionComposer`):** `ui/action_composer.rs` already implements `toggle_modifier_in_text` and `parse_modifiers_from_text`. Consolidating on `ActionComposer` must retain two-way sync between modifier chips (`Win`/`Cmd`/`Ctrl`/`Alt`/`Shift`/`Fn`) and the text box, paired with a **"Record Shortcut"** capture button and collapsible syntax legend (`Win+V`, `Fn+F11`).
- **R2 — Scope-Aware Reset Matrix:** Resetting a control must account for global vs. per-app profile scope (`editing_app == Some(app)`) and gesture maps (`ButtonId::GestureButton`), not just flat button maps.
- **R3 — Selected Checkmark on Custom Action Rows (`picker.rs` L275):** `picker.rs` gates the blue selection checkmark with `.when(selected && !is_custom, ...)`, hiding the checkmark whenever a `SavedShortcut` is the active binding. All selected rows must render `IconName::Check` before the row action controls.
- **R4 — Missing `StateEvent` Broadcast on Shortcut Save/Delete (`state/bindings.rs` L549–560):** `commit_custom_shortcut` and `delete_custom_shortcut` persist to disk without emitting a `StateEvent`. They must emit `StateEvent::CustomShortcutsChanged` (or `BindingsChanged` on cascade) so open views (`MouseModelView`, `FunctionRowView`, `K270MediaKeysView`, `ActionRingPanel`) repaint automatically.
- **R5 — Editable Workflow Step Rows Before Cross-Surface Exposure (`keyboard/editors.rs` L207–298):** Today `+ Add Step` hardcodes `WorkflowStep::TypeText(String::new())` and `WorkflowStepRow` is read-only preview + delete. Step-type selection (`TypeText` / `PressKey` / `Delay` / `AppleScript` / `Shell`) and inline payload editing must ship in **Phase 4** before exposing `.power_user(true)` on Mouse and K270.

---

## 2. Target UX & Scope-Aware Reset Specification

### 2.1 One User-Facing Concept: **Custom Actions**
- UI labels standardize on **"Custom Actions"** (`"+ New Custom Action"`, `"Edit Custom Action"`).
- On-disk TOML retains `[[custom_shortcuts]]` with `#[serde(alias = "custom_actions")]` for zero-migration backward compatibility in Phases 1–4.

### 2.2 Scope-Aware "Off / Reset to Default" Matrix (R-OFF + R2)

Every picker/inspector header uses the shared `ui::binding_header::control_status_header(...)` component showing the active binding, a state badge (`Default` / `Customized` / `App Override` / `Do Nothing`), and a contextual reset button:

| Surface / Control | Global Profile (`editing_app == None`) | Per-App Profile (`editing_app == Some(app)`) |
|---|---|---|
| **S1/S2 Mouse Standard Button** | `AppState::reset_binding(button)` → `Config::remove_binding(dev, button)` + set in-memory projection to `default_binding(button)`. Badge: `Default`. | Call existing `state.clear_app_binding(button)`. Badge: `Inherited`. |
| **S1/S2 Mouse Gesture Direction** | Reset single direction `dir` to `default_gesture_binding(dir)` (or reset all 5 via `"Reset Gesture Button"`). | Remove direction override from app profile gesture map. |
| **S3 Actions Ring Slot** | `AppState::clear_ring_slot(slot)` removes `slot` from `action_ring.slots`. Shows `Empty slot`. | Remove slot override from per-app ring layout. |
| **S4 Keyboard Function Row (`Keys` tab)** | Call `state.commit_keyboard_binding(trigger, None)`. Key returns to native OS/hardware pass-through (`Off`). | N/A (keyboard F-key bindings are global). |
| **S5 K270 Media Keys** | Call existing `commit_media_key(key, None, cx)` (`k270/view.rs` L396). | N/A (device-scoped). |

- **"Do Nothing" (`Action::None`)** remains in the catalog under `System` with an explicit tooltip: *"Captures the button/key and suppresses its output."*

---

## 3. Data Model & Architecture

### 3.1 Staged Union & Propagation Model
1. **Phases 1–4 (Zero Wire/Schema Bump):**
   - Keep `ActionTarget` as `Shortcut(KeyCombo) | Application(ApplicationTarget)`.
   - Anonymous inline Power User bindings (`Action::TypeText`, `RunAppleScript`, `RunShellCommand`, `Workflow`) remain valid and render in-place as `"Custom: <preview>"` with a **"Save to Custom Actions…"** promotion button once Phase 5 lands.
   - Centralize `Config::cascade_custom_shortcut_update(&entry)` and `Config::cascade_custom_shortcut_delete(id) -> usize` inside `openlogi-core/src/config.rs` so edits/deletes update or clear every embedded `Action::SavedShortcut` across device bindings, gesture maps, per-app profiles, `keyboard.bindings`, K270 `media_keys`, and `action_ring` slots atomically.
2. **Phase 5 (Schema Fork & Full Target Union):**
   - Extend `ActionTarget` with `Text(String)`, `AppleScript(String)`, `Shell(String)`, and `Workflow(Vec<WorkflowStep>)` (append-only wire change, `PROTOCOL_VERSION` bump + `wire_format` test, `SCHEMA_VERSION` migration).

### 3.2 Shared Desktop Components (`crates/openlogi-desktop/src/ui/`)
- **`ui/binding_header.rs`** — shared badge (`Default` / `Customized` / `Do Nothing`) + `"Reset to default"` / `"Clear slot"` button header.
- **`ui/action_library.rs`** — single `ActionLibrary` builder owning search input, `grouped_catalog` + capability-gated Power User section, `IconName::Check` on *all* selected rows (fixing R3), and edit/delete row affordances.
- **`ui/custom_action.rs`** — root-hosted `CustomActionModal` + extracted `PowerUserEditorState`, capability-filtered via:
  ```rust
  pub fn supported_power_user_kinds() -> &'static [PowerUserKind] {
      #[cfg(target_os = "macos")]
      {
          &[
              PowerUserKind::TypeText,
              PowerUserKind::RunAppleScript,
              PowerUserKind::RunShellCommand,
              PowerUserKind::Workflow,
          ]
      }
      #[cfg(not(target_os = "macos"))]
      {
          &[PowerUserKind::RunShellCommand, PowerUserKind::Workflow]
      }
  }
  ```

---

## 4. Phased Execution Plan

### Phase 0 — Characterization & Audit (S)

* Pin `action_rows_matching` affordance matrix and `wire_format` baseline in tests.

### Phase 1 — Keys-Tab & Ring Quick Wins + R-OFF (S, Zero Schema/Wire Change)

1. **Shared `binding_header` + Keys-Tab "Reset to default" & Ring "Clear slot":**
* Add `Default` / `Customized` badge and `"Reset to default"` button to `FunctionRowView::config_panel` (`keyboard/function_row.rs` L783), calling `commit_keyboard_binding(trigger, None)`.
* Add `"Clear slot"` button to `action_ring/editor.rs` when `current.is_some()`.

2. **Search Input on Keys Tab (S4):**
* Add `search: Entity<InputState>` to `FunctionRowView` and switch `panel_action_rows` from `action_rows` to `action_rows_matching`.

3. **Fix C2 (Destructive-Only `X`) & R3 (Missing Selected Checkmark):**
* In `picker.rs` L259, gate the delete `X` on `on_edit.is_some()` so S4/S5 do not show an unconfirmed delete button without edit support.
* In `picker.rs` L275, render `IconName::Check` on all selected rows (including `SavedShortcut`), positioned cleanly beside row action buttons.

4. **Fix R4 (`StateEvent` Broadcast):**
* Emit `StateEvent::CustomShortcutsChanged` from `commit_custom_shortcut` and `delete_custom_shortcut` (`state/bindings.rs` L549–560) and subscribe `MouseModelView`, `FunctionRowView`, `K270MediaKeysView`, and `ActionRingPanel`.

5. **Platform Capability Gate:**
* Gate `PowerUserKind` rows in `function_row.rs` (and step kinds in `editors.rs`) via `supported_power_user_kinds()`.

6. **i18n & Tooltips:**
* Wrap all hardcoded strings in `ui/shortcut_builder.rs`, `ui/action_composer.rs`, `keyboard/editors.rs`, and `function_row.rs` in `tr!(...)`; add keys at identical positions across all 20 `crates/openlogi-ui/locales/*.yml` files; add clarifying tooltip on `"Do Nothing"`.

### Phase 2 — Extract `ActionLibrary` (M)

* Extract `ui/action_library.rs`; migrate S4 → S2 → S1 → S5 → S3; deduplicate `action_icon_path`.

### Phase 3 — Root `CustomActionModal` Host + Cascade Propagation + Safe Delete (M)

* Move modal host to `AppState` / root view; wire `+ New Custom Action` and edit pencil into S3, S4, and S5.
* Implement `Config::cascade_custom_shortcut_update/delete` + usage count popover + 6-second Undo toast.

### Phase 4 — Editable Workflow Steps (R5) + Power User & Reset Everywhere (M) — (Completed)

* Upgrade `workflow_editor_card` with a step-type picker (`TypeText` / `PressKey` / `Delay` / `AppleScript` / `Shell`) and inline payload editing (R5).
* Extract `PowerUserEditorState` and enable `.power_user(true)` on Mouse (S1/S2) and K270 (S5).
* Add `AppState::reset_binding` + `Config::remove_binding` for mouse buttons and wire into S1/S2 headers (R2).

### Phase 5 — Full `ActionTarget` Union & Schema Migration (M–L) — (Completed)

* Extend `ActionTarget` with `Text`, `AppleScript`, `Shell`, and `Workflow`; bump `PROTOCOL_VERSION` (31) and `SCHEMA_VERSION` (8) with round-trip tests and golden wire assertions.
* Reusable `workflow_steps_list_and_adder` and inline power user promote button (`"Save to Custom Actions…"`) wired across Keys, Mouse, and K270.
* Multi-mode `ActionComposer` supporting all 6 target kinds with platform capability filtering.

---

## 5. Locked-In Decision Log (§7 Closed)

| # | Question | Locked-In Decision |
| --- | --- | --- |
| **Q1** | `SavedShortcut` propagation | Cascade sweep in `Config::save/delete_custom_shortcut` now (Phase 3); resolve-by-ID at schema fork (Phase 5). |
| **Q2** | Target model | Keep `ActionTarget` as `Shortcut | Application` in Phases 1–4; extend to all 6 kinds in Phase 5. |
| **Q3** | Power User surfaces | Enable on Keys (S4), Mouse (S1/S2), and K270 (S5) once R5 lands; omit raw inline editors from Actions Ring (S3). |
| **Q4** | Off-macOS `TypeText` / `AppleScript` | Gate via `supported_power_user_kinds()` immediately in Phase 1. |
| **Q5** | Delete UX | Conditional confirmation popover when `usage_count > 0` + 6-second Undo toast. |
| **Q6** | Ring composer | Replace raw inline `composer_section` with `ActionLibrary` + shared modal in Phase 3; keep two-way modifier sync + chord recorder in `ActionComposer` (R1). |
| **Q7** | UI Naming | **"Custom Actions"** in UI; `[[custom_shortcuts]]` (with `alias = "custom_actions"`) in TOML. |
| **Q8** | Phase 1.1 Reset scope | Ship Keys-tab `"Reset to default"` and Ring `"Clear slot"` immediately in Phase 1 via `ui/binding_header.rs`. |
| **Q9** | Legacy inline bindings | Render in-place as `"Custom: <preview>"` with explicit promote button; never auto-mutate on load. |
| **Q10** | Mouse reset semantics | Scope-aware `Config::remove_binding` + `clear_app_binding` per the §2.2 matrix (R2). |
