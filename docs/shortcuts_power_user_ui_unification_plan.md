> Superseded by [custom_actions_ui_unification_plan.md](custom_actions_ui_unification_plan.md)

# Custom Shortcuts & Power User — UI Investigation and Redevelopment Plan

Status: **draft for discussion** (no code changed by this document).
Scope: the action-assignment surfaces of `openlogi-desktop` — Custom Shortcuts
(`SavedShortcut` / `ShortcutBuilderModal`) and the Power User section
(Type Text / Run AppleScript / Run Shell Command / Workflow).
Goal: one consistent, non-duplicated way to browse, create, edit, and assign
these actions from **every** picker (mouse buttons, Actions Ring, keyboard
function row, K270 media keys), plus a UX improvement backlog.

All line numbers reference the tree at the time of writing; treat them as
anchors, not guarantees.

---

## 1. Ground truth: what exists today

### 1.1 The action-assignment surfaces

| # | Surface | File | Action list | Search | See custom shortcuts | **+ New** | **Edit** | **Delete** | Power User |
|---|---|---|---|---|---|---|---|---|---|
| S1 | Mouse button popover picker | `features/mouse/inspector.rs` + `features/mouse/picker.rs` | `action_rows_matching` | ✅ | ✅ (`Category::CustomShortcuts`) | ✅ | ✅ (pencil) | ✅ (X) | ❌ |
| S2 | Mouse inspector inline `action_library` | `features/mouse/inspector.rs` L616–677 | `action_rows_matching` | ✅ | ✅ | ✅ (`+ New Shortcut`, L650) | ✅ | ✅ | ❌ |
| S3 | Actions Ring slot editor | `features/action_ring/editor.rs` | **own** `action_sections` + `icon_editor` + inline `composer_section` | ❌ | ✅ | ❌ (edit only) | ✅ | ? (own list) | ❌ |
| S4 | Keyboard function-row panel (Keys tab) | `features/keyboard/function_row.rs` `config_panel` L783 + `panel_action_rows` L852 | `action_rows` (`on_edit = None`) | ❌ | ✅ | ❌ | ❌ | ✅ (unconditional X) | ✅ (4 kinds, inline editors) |
| S5 | K270 Media keys tab | `features/k270/view.rs` | `action_rows_matching` | ✅ | ✅ | ❌ | ❌ | ✅ (unconditional X) | ❌ |

Read-only consumers: the overlay Actions Ring renders `SavedShortcut` icons
(`ActionRingIcon::asset_path` via `picker::action_icon_path`, mirrored in
`action_ring/action_icons.rs`).

### 1.2 The two feature silos

**Custom Shortcuts** (create/edit UI exists only beside mouse buttons):

- Modal: `ui/shortcut_builder.rs` — `ShortcutBuilderModal` (name input +
  embedded `ui/action_composer.rs::ActionComposer`; Save disabled until valid;
  ids generated as `custom_<millis>`).
- Host wiring is **copy-pasted verbatim** into two views:
  `mouse/view.rs` `open_shortcut_builder_with` L176–232 and
  `action_ring.rs` L61–115 (build modal → `on_save` →
  `AppState::commit_custom_shortcut` → close; identical `on_cancel`).
- Storage: global `config.custom_shortcuts: Vec<CustomShortcutEntry>`
  (`openlogi-core/src/config.rs` L49–59, L1072–1094) — *not* per-device.
- Bindings reference a shortcut by embedding a **denormalized copy**:
  `Action::SavedShortcut { id, name, target, icon }`
  (`openlogi-core/src/binding/action.rs` L196–205).
- The catalog injects saved entries as a first section
  (`picker.rs::grouped_catalog` L33–64), so *every* surface lists them —
  but only S1/S2/S3 can edit; S4/S5 can only delete (see C2).

**Power User** (exists only in the keyboard panel):

- Rows: `keyboard/function_row.rs::panel_action_rows` L861–932 — four hardcoded
  labels under `tr!("Power User")`; click → `FunctionRowView::open_editor(kind)`.
