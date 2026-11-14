## 1. Answers & Architectural Decisions for the Open Questions

### Q1: `SavedShortcut` Data Model — Resolve-by-ID (Option A) vs. Propagate-on-Write (Option B)?

* **Answer & Agent Resolution Audit:** In `crates/openlogi-core/src/binding/effect.rs` (L278–282), `Action::SavedShortcut { target, .. }` lowers directly from its embedded `target` field (`ActionTarget::Shortcut(combo) => Effect::Key(combo)` and `ActionTarget::Application(_) => Effect::AgentSide`) without looking up `id` in `config.custom_shortcuts`. Similarly, UI label and icon helpers (`Action::label()` in `action.rs` L323–333 and `action_icon_path()` in `picker.rs` L117–123) read the embedded `name`, `target`, and `icon` directly from the `Action::SavedShortcut` variant.


* **Recommendation — Stage Option B immediately (Phase 1/3), schedule Option A for the fork-to-trunk schema migration:**
* **Why Option B now:** ` audit_result.md` and `audit_migrationplan.md` note that on-disk `SCHEMA_VERSION = 7` and `IPC PROTOCOL_VERSION = 29/30` already carry fork-specific divergence around `[[custom_shortcuts]]` and `Action::SavedShortcut`. Changing the wire shape of `Action::SavedShortcut` right now adds another IPC protocol bump and config migration before the trunk renumbering (`v7 → v12`).


* **How Option B works cleanly:** Centralize a reference-sweep mutator inside `Config::save_custom_shortcut` and `Config::delete_custom_shortcut` (`crates/openlogi-core/src/config.rs` L1077–1094). Because all bindings (per-device `bindings`, `gesture_maps`, per-app overrides, global `keyboard.bindings`, K270 `media_keys`, and `action_ring` slots) live inside `Config`, a single method `Config::cascade_custom_shortcut_update(&entry)` and `Config::cascade_custom_shortcut_delete(id) -> usize` updates or clears every embedded `Action::SavedShortcut` atomically on save/delete with **zero wire or schema changes**.





### Q2: Target Model — Keep `ActionTarget` Disjoint (Option A) vs. Extend with `Text` (Option B) vs. Full Macro (Option C)?

* **Answer & Recommendation — Option A for Phases 1–4, with a "Save as Custom Shortcut…" promotion path where supported; defer Option B (`ActionTarget::Text`) until Phase 5:**
* Currently, `ActionTarget` (`Shortcut(KeyCombo)` | `Application(ApplicationTarget)`) was just unified in config schema `v7` and IPC `v29`.


* Rather than immediately adding a third `ActionTarget::Text` variant (which forces an IPC `PROTOCOL_VERSION` bump and spreads the Linux/Windows `TypeText` unimplemented gap into saved shortcuts), keep `ActionTarget` as `Shortcut | Application` during the UI unification phases.


* Solve the UX confusion between `ActionComposer`'s **"Target Key"** and Power User's **"Type Text…"** immediately by:
1. Replacing the manual modifier-toggle + text box in `ActionComposer` with the **P1 Chord Recorder** (so users press keys to record a chord rather than typing strings into a text box).


2. Renaming the field label from `"Target Key"` to `tr!("Key Combination")` with a live keycap preview.







### Q3: Should Power User Actions Be Assignable on Mouse, Actions Ring, and K270?

* **Answer & Recommendation — Yes for Mouse Buttons and K270 Media Keys; No for Actions Ring (keep Ring to catalog + saved shortcuts/apps):**
* **Mouse & K270:** At the execution layer (`effect.rs`), `Action::TypeText`, `Action::RunAppleScript`, `Action::RunShellCommand`, and `Action::Workflow` are regular `Action` variants that lower to `Effect::Text` and `Effect::Script`. Both mouse button maps (`BTreeMap<ButtonId, Action>`) and K270 maps (`BTreeMap<K270MediaKey, Action>`) store `Action` directly. Once `PowerUserEditorState` is extracted from `FunctionRowView`, exposing the Power User section in the Mouse Inspector (`S2`) and K270 Inspector (`S5`) requires only passing `.power_user(true)` to the shared `ActionLibrary`.


