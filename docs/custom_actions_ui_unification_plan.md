# Custom Actions — Unified Library, Everywhere-Available, Restorable-to-Off

## UI Investigation and Redevelopment Plan (consolidated)

Status: **draft for discussion** (no code changed by this document).
Supersedes and consolidates:

- [`shortcuts_power_user_ui_unification_plan.md`](shortcuts_power_user_ui_unification_plan.md) — the "parent plan" (surface audit S1–S5, duplication D1–D6, consistency gaps C1–C7, data risks R1–R3, architecture §2, backlog §4). Its findings were re-verified against the current tree and **all remain accurate**; they are adopted here with their original IDs.
- [`keys_tab_shortcuts_power_user_plan.md`](keys_tab_shortcuts_power_user_plan.md) — the "Keys-first plan" (platform-gating finding §2.4, target-model options §5.2, "Target Key vs Type Text" overlap analysis §3). Adopted.
- [`keys_tab_shortcuts_power_user_plan2.md`](keys_tab_shortcuts_power_user_plan2.md) — answers to the open questions Q1–Q6 (agent resolution audit, staged data-integrity strategy, capability gating, delete UX, ring composer decision). Adopted as the "decision log" in §7.

This document adds two requirements the earlier plans did not cover:

- **R-UNION** — a true *union* of Custom Shortcuts and the four Power User
  actions (Type Text / Run AppleScript / Run Shell Command / Workflow) into
  one concept — **Custom Actions** — creatable, editable, and assignable from
  every action-assignment surface (mouse buttons, Actions Ring, Keys tab,
  K270 Media keys).
- **R-OFF** — an explicit **Off / Reset to default** affordance on every
  surface, distinct from the existing **"Do Nothing"** (`Action::None`).
  Today a function key bound to any action under the Keys tab **cannot be
  restored to pass-through** from the GUI at all.

All line numbers are anchors at the time of writing, not guarantees.

---

## 1. Ground truth (re-verified)

### 1.1 The five action-assignment surfaces (parent §1.1, unchanged)

| # | Surface | List builder | Search | + New | Edit pencil | Delete X | Power User | Reset-to-default |
|---|---|---|---|---|---|---|---|---|
| S1 | Mouse popover picker (`features/mouse/inspector.rs` + `picker.rs`) | `action_rows_matching` | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| S2 | Mouse inline `action_library` (`inspector.rs` L616–677, button at L650) | `action_rows_matching` | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| S3 | Actions Ring slot editor (`features/action_ring/editor.rs`) | own sections + inline `composer_section` | ❌ | ❌ | ✅ | own list | ❌ | ? (audit) |
| S4 | Keys tab panel (`features/keyboard/function_row.rs` `config_panel` L783, `panel_action_rows` L852) | `action_rows` (`on_edit: None`) | ❌ | ❌ | ❌ | ✅ unconditional | ✅ (only here) | ❌ **(user-visible bug-class)** |
| S5 | K270 Media keys (`features/k270/view.rs`) | `action_rows_matching` | ✅ | ❌ | ❌ | ✅ unconditional | ❌ | ✅ **"Reset to default" exists** |

### 1.2 The silos and redundancies (parent §1.2–§1.4, adopted)

- **Custom Shortcuts** (`ui/shortcut_builder.rs::ShortcutBuilderModal` L28 +
  `ui/action_composer.rs::ActionComposer` L93): create/edit UI exists only on
  S1/S2 (and edit-only on S3). Host wiring copy-pasted between
  `mouse/view.rs::open_shortcut_builder_with` L177–232 and
  `action_ring.rs` L61–115 (**D3**). Storage is global
  `config.custom_shortcuts: Vec<CustomShortcutEntry>`; bindings embed a
  **denormalized copy** (`Action::SavedShortcut { id, name, target, icon }`).
- **Power User** (S4 only): four hardcoded rows under `tr!("Power User")`
  (`function_row.rs` L861–882); inline editors in `keyboard/editors.rs`
  (`PowerUserKind` L38, `editor_card` L82, `text_editor_card` L103,
  `workflow_editor_card` L179). Draft state (`active_editor`, `text_state`,
  `workflow_draft`) welded to `FunctionRowView` L119–133 (**D6**). These
  variants are `not_pickable` in the core catalog, so no other surface can
  even list them.