- Editors: `keyboard/editors.rs` — `PowerUserKind` L38, `editor_card` L82,
  shared `text_editor_card` L103 (one field; commit wraps the text as
  `Action::TypeText` / `RunAppleScript` / `RunShellCommand`),
  `workflow_editor_card` L179 (`+ Add Step` / `Save Workflow`;
  `WorkflowStepRow` is preview + remove only — no per-step edit or reorder).
- Draft state (`active_editor`, `text_state`, `workflow_draft`) is
  hand-rolled on `FunctionRowView` L119–133, L188–239 — not reusable.
- These actions are `not_pickable` in the core catalog
  (`power_user_actions_excluded_from_catalog`, `binding/tests.rs`), so mouse,
  ring, and K270 **cannot assign them at all**.

### 1.3 Duplication inventory

| ID | Duplication | Where |
|---|---|---|
| D1 | Action-list chrome assembled 5 different ways around (mostly) the same row builder | S1–S5 above |
| D2 | `action_library` header + search + rows + edit-callback | `mouse/inspector.rs` L616 vs `action_ring/editor.rs` L33 |
| D3 | Shortcut-builder open/save/close host wiring | `mouse/view.rs` L176–232 vs `action_ring.rs` L61–115 |
| D4 | Two UXs for defining the same `ActionTarget`: modal `ActionComposer` vs ring's inline `composer_section` + `toggle_modifier_in_text` | `ui/action_composer.rs` vs `action_ring/editor.rs` L109 |
| D5 | Two `action_icon_path` implementations | `mouse/picker.rs` L71 vs `action_ring/action_icons.rs` |
| D6 | Power-user draft-state machine welded to `FunctionRowView` | `keyboard/function_row.rs` |

### 1.4 Consistency gaps