* **Actions Ring (`S3`):** Ring slots are filtered by `RingAction::is_ok` and designed around fast radial visual recognition (`ActionRingEntry` with 54 icons). Multi-step workflows and shell scripts fit poorly in an inline slot editor unless wrapped as a named shortcut later; keep `S3` focused on catalog actions, saved shortcuts, and app targets.





### Q4: Linux/Windows `TypeText` — Gate the UI Row Now or Implement in `openlogi-inject` First?

* **Answer & Recommendation — Gate in UI immediately (Phase 1) via a platform capability helper, then unlock per-OS as `openlogi-inject` lands:**
* Showing an interactive `"Type Text…"` or `"Run AppleScript…"` row that silently logs a warning at runtime violates user trust.
* Introduce `PowerUserKind::supported_on_current_platform() -> &'static [PowerUserKind]` in `ui/power_user.rs`:
* `#[cfg(target_os = "macos")]` → `[TypeText, RunAppleScript, RunShellCommand, Workflow]` (all 4).
* `#[cfg(not(target_os = "macos"))]` → `[RunShellCommand, Workflow]` (and filter out `WorkflowStep::RunAppleScript` and `WorkflowStep::TypeText` from step creation until Windows `SendInput(..., KEYEVENTF_UNICODE)` and Linux `uinput` text synthesis are implemented).





### Q5: Delete UX — Confirmation Dialog, Undo Toast, or Both?

* **Answer & Recommendation — Conditional Confirmation + Always-On Undo Toast (Backlog P2):**
* Using the reference-count helper `AppState::custom_shortcut_usage(&id) -> ShortcutUsageSummary` (counting references across device buttons, gestures, per-app profiles, keyboard keys, K270 keys, and ring slots):


* **If `usage_count == 0`:** Delete immediately without a blocking modal and show a non-blocking **Undo toast** (`"Deleted '<Name>' — [Undo]"`), keeping the `CustomShortcutEntry` in transient memory on `AppState` for 6 seconds.
* **If `usage_count > 0`:** Show an inline confirmation popover on the row (`"Used by N bindings (e.g. MX Master 3S · Back, Keyboard · F5). Delete and unbind?"`) AND still offer the Undo toast (which restores both the library entry and its former bindings snapshot).





### Q6 (from Parent Plan): Actions Ring Editor — Drop Inline `composer_section` or Embed `ActionComposer`?

* **Answer & Recommendation — Drop the raw inline `composer_section` and use `+ New Shortcut` Modal + a "Quick Custom Action" collapsible `ActionComposer`:**
* Currently, `features/action_ring/editor.rs` (L109+) builds its own `composer_section` with raw `["Win", "Cmd", "Ctrl", "Alt", "Shift", "Fn"]` buttons calling `toggle_modifier_in_text`, a ` shortcut_input` with `"Add Shortcut"` (which commits an anonymous `Action::CustomShortcut(combo)`), and a `path_input` with `"Add Path"` (committing `Action::OpenApplication`). Meanwhile, it also renders a separate 54-icon `icon_editor` duplicating `ActionComposer::render_icon_grid`.


* **Unification:**
1. Replace `action_ring/editor.rs`'s custom `action_sections` + `ring_catalog` with the shared `ActionLibrary` (configured with `.filter(ActionFilter::RingCompatible)` and `.custom_shortcuts(Full)`), which automatically adds the **`+ New Shortcut`** button and search bar to the Actions Ring editor.


2. Replace the duplicate 54-icon grid in `action_ring/editor.rs` with `ui::action_composer::render_icon_grid`.


3. Remove the raw `composer_section` from `action_ring/editor.rs`; creating a shortcut or app launcher from the Ring now goes through `+ New Shortcut` (`ShortcutBuilderModal`), making every custom ring action a named, icon-bearing, reusable `SavedShortcut`. For existing configs with anonymous `Action::CustomShortcut` or `Action::OpenApplication` in a ring slot, show a **"Save to Library…"** chip in the slot header to promote it into `custom_shortcuts`.