- Duplication **D1/D2/D5** (five chrome assemblies around one row builder;
  two `action_icon_path` implementations) and gaps **C1–C7** (create-only-on-
  mouse, destructive-without-constructive X on S4/S5, no delete confirm,
  missing search on S3/S4, i18n drift, panel-width drift, global-vs-per-device
  scope mismatch) all confirmed as written in the parent plan.

### 1.3 The union blocker (Keys-first plan §3, §5.2 — verified)

The two systems cannot be merged by UI alone because their **payload models
are disjoint**:

- `ActionTarget` (`openlogi-core/src/binding/action_target.rs` L10–15) is
  `Shortcut(KeyCombo) | Application(ApplicationTarget)` only — a saved
  shortcut can never carry text or a script.
- Power User actions are **anonymous inline bindings**
  (`Action::TypeText("hi")` stored directly on the one key) — they can never
  be a named, reusable, cross-device entry, and they are excluded from the
  catalog so no other surface can assign them.

So "make the union available across the board" requires a **data-model
decision** (§3 below), not just a shared component.

### 1.4 Platform capability gap (Keys-first plan §2.4 — verified)

| Kind | macOS | Linux | Windows | UI gates it today? |
|---|---|---|---|---|
| Type Text | ✅ `post_unicode` (`inject/macos.rs`) | ❌ warn + no-op (`inject/linux.rs`) | ❌ warn + no-op (`inject/windows.rs`) | **No** |
| Run AppleScript | ✅ | ❌ | ❌ | **No** |
| Run Shell Command | ✅ | ✅ | ✅ | n/a |
| Workflow | ✅ | ✅ (AppleScript steps warn) | ✅ | **No** |

Any "everywhere" exposure must be capability-filtered, not just
`cfg(target_os = "macos")`-gating AppleScript.

### 1.5 Data-integrity risks R1–R3 (parent §1.5 + decision-log audit)

R1 (edit does not propagate), R2 (delete orphans bindings), R3 (denormalized
copy drifts) all confirmed: `state/bindings.rs::commit_custom_shortcut` L549 /
`delete_custom_shortcut` L556 → `config.rs::save_custom_shortcut` L1077 /
`delete_custom_shortcut` L1090 — no reference sweep anywhere.
**Resolved fact** (plan2 audit): the agent lowers `Action::SavedShortcut`
from its **embedded** `target` (`effect.rs` L278–282) — it never re-resolves
`id` against config. Exposing create/edit/delete everywhere multiplies the
blast radius of R1/R2, so the data-integrity phase must not lag the
affordance work by a full release.

### 1.6 "Off" vs "Do Nothing" — verified semantics (new finding, R-OFF)

The codebase *already models* "Off" as **absence of a binding**; only the GUI
fails to expose it consistently. `Action::None` is labeled **"Do Nothing"**
(`action.rs` L284, System category, in the catalog): the event is captured
and swallowed. The two are different features:

| Surface | "Off" (= native pass-through) in the model | How config stores it | GUI affordance today |
|---|---|---|---|
| Mouse button | `capture_plan` diverts a button only when its binding **differs from** `default_binding` (`agent-core/src/capture_plan.rs` L93–95) | absent entry, or an entry equal to the default | **None** — `commit_binding(button, Action)` (`state/bindings.rs` L203) always inserts; `Config::set_binding` L226 takes no `Option`; `default_binding` is never referenced by the GUI |
| Keyboard F-key | absent entry in global `config.keyboard.bindings` → key never diverted | `set_keyboard_binding(trigger, None)` removes (`config.rs` L238) | **None** — `commit_keyboard_binding(_, None)` exists (`state/bindings.rs` L336–350) but `panel_action_rows`' `on_pick` only ever commits `Some(action)`; picking the catalog row "Do Nothing" commits `Some(Action::None)` — a *swallow* binding, not Off. **This is the reported bug.** |
| K270 media key | absent entry → `key.default_action()` (`effective_k270_media_key_action`) | `set_media_key_binding(_, None)` | ✅ **"Reset to default"** button + Default/Customized badge (`k270/view.rs` L352–398, commits `None` at L396) — the pattern to generalize |
| Actions Ring slot | **empty slot = absent map entry**; `Action::None` is explicitly rejected as `RingActionError::EmptyAction` ("Do Nothing is represented by an empty Actions Ring slot", `core/src/binding/action_ring.rs` L105–112) | absent slot entry | audit the editor for a "Clear slot" affordance (parent marked it "?") |
| App-profile override | falls back to global binding | `clear_app_binding(button)` (`state/tests.rs` L489+) | ✅ exists |