| ID | Gap |
|---|---|
| C1 | `+ New Shortcut` exists only on mouse surfaces; ring can edit but not create; keyboard/K270 can neither. A user discovers the feature exists only by owning a mouse. |
| C2 | On keyboard/K270, custom-shortcut rows show a **delete X but no edit pencil** (`on_edit: None`, but the X is rendered unconditionally in `action_rows_matching` L233–273). Destructive affordance without the constructive one. |
| C3 | Delete is instant and unconfirmed on every surface; no reference check, no undo. |
| C4 | Search exists on S1/S2/S5 but not S3/S4. |
| C5 | i18n drift: `ShortcutBuilderModal` strings are hardcoded English ("New Custom Action", "Edit Custom Action", "Name", "Cancel", "Save Action" — `shortcut_builder.rs` L110–166) and Power-User row labels/headings are hardcoded (`(*label).to_string()`, `PowerUserKind::heading`), even though keys like `"Type Text…"`, `"Save Workflow"` already ship in every locale (`locales/en.yml` L412–418). Ellipsis style also drifts (`"…"` in rows vs bare headings). |
| C6 | Panel widths, header patterns, and Cancel/Save button styling differ per surface (`compact_panel` w=300 vs 320 vs `PANEL_W`, ring's own card). |
| C7 | Conceptual mismatch: shortcuts are **global** config but the create button is buried in a **per-device** mouse panel. There is no place to manage the library as a whole (rename, see where used). |

### 1.5 Data-integrity risks (investigate before/while rebuilding)

| ID | Risk | Evidence |
|---|---|---|
| R1 | **Edit does not propagate.** `commit_custom_shortcut` (`state/bindings.rs` L549) saves the entry but never touches existing `SavedShortcut` bindings, which carry their own frozen `name`/`target`/`icon`. Editing "Cmd+Shift+P → Cmd+Shift+B" silently leaves every already-assigned button firing the old chord. | `config.rs::save_custom_shortcut`, no reference sweep |
| R2 | **Delete orphans bindings.** `delete_custom_shortcut` (L556) drops the entry; bindings embedding that `id` keep working as zombie frozen copies (or misbehave — agent-side resolution needs an audit). | `config.rs::delete_custom_shortcut` |
| R3 | Denormalized `SavedShortcut` will drift further as fields grow (e.g. future Tap/Hold variants). Every feature must decide twice: on the entry and on every binding copy. | `action.rs` L196 |

**To verify during Phase 0:** how the agent resolves `Action::SavedShortcut`
at runtime (uses the embedded copy vs. re-resolves `id` against config). The
answer decides §2.4.

---

## 2. Target architecture

### 2.1 One `ActionLibrary` component (kills D1–D2, D5)

New shared module, e.g. `crates/openlogi-desktop/src/ui/action_library.rs`:

```text
ActionLibrary::new(...)
    .search(bool)              // show filter input
    .custom_shortcuts(Flags { create, edit, delete })
    .power_user(bool)          // show Power User section (platform-filtered)
    .commit(Arc<dyn Fn(Action, ...)> )   // where a pick lands (device vs keyboard vs ring slot)
```

- Owns: section grouping (`grouped_catalog` + Power User), search state,
  row rendering (single `action_rows` implementation), row affordances
  (pencil/X per flags), empty states.
- The five surfaces become thin adapters that supply the commit closure and
  the chrome (popover vs inline card vs side panel).
- `action_icon_path` is deduplicated into the same module (or `openlogi-ui`
  if the overlay needs it — remember the overlay links `openlogi-ui`, not
  `openlogi-desktop`).

### 2.2 App-level `ShortcutBuilder` host (kills D3, enables C1 fix everywhere)

Move modal ownership out of `MouseModelView` / `ActionRingPanel` into a small
service on `AppState` (or a dedicated root-level view):

```text
AppState::open_shortcut_builder(existing: Option<CustomShortcutEntry>, ...)
AppState::close_shortcut_builder()
```

- One place wires `on_save` → `commit_custom_shortcut` (+ propagation, §2.4).
- Any surface — keyboard panel, K270, ring — gets "create/edit" for free by
  calling the service; the modal renders once at the app root overlay layer.

### 2.3 Shared Power-User editor controller (kills D6, enables Power User everywhere)

Extract `active_editor` / `text_state` / `workflow_draft` + open/close/seed
into a `PowerUserEditorState` struct usable by any host view, and move
`editors.rs` cards to `ui/power_user.rs` so the mouse inspector and ring can
render the same inline editor. Platform filter: `RunAppleScript` rows are
`cfg(target_os = "macos")`-gated (see `.claude/rules/cross-platform.md`);
Linux/Windows show Type Text / Shell / Workflow.

### 2.4 `SavedShortcut` model decision (fixes R1–R3) — **maintainer decision needed**

| Option | Description | Cost / risk |
|---|---|---|
| **A. Resolve-by-id** (recommended) | Binding keeps `id` (+ optional display-name snapshot for offline/robustness); GUI and agent resolve `name`/`target`/`icon` from `config.custom_shortcuts` at use time. Edit/delete propagate automatically. | Wire + config change: agent must load shortcuts before resolving; deleted-id fallback (`Action::None` + warning) needed; `PROTOCOL_VERSION` bump + `wire_format` test (ipc rule); migration for existing bindings. |
| **B. Propagate-on-write** | Keep the denormalized shape; `commit_custom_shortcut` sweeps all bindings (device + keyboard + ring) rewriting copies; delete sweeps to `Action::None` with confirm. | No wire change; but sweep must cover every binding site forever; races with per-app profiles. |
| C. Status quo | Silent drift. | Not acceptable if edit UI is exposed everywhere. |

Phase 1–3 can proceed under either option; Phase 5 implements the choice.

---

## 3. Phased execution plan

Each phase is independently shippable; sizes S/M/L are relative.

### Phase 0 — Characterization & audit (S)
- Pin current behavior with tests: row-building tests for
  `action_rows_matching` (affordance matrix per flags), a test asserting
  `commit_custom_shortcut`'s (non-)propagation (documents R1), and the
  existing `wire_format` snapshot for the IPC types.
- Audit the agent-side resolution of `SavedShortcut` (§1.5 R3 decision input).
- Extract every hardcoded string in `shortcut_builder.rs`,
  `action_composer.rs`, `editors.rs`, `panel_action_rows` into a list; diff
  against `locales/en.yml` (many keys already exist).
- **Gate:** tests only; no behavior change.

### Phase 1 — Consistency quick wins (S, no architecture change)
1. i18n: wrap all strings from Phase 0 in `tr!`; add missing keys at the
   same position in **all** `locales/*.yml`; standardize on `…`;
   `cargo test -p openlogi-desktop i18n`.
2. Affordance parity: pass `on_edit` into the keyboard panel and K270 lists
   (they can already host the modal via their own `open_shortcut_builder*`
   once §2.2 lands — interim: skip pencil, **hide delete when edit is
   unavailable** so C2 disappears either way).
3. Delete confirmation popover on all surfaces ("Used by 3 buttons" when the
   reference sweep finds hits — pure read of current config).
4. Add search to the keyboard panel (S4) — it already reuses
   `action_rows_matching`, just missing the input.

### Phase 2 — Extract `ActionLibrary` (M)
- Build §2.1 from the *mouse inline* variant (most complete); migrate
  surfaces one at a time: S2 → S1 → S5 → S4 → S3 (ring last, it diverges
  most). Delete `action_ring/editor.rs::action_sections` and the duplicate
  `action_icon_path` as each migrates.
- Acceptance: one row-builder in the workspace; `rg "fn action_rows"` finds
  exactly one definition.

### Phase 3 — `ShortcutBuilder` host service + single composer UX (M)
- §2.2: move the modal to app-root; delete both per-view copies; wire
  `+ New Shortcut` into the ring editor, keyboard panel, and K270 list
  headers (same button as S2).
- §2.4-D4: ring's inline `composer_section` becomes either (a) removed in
  favor of "New/Edit Shortcut" opening the shared modal, or (b) an embed of
  `ActionComposer` — decide by screen-space feel in the ring panel.

### Phase 4 — Power User everywhere (M)
- §2.3 extraction; add the Power User section behind `ActionLibrary`'s
  `.power_user(true)` to mouse, ring, K270; keep catalog exclusion in core
  (the section is a deliberate GUI affordance, not catalog pollution).
- macOS-gate `RunAppleScript`; cross-lint Linux/Windows per
  `.claude/rules/cross-platform.md` (macOS-green proves nothing there).
- Commit path: mouse/ring use `commit_binding`, keyboard uses
  `commit_keyboard_binding` — already parameterized by the library's commit
  closure.

### Phase 5 — Data integrity (M–L, depends on §2.4 decision)
- Option A: binding shape change; migration in `openlogi-core`
  (`SCHEMA_VERSION` bump + migration chain); agent resolves by id with
  deleted-id fallback; `PROTOCOL_VERSION` bump + `cargo test -p
  openlogi-ipc --test wire_format`; GUI shows "renamed/removed shortcut"
  state on stale bindings.
- Option B: reference sweep in `commit_custom_shortcut` /
  `delete_custom_shortcut` across device bindings, keyboard bindings, ring
  slots, per-app profiles; delete-confirm dialog lists affected controls.
- Either way: usage-count read helper on `AppState` (feeds Phase 1's
  confirmation and the Shortcuts page below).

### Phase 6 — Backlog items (§4) as separate small PRs.

---

## 4. Additional UX/UI improvement backlog

Prioritized; P = priority.

| P | Idea | Rationale |
|---|---|---|
| P1 | **Dedicated "Shortcuts" sidebar page** managing the global library: list, search, rename, duplicate, delete, icon, and a "Used by" column (device/button/ring slot/keyboard key). Matches the global scope of `config.custom_shortcuts` (C7). | discoverability + management |
| P1 | **Chord recorder** in `ActionComposer`: press the real modifier+key combination to capture, instead of toggling Win/Cmd/Ctrl/Alt/Shift/Fn buttons then typing a key. Esc-to-cancel, backspace to clear. | industry-standard (Options+, Raycast); halves modal complexity |
| P2 | **"Test / Try it" button** in the shortcut modal and Power User editors: fire the action once before committing (safe for shortcuts/text; confirm dialog for shell/AppleScript). | catches typos in scripts now silently bound to a button |
| P2 | **Per-step editing + drag-reorder in Workflow editor** (today: add blank step / remove only; payload of an existing step is uneditable). Reuse the chord recorder and text editor per step type. | the biggest functional hole in Power User |
| P2 | Row affordances move into a **kebab menu** (Edit / Duplicate / Delete) instead of inline pencil+X; keeps rows calm and makes the destructive item consistent, not floating. | C2/C3 |
| P2 | **Undo toast** after delete (keep entry in memory until dismiss) — cheaper than confirm fatigue, pairs with the Phase 1 confirm. | safety |
| P3 | **Live preview line** in the modal: render the resulting `Action` label exactly as the row will show it ("Open Safari", "Type "…"). | WYSIWYG binding |
| P3 | Monospace font + line numbers/growing textarea for AppleScript/Shell fields (single-line input today). | scripts are not one-liners |
| P3 | Empty state in the Custom Shortcuts section: "No custom shortcuts yet — **+ New Shortcut**" CTA row. | turns the section into a teacher |
| P3 | Keyboard panel header shows the **currently bound action** above the list (ring editor already does this with `current_label`). | orientation after scroll |
| P3 | Duplicate-name warning in the modal (ids are unique, names are not). | library grows readable |
| P3 | Accessibility pass: `aria_label` on pencil/X/kebab buttons, roving focus for Power User rows, `Role::MenuItem` consistency; search box `aria` labelling. | rows already use `MenuRow`/roles — finish the job |
| P4 | Icon picker polish: show target-derived default icon as placeholder in `ActionComposer`'s icon grid (currently only the ring's `icon_editor` adapts by target). | consistency with `action_icon_path` fallbacks |

---

## 5. Constraints from the project rules (non-negotiables)

- **IPC** (`.claude/rules/ipc-protocol.md`): `CustomShortcutEntry`,
  `Action`, `ActionTarget` ride the wire. Any Phase 5 shape change →
  append-only, `PROTOCOL_VERSION` bump, `wire_format` test green.
- **i18n** (`.claude/rules/i18n.md`): every new key in **all**
  `locales/*.yml` at the same position; `cargo test -p openlogi-desktop i18n`.
- **GUI rule** (`.claude/rules/gui.md`): anything the overlay also needs goes
  to `openlogi-ui`; icon-path dedup (§2.1) must respect that the overlay
  links `openlogi-ui`, never `openlogi-desktop`.
- **Cross-platform** (`.claude/rules/cross-platform.md`): AppleScript gating
  and any cfg blocks → cross-lint or hand-audit Linux/Windows.
- **Verification**: affected-package tier of the local gate per phase
  (`openlogi-desktop` + `openlogi-core` changes fan out widely — use
  `cargo tree --invert`); Phase 5 additionally runs the ipc tests and, if
  agent behavior changes, hardware-verification notes in the PR ("not
  runtime-tested on hardware" where true).

## 6. Open questions for the maintainer

1. §2.4: resolve-by-id (A) vs propagate-on-write (B)? Does the agent
   currently trust the embedded copy (Phase 0 will confirm)?
2. Should Power User actions be assignable on **mice** too, or is
   keyboard-only a deliberate scope choice? (Plan assumes yes-everywhere.)
3. Ring editor: drop the inline composer or embed `ActionComposer`?
4. Is a new top-level "Shortcuts" sidebar page acceptable navigation change,
   or keep management embedded in pickers only?
5. Delete UX: confirm dialog, undo toast, or both?