---

## 2. Backlog Evaluation & Architectural Integration

Rather than deferring all 13 backlog items to an open-ended "Phase 6", several items directly solve core architectural defects and should be pulled into the main phases:

| P | Backlog Item | Architectural Fit & Scheduling Decision | Target Phase |
| --- | --- | --- | --- |
| **P1** | **Chord recorder in `ActionComposer**` | **Pull into Phase 3.** Directly eliminates the UX confusion between `"Target Key"` and `"Type Text…"` AND fixes the historical confusion around how the `"Modifiers"` buttons interact with the text input. Keep modifier chips as clickable toggles/indicators synced with key capture.

 | **Phase 3** |
| **P1** | **Dedicated "Shortcuts" sidebar page** | **Schedule in Phase 5.** Since `config.custom_shortcuts` is global across all devices, a top-level or device-independent library view showing `"Used by"` badges, search, duplicate, and bulk management gives global shortcuts a proper home.

 | **Phase 5** |
| **P2** | **Row affordances → Kebab menu (`⋯`)** | **Pull into Phase 2 (`ActionLibrary`).** In narrow 300px–320px panels (`compact_panel`), inline Pencil + Close icons crowd long action names and cause accidental one-click deletes. A single `⋯` menu (`Edit…`, `Duplicate`, `Delete`) solves C2/C3 cleanly.

 | **Phase 2** |
| **P2** | **Undo toast after delete** | **Pull into Phase 3** alongside the reference-sweep cascade delete so every surface (`S1`–`S5`) gets safe deletion at once. | **Phase 3** |
| **P2** | **Per-step editing + reorder in Workflow editor** | **Pull into Phase 4 (`ui/power_user.rs`).** Right now `WorkflowStepRow` only previews and removes steps (`+ Add Step` always pushes an empty `WorkflowStep::TypeText(String::new())` with no way to change step type or edit payload!). Making steps editable is essential for `Workflow` to be functional. | **Phase 4** |
| **P2** | **"Test / Try it" button in modal & Power User** | **Schedule in Phase 4.** Calls into IPC / agent test-fire endpoint (with confirmation for `RunShellCommand` / `RunAppleScript`). | **Phase 4** |
| **P3** | **Empty state CTA in Custom Shortcuts section** | **Pull into Phase 2 (`ActionLibrary`).** When `state.custom_shortcuts().is_empty()`, render `"No custom shortcuts yet — + New Shortcut"` so users on Keyboard/K270/Mouse immediately discover the feature.

 | **Phase 2** |
| **P3** | **Keyboard panel header shows currently bound action** | **Pull into Phase 1.** Matches `action_ring/editor.rs` (`current_label`) and Mouse Inspector (`binding_inspector`) so scrolling the 500px list never loses context.

 | **Phase 1** |
| **P3** | **Live preview line in `ShortcutBuilderModal**` | **Pull into Phase 3.** Shows exact row appearance (`icon` + `name` + target subtitle, e.g., `⌘⇧P` or `Open Safari.app`).

 | **Phase 3** |
| **P3** | **Duplicate-name warning in modal** | **Pull into Phase 3.** Non-blocking warning caption when `name_input` matches an existing `CustomShortcutEntry.name` with a different `id`.

 | **Phase 3** |
| **P3** | **Accessibility pass (`aria_label`, roving focus)** | **Pull into Phase 2 & Phase 4** as `ActionLibrary` and `PowerUserEditorState` are built, rather than retrofitting later.

 | **Phase 2 / 4** |
| **P3** | **Monospace + multi-line input for AppleScript/Shell** | **Pull into Phase 4** inside `text_editor_card` when `kind` is `RunAppleScript` or `RunShellCommand`. | **Phase 4** |
| **P4** | **Icon picker default-icon placeholder polish** | **Pull into Phase 3** when unifying `render_icon_grid` in `ui/action_composer.rs` (`Shortcut(_)` → `Keyboard`, `Application(_)` → `Applications`).

 | **Phase 3** |

---