Conclusion: R-OFF needs (a) one state-layer addition for mice
(`reset_binding` → remove entry), (b) GUI affordances on S1/S2/S3/S4, and
(c) a label/terminology pass so "Off (default)" and "Do Nothing" are never
confused.

---

## 2. Target UX

### 2.1 One concept: **Custom Actions**

Rename the user-facing concept from "Custom Shortcuts" to **Custom Actions**
(TOML key `custom_shortcuts` stays for compatibility; see §3.3). One library,
one row style, one editor, N kinds:

| Kind | Payload | Today lives as |
|---|---|---|
| Keyboard shortcut | `KeyCombo` | `CustomShortcutEntry` + `ActionTarget::Shortcut` |
| Open app / folder / URL | `ApplicationTarget` | `CustomShortcutEntry` + `ActionTarget::Application` |
| Type text | `String` | anonymous `Action::TypeText` (S4 only) |
| Run AppleScript | `String` | anonymous `Action::RunAppleScript` (S4 only) |
| Run shell command | `String` | anonymous `Action::RunShellCommand` (S4 only) |
| Workflow | `Vec<WorkflowStep>` | anonymous `Action::Workflow` (S4 only) |

Every surface (S1–S5) gets: search, the Custom Actions section (create ✅,
edit ✏️, delete 🗑 with confirm + undo), capability-filtered kinds, and
**Off / Reset to default**. Kinds unsupported on the current OS are hidden
(one `capabilities()` function, §4.3), never shown-but-no-op.

### 2.2 Off vs Do Nothing in the UI

- Every picker/panel header shows the **current binding** with a state badge
  (`Default` / `Customized` / `Do Nothing`) — the K270 pattern
  (`k270/view.rs` L352–388) promoted to a shared component.
- A **Reset to default** button (ghost, disabled while already default)
  commits the *absence* of a binding. Tooltips state the effect concretely:
  "Restores F5 to its system behavior."
- "Do Nothing" stays in the catalog as a distinct choice with a clarifying
  tooltip: "The button/key is captured and does nothing at all."
- Ring slots get **Clear slot** (removes the entry — matches
  `EmptyAction` semantics); mouse buttons get reset via the new
  `reset_binding` (§4.1).

### 2.3 Single editor for all kinds

`ShortcutBuilderModal` becomes **`CustomActionModal`**: a kind selector
(segmented; capability-filtered) switching between the existing
`ActionComposer` (chord/app modes) and the Power User editor bodies
(text field per kind / workflow step list). One name field, one icon grid,
one Save — killing D4 (modal composer vs ring inline `composer_section`) and
D6 (welded draft state) in one component.

---

## 3. Data model decisions

### 3.1 Union representation (new decision, extends parent §2.4 / Keys-first §5.2)

| Option | Description | Cost / risk |
|---|---|---|
| **A. UI-level union only** | Keep `CustomShortcutEntry` (chord/app) and inline anonymous Power User bindings as two stores; one merged "Custom Actions" *section* renders both. | Zero schema change; but text/script actions remain **unreusable** (one key only), the C7 management gap stays, and every editor still has two code paths. Does not satisfy R-UNION. |
| **B. Extend `ActionTarget` to the full kind set** (recommended) | Append `Text(String)`, `AppleScript(String)`, `Shell(String)`, `Workflow(Vec<WorkflowStep>)` to `ActionTarget`. Library entries carry any target; picking an entry commits `Action::SavedShortcut { id, … }` from *any* surface; the agent lowers the new targets through the existing `Effect::Text` / `Effect::Script` paths (`effect.rs`). | Append-only enum: `PROTOCOL_VERSION` bump + `wire_format` test (ipc rule); `SCHEMA_VERSION` note for hand-edited configs; old agents must gracefully ignore unknown targets; inherits §1.4 capability gap for *saved* text targets too (gate creation, keep existing entries renderable). |
| C. Library entry stores a full `Action` | `CustomShortcutEntry { id, name, icon, action: Action }` — maximally flexible. | Recursive references possible (entry → SavedShortcut), every consumer of `ActionTarget` must special-case entries; heavier than needed. Reject unless a future kind outgrows B. |