## 3. Refined UX/UI Redevelopment Plan

### 3.1 Target Component Architecture

```
crates/openlogi-desktop/src/ui/
├── action_library.rs      (NEW: unified searchable catalog + Custom Shortcuts + Power User rows)
├── shortcut_builder.rs    (UPDATED: app-level ShortcutBuilderHost + i18n + live preview + duplicate check)
├── action_composer.rs     (UPDATED: Chord Recorder + mode tabs + shared render_icon_grid with target fallback)
└── power_user.rs          (MOVED from features/keyboard/editors.rs: PowerUserEditorState + full Workflow step editor)

```

#### 1. `ui::action_library::ActionLibrary` (Single Action List Across All 5 Surfaces)

Replaces the 5 divergent list assemblies (`S1` mouse popover, `S2` mouse inline library, `S3` ring `action_sections`, `S4` keyboard `panel_action_rows`, `S5` K270 `render_inspector`):

* **Header Bar:** Section title (`tr!("Actions")`) + **`+ New Shortcut`** ghost button (`Button::new("new-custom-shortcut")`) + current-binding summary pill (for `S3`/`S4` side panels) + cleanable search input (`tr!("Search actions…")`).


* **Custom Shortcuts Section (`Category::CustomShortcuts`):**
* Always rendered at the top of the list (when unfiltered):


* If empty: renders a subtle empty-state card with `"No custom shortcuts yet"` and a **`+ Create Shortcut`** CTA (Backlog P3).
* If non-empty: renders each `SavedShortcut` row with its resolved icon, name, a muted target badge (`⌘⇧P` or app name), selection checkmark (`when(selected)` — fixing the bug in `picker.rs` L275 where `.when(selected && !is_custom, ...)` suppressed the checkmark on selected custom shortcuts!), and a **`⋯` Kebab Menu** (`Edit…`, `Duplicate`, `Delete`) (Backlog P2).






* **Standard Catalog Sections:** Filtered by `ActionFilter::All` or `ActionFilter::RingOnly` (`RingAction::is_ok`), using a single deduplicated `action_icon_path` function.


* **Power User Section (`tr!("Power User")`):**
* Enabled on `S4` (Keyboard), `S1`/`S2` (Mouse), and `S5` (K270).
* Filtered by `PowerUserKind::supported_on_current_platform()` so unsupported OS actions never render.



#### 2. App-Level `ShortcutBuilderHost` + Upgraded `ActionComposer`

* **Single Host Controller:** Instead of duplicating `shortcut_builder: Option<Entity<ShortcutBuilderModal>>` and 55 lines of `open_shortcut_builder_with` / `on_save` / `on_cancel` closures in `MouseModelView` (`mouse/view.rs` L176–232) and `ActionRingPanel` (`action_ring.rs` L61–115), `AppView` (or `AppState`) hosts a single modal instance rendered at the workspace root overlay layer. Any surface (`S1`–`S5`) can call `AppState::open_shortcut_builder(existing_entry, optional_auto_assign_callback)`.


* **Auto-Assign on Create:** When a user clicks `+ New Shortcut` from *inside* a specific key/button inspector (e.g. `F5` on the Keys tab or `Back` on the Mouse tab), saving the new shortcut both adds it to `config.custom_shortcuts` **and** binds it to the active control in one step.
* **Chord Recorder + Clearer Target Modes (`ActionComposer`):**
* Segmented mode toggle: **`[ Keyboard Shortcut | Launch App / Path ]`**.


* In **Keyboard Shortcut** mode:
* Replace the confusing `"Target Key"` text box with a **"Record Shortcut"** capture box (`"Click to record shortcut, or toggle modifiers below"`). Pressing a physical key combo captures the modifiers and keycode directly; `Esc` cancels capture; `Backspace` clears.


* Keep the modifier chips (`Win`, `Cmd`, `Ctrl`, `Alt`, `Shift`, `Fn`) synced bidirectionally (`parse_modifiers_from_text` / `toggle_modifier_in_text`) for combinations the OS intercepts (such as `Cmd+Tab` or `Fn` chords).




* **Icon Grid Fallback (Backlog P4):** Highlight the default target icon (`ActionRingIcon::Keyboard` for shortcuts, `ActionRingIcon::Applications` for apps) when `selected_icon` is `None` (`"Auto"`), and use `ui::theme::Palette` tokens (`pal.panel`, `pal.control`, `pal.control_hover`, `hsla` / `rgb(ACCENT_BLUE)` properly converted) to avoid the GPUI type mismatches seen in `shortcut_builder.rs`.





#### 3. Extracted `PowerUserEditorState` & Full Workflow Step Builder (`ui/power_user.rs`)

* **Reusable Controller:** Extract `active_editor`, `text_state`, and `workflow_draft` out of `FunctionRowView` into `pub struct PowerUserEditorState` so keyboard, mouse, and K270 inspectors can embed it without boilerplate.
* **Upgraded Text & Script Editors (Backlog P2, P3):**
* When `kind` is `RunAppleScript` or `RunShellCommand`, render a monospace multi-line/expanding input instead of a cramped single-line input.
* Add a **`Try It`** secondary button next to `Cancel` / `Save` that dispatches a one-shot test execution (with a warning prompt for shell/AppleScript).


* **Functional Workflow Editor (Fixes the Critical Power-User Gap — Backlog P2):**
* Currently, clicking `+ Add Step` in `workflow_editor_card` blindly appends `WorkflowStep::TypeText(String::new())`, and `WorkflowStepRow` only supports deleting the row (`remove_workflow_step`).
* **New Step Editor UX:**
1. Replace `+ Add Step` with an **`+ Add Step ▾`** menu (or segmented step-type selector): `Press Key`, `Type Text` (if supported on OS), `Delay (ms)`, `Shell Command`, `AppleScript` (macOS only).
2. Clicking an existing `WorkflowStepRow` expands an inline step editor (reusing the chord input for `PressKey`, numeric ms input for `Delay`, and text input for `TypeText`/`Shell`/`AppleScript`).
3. Add Up/Down reorder chevrons (or drag handles) on each step row.





---

### 3.2 Revised Phased Execution Roadmap

#### Phase 0 — Characterization Tests & String Audit (Size: S)

* Add unit tests in `crates/openlogi-core/src/config/tests.rs` verifying `save_custom_shortcut` and `delete_custom_shortcut` behavior across bound controls.


* Audit all untranslated literals in `ui/shortcut_builder.rs`, `ui/action_composer.rs`, `features/keyboard/editors.rs`, `features/keyboard/function_row.rs`, and `features/mouse/picker.rs` against `crates/openlogi-ui/locales/en.yml`.


* **Gate:** `cargo test -p openlogi-core -p openlogi-desktop`.

#### Phase 1 — Keys-Tab Parity Quick Wins, Cascade Propagation & i18n (Size: S–M)

1. **Fix Data Integrity (Q1 — Option B Cascade):** Update `Config::save_custom_shortcut` and `Config::delete_custom_shortcut` in `crates/openlogi-core/src/config.rs` to sweep and update/clear matching `Action::SavedShortcut { id, .. }` entries across device buttons, gestures, app overrides, keyboard bindings, K270 media keys, and ring slots. Add `Config::count_custom_shortcut_usages(id) -> usize`.


2. **Keys Tab Orientation & Search (Backlog P3):** In `FunctionRowView::config_panel` (`function_row.rs`), add the current-binding summary header above the divider and a search `InputState`.


3. **Platform Capability Gating (Q4):** Gate `PowerUserKind::RunAppleScript` and `PowerUserKind::TypeText` behind `#[cfg(target_os = "macos")]` (via `PowerUserKind::supported_on_current_platform()`).
4. **Fix Selected Checkmark Bug on Custom Shortcuts:** In `picker.rs` L275, fix `.when(selected && !is_custom, ...)` so selected custom shortcuts properly display their active selection state alongside the row menu.


5. **i18n Pass:** Wrap all hardcoded strings in `tr!(...)`, normalize ellipsis to `…`, and add missing keys at matching line positions across all `crates/openlogi-ui/locales/*.yml` files.