**Staging (from the decision log, plan2 Q1/Q2):** Phases 1–4 proceed under
the current shape; Phase 5 lands option B together with the R1–R3 fix.
Until B lands, Power User picks keep committing inline anonymous actions
(unchanged behavior) but gain a **"Save as Custom Action…"** promotion button
so the library becomes the only place to build a *reusable* macro-ish action.

### 3.2 R1–R3 fix: staged propagate-on-write → resolve-by-id

- **Now (Phase 5a, no wire change):** centralize a reference sweep in core —
  `Config::cascade_custom_shortcut_update(&entry)` and
  `Config::cascade_custom_shortcut_delete(id) -> usize` — rewriting or
  clearing every embedded `Action::SavedShortcut` across device bindings,
  gesture maps, per-app profiles, `keyboard.bindings`, K270 `media_keys`, and
  ring slots, atomically on save/delete. Feeds the delete-confirm popover
  ("Used by N controls") and the usage count.
- **At the schema fork (Phase 5b):** bindings keep `id` (+ display-name
  snapshot), GUI and agent resolve name/target/icon from
  `config.custom_shortcuts` at use time; deleted-id fallback = treat as
  unbound (Off) + warning; `PROTOCOL_VERSION` bump + migration chain.

### 3.3 Migration & compatibility

- TOML: `[[custom_shortcuts]]` key and entry shape unchanged in Phases 1–4.
  Phase 5a sweeps rewrite embedded copies in `[keyboard.bindings]`,
  `[devices.*.bindings]`, ring slots, etc. — round-trip tests per section.
- Phase 5b: `SCHEMA_VERSION` bump + migration; `wire_format` snapshot;
  `docs/CONFIGURATION.md` + `docs/config.example.toml` updates.
- Existing anonymous `Action::TypeText`-style bindings **stay valid** (the
  variants remain); the GUI renders them as "Unsaved custom action" rows with
  an offer to promote into the library.

---

## 4. Shared components (kills D1–D6)

### 4.1 State layer (`openlogi-desktop/src/state`)

```text
AppState::open_custom_action_editor(existing: Option<CustomActionEntry>)
AppState::close_custom_action_editor()          // modal renders once at app root
AppState::reset_binding(button)                 // NEW: removes the config entry → native
AppState::reset_keyboard_binding(trigger)      // thin wrapper over commit_keyboard_binding(_, None)
AppState::clear_ring_slot(slot)                 // if the audit finds it missing
AppState::custom_action_usage(id) -> UsageSummary // read-only sweep; feeds confirm + "Used by"
```

`Config::remove_binding(device_key, button)` is the only core addition for
reset (config map is already sparse; `capture_plan` already treats absence as
native). Delete the two copy-pasted modal wirings (D3).

### 4.2 `ui/action_library.rs` (parent §2.1)

One component owning: section grouping (catalog + Custom Actions), search,
row rendering, per-row affordances (edit/delete gated on capability flags),
empty-state CTA, and the Off/Reset header block. Surfaces S1–S5 become thin
adapters supplying the commit closure (`commit_binding` /
`commit_keyboard_binding` / `commit_k270_media_key` / ring slot setter) and
the chrome (popover vs inline card vs side panel). `action_icon_path` dedups
here (D5) — into `openlogi-ui` only if the overlay needs it (gui rule: the
overlay links `openlogi-ui`, never `openlogi-desktop`).

### 4.3 `ui/custom_action.rs` (parent §2.3 + R-UNION)

`CustomActionModal` (§2.3) + `CustomActionEditorState` extracted from
`FunctionRowView` (D6). One capability function:

```rust
fn supported_kinds() -> &'static [ActionKind] {
    #[cfg(target_os = "macos")] { ALL }
    #[cfg(not(target_os = "macos"))] { &[Shortcut, Application, Shell, Workflow] }
    // + TypeText as soon as openlogi-inject implements unicode synthesis
}
```

Cross-lint Linux/Windows per `.claude/rules/cross-platform.md` —
macOS-green proves nothing in cfg-gated code.

### 4.4 Ring editor (decision-log Q6)

Drop the ring's inline `composer_section` + duplicate `icon_editor`; the ring
slot editor uses `ActionLibrary` + opens the shared `CustomActionModal`.
Power User kinds stay **out of ring slots** (decision-log Q3): radial icons
suit atomic actions; wrap a script in a named Custom Action and… that is
exactly what the library is for — revisit after Phase 5 if users ask.

---

## 5. Phased execution (Keys-first, union-terminating)

Each phase independently shippable; sizes S/M/L relative. Verification per
phase: affected-package tier of the local gate (`AGENTS.md`), widened with
`cargo tree --target all --invert` since `openlogi-desktop`/`openlogi-core`
fan out widely.

### Phase 0 — Characterization & audit (S)
- Tests pinning: affordance matrix of `action_rows_matching`; a test
  documenting R1 non-propagation (failing-today, fixed in 5a); `wire_format`
  baseline.
- Audit: does the ring editor offer slot clearing (§1.6)? How does the GUI
  render a mouse button whose config entry is absent (expect
  `default_binding_for` fallback)?
- Inventory hardcoded strings (`shortcut_builder.rs`, `action_composer.rs`,
  `editors.rs`, `panel_action_rows`) vs `locales/en.yml`.
- **Gate:** tests only; no behavior change.

### Phase 1 — Keys-tab quick wins + R-OFF on Keys (S, no architecture change)
1. **Reset to default on the Keys panel** — header button (enabled when the
   key has a binding) calling `commit_keyboard_binding(trigger, None)`, plus
   the Default/Customized badge. *This is the user-reported dead-end; it is
   shippable today with zero model change.*
2. Search input in `config_panel` (the list already filters on `query`).
3. Hide the delete X on `SavedShortcut` rows when `on_edit` is `None`
   (kills C2 safely on S4/S5 until edit lands).
4. Capability-gate Power User rows (macOS: all four; Linux/Windows: Shell +
   Workflow) behind one helper.
5. i18n wrap of every string in the §1.2/§2.3 files; keys in **all**
   `locales/*.yml` at the same position; standardize `…`;
   `cargo test -p openlogi-desktop i18n`.
6. Tooltips distinguishing "Do Nothing" from Off on every catalog row set.
- **Acceptance:** a keyboard-only user can bind *and unbind* any F-key; no
  destructive-only affordance; no hardcoded English in touched files; no
  no-op rows bindable on Linux/Windows.

### Phase 2 — Extract `ActionLibrary` (M)
Build §4.2 from the mouse inline variant; migrate **S4 first**, then
S2 → S1 → S5 → S3. Acceptance: `rg "fn action_rows"` and
`rg "fn action_icon_path"` each find exactly one definition.

### Phase 3 — `CustomActionModal` host + create/edit everywhere (M)
§4.1 host service; delete both per-view wirings; wire **+ New Custom
Action** and the edit pencil into S3/S4/S5 headers; pencil + X appear
together (Phase 1.3 retires). Delete-confirm popover with "Used by N
controls" (read-only sweep). Acceptance: one open/save/close wiring; a
keyboard-only user can create, edit, assign, and delete a shortcut.

### Phase 4 — Power User everywhere + R-OFF everywhere (M)
- Extract `CustomActionEditorState` (§4.3); Power User kinds appear on S1/S2
  and S5 behind `capabilities()` (decision-log Q3: yes for mouse/K270, no for
  ring).
- Mouse: `AppState::reset_binding` + `Config::remove_binding` + Reset button
  in the button inspector when the binding differs from `default_binding`;
  ring: **Clear slot** (per Phase 0 audit).
- Per-kind "Save as Custom Action…" promotion button on inline editors.
- Cross-lint Linux/Windows for all new cfg blocks.

### Phase 5 — Union + data integrity (M–L; needs §3 decisions)
- **5a (no wire change):** `Config::cascade_custom_shortcut_update/delete`
  sweeps + usage helper wired into delete confirm and undo toast.
- **5b (schema fork):** `ActionTarget` extension (option B, §3.1) → the
  library becomes the single home for all six kinds; picking a Text/Script/
  Workflow entry from any surface commits `SavedShortcut { id, … }`;
  resolve-by-id per decision-log Q1; `PROTOCOL_VERSION` bump +
  `cargo test -p openlogi-ipc --test wire_format`; `SCHEMA_VERSION`
  migration; stale-entry ("renamed/removed action") row state;
  `docs/CONFIGURATION.md` updates.