* **Gate:** `cargo test -p openlogi-desktop i18n` and `cargo test -p openlogi-core`.

#### Phase 2 — Extract Shared `ActionLibrary` + Kebab Row Menu + Empty State (Size: M)

1. Create `crates/openlogi-desktop/src/ui/action_library.rs` consolidating `grouped_catalog`, `action_rows_matching`, and `action_icon_path` (deleting the duplicate `features/action_ring/action_icons.rs`).


2. Replace inline Pencil + X buttons on `SavedShortcut` rows with a **`⋯` Kebab Menu** (`Edit…`, `Duplicate`, `Delete`) with proper `aria_label`s (Backlog P2, P3).


3. Add the empty-state CTA row in the `Custom Shortcuts` category when no shortcuts exist (Backlog P3).
4. Migrate surfaces in order: **S4 (Keys tab)** → **S2/S1 (Mouse)** → **S5 (K270)** → **S3 (Actions Ring)**.



* **Gate:** `rg "fn action_rows"` and `rg "fn action_icon_path"` each return a single definition in `openlogi-desktop`.

#### Phase 3 — Root `ShortcutBuilderHost` + Upgraded `ActionComposer` (Chord Recorder) + Safe Delete (Size: M)

1. Move `ShortcutBuilderModal` hosting to the app root so **`+ New Shortcut`** and **`Edit…`** work identically from **Keys (`S4`)**, **Media Keys (`S5`)**, **Mouse (`S1`/`S2`)**, and **Actions Ring (`S3`)**. Delete the duplicate host methods in `mouse/view.rs` and `action_ring.rs`.


2. Upgrade `ActionComposer` (`ui/action_composer.rs`):
* Add interactive **Chord Recorder** capture mode + rename `"Target Key"` to `tr!("Key Combination")` (Backlog P1).


* Deduplicate the 54-icon grid between `action_composer.rs` and `action_ring/editor.rs`, adding default target-icon highlighting (Backlog P4).


* Retire `action_ring/editor.rs`'s inline `composer_section` in favor of the shared modal + a `"Save to Library…"` promotion chip for legacy anonymous slot actions (Q6).




3. Add **Live Preview line** and **Duplicate-name warning** to `ShortcutBuilderModal` (Backlog P3).


4. Add **Conditional Delete Confirmation** (when `usage_count > 0`) + **6-second Undo Toast** (Q5, Backlog P2).

#### Phase 4 — Shared `PowerUserEditorState` + Full Workflow Editor + Multi-Surface Rollout (Size: M)

1. Move `features/keyboard/editors.rs` and the draft fields on `FunctionRowView` into `ui/power_user.rs` (`PowerUserEditorState`).


2. Upgrade `workflow_editor_card`:
* Add step-type picker (`Press Key`, `Type Text`, `Delay`, `Shell`, `AppleScript`), inline per-step payload editing, and step reordering (Backlog P2).


3. Upgrade `text_editor_card` with monospace multi-line input for scripts (Backlog P3) and a **`Try It`** test button (Backlog P2).
4. Enable `.power_user(true)` on Mouse (`S1`/`S2`) and K270 (`S5`) inspectors (Q3).

#### Phase 5 — Global "Shortcuts" Library Page & Future Schema Cleanup (Size: M–L)

1. Add the **Dedicated "Shortcuts" Sidebar Page** (Backlog P1) for global library management: search, filter by target type (Shortcut vs. App), `"Used by"` column listing every bound device/key/slot with click-to-jump, duplicate, and bulk delete.
2. When the planned fork-to-trunk config migration (`v7 → v12` in `audit_migrationplan.md`) runs, optionally normalize `Action::SavedShortcut` to resolve by `id` (Q1 Option A) and evaluate adding `ActionTarget::Text` once Linux/Windows unicode text injection lands in `openlogi-inject` (Q2/Q4).



---

Would you like me to update `docs/keys_tab_shortcuts_power_user_plan.md` with this refined plan, or start implementing **Phase 1** right away?