- If saved Text targets ship: Linux/Windows unicode injection in
  `openlogi-inject` is a hard prerequisite, or the kind stays capability-
  hidden there (§1.4, decision-log Q4).

### Phase 6 — Backlog (parent §4, trimmed; small PRs)
P1 chord recorder in the modal (replaces modifier toggles + typed chord; also
resolves the "Target Key" vs "Type Text" label confusion); P2 per-step
workflow editing + drag-reorder; P2 undo toast after delete; P2 kebab row
menu (Edit/Duplicate/Delete); P2 "Test / Try it" button (confirm dialog for
script kinds); P3 monospace growing textarea for script kinds; P3 empty-state
CTA; P3 dedicated **Custom Actions** sidebar page with "Used by" column
(parent C7/P1 — now that entries exist everywhere, management needs a home);
P3 duplicate-name warning; P3 accessibility pass (aria labels on pencil/X/
kebab/reset, search labelling).

---

## 6. Consistency checklist (definition of done)

- One row builder, one icon-path fn, one modal host, one editor-state
  machine, one capability function, one reset pattern (badge + button).
- S1–S5 all offer: search, list, create, edit, delete-with-confirm+undo,
  Off/Reset, capability-filtered kinds — or a deliberate, documented
  exception (ring: no Power User kinds).
- "Do Nothing" and "Off (default)" distinct everywhere, each with a tooltip.
- All strings through `tr!`; keys identical across locales; `…` unified.
- Panels share width tokens and Cancel/Save styling (kills C6).

## 7. Decision log & remaining open questions

Adopted from plan2 (re-verify before Phase 5):

| Q | Decision |
|---|---|
| Q1 `SavedShortcut` model | Staged: propagate-on-write sweep now (5a), resolve-by-id at the schema fork (5b). Agent confirmed to trust the embedded copy (`effect.rs` L278–282). |
| Q2 Target model | Full-kind `ActionTarget` (B) deferred to 5b; promotion path + chord recorder now. |
| Q3 Power User on mice/ring/K270 | Mouse ✅, K270 ✅, Ring ❌ (named-entry workaround via library). |
| Q4 Linux/Windows TypeText | Capability-hide now; implement `openlogi-inject` unicode synthesis to unlock. |
| Q5 Delete UX | Confirm popover when usage > 0, always-on undo toast. |
| Q6 Ring composer | Drop inline `composer_section`; shared modal. |

New questions for the maintainer:

1. **Naming:** adopt "Custom Actions" as the UI term (TOML key unchanged)?
2. **Phase 1.1 scope:** ship the Keys-tab Reset before the shared components
   land (fast fix, one more pre-refactor copy of the badge/button pattern),
   or fold it into Phase 2's shared header block?
3. **Anonymous legacy bindings:** render as "Unsaved custom action" with a
   promote offer (proposed), or auto-migrate into the library at load?
4. **Mouse reset semantics:** remove the config entry (proposed; native tilt
   preserved per `capture_plan`) vs commit `default_binding` explicitly?
5. **Ring slot clearing:** confirm the editor gap in Phase 0 and add
   "Clear slot" in Phase 4?

## 8. Constraints from the project rules (non-negotiables, inherited)

- **IPC** (`.claude/rules/ipc-protocol.md`): `Action`, `ActionTarget`,
  `CustomShortcutEntry` ride the wire — Phase 5 = append-only +
  `PROTOCOL_VERSION` bump + `wire_format` green.
- **i18n** (`.claude/rules/i18n.md`): new keys in **all** `locales/*.yml` at
  the same position; `cargo test -p openlogi-desktop i18n`.
- **GUI** (`.claude/rules/gui.md`): anything the overlay needs goes to
  `openlogi-ui`; the overlay never links `openlogi-desktop`.
- **Cross-platform** (`.claude/rules/cross-platform.md`): capability gating
  and every new cfg block → cross-lint or hand-audit Linux/Windows.
- **Verification:** affected-package tier per phase; Phase 5 adds the IPC
  tests, config-migration round-trip tests, and hardware-verification notes
  ("not runtime-tested on hardware" where true).
- **Docs:** `docs/CONFIGURATION.md` + `docs/config.example.toml` updates
  accompany any Phase 5 shape change.
