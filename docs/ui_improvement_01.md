With the execution paths, K270 search input, and app picker in place, the remaining high-leverage UX/UI opportunities fall into **Phase 6** of your unification plan (plus two ergonomic polish items surfaced by the new app picker and cross-surface custom actions):

1. **Interactive Chord Recorder + Live Preview Pill in `ActionComposer`:**
* Even with bi-directional modifier chips, users still have to type key names (`F12`, `PageUp`, `Space`) into a text box. Adding a **"Record"** toggle button inside `ActionComposer` that captures the next physical key + modifier combination directly (with `Esc` to cancel, `Backspace` to clear, and a live formatted keycap preview badge like `[ ⌘ Cmd ] + [ ⇧ Shift ] + [ P ]`) eliminates syntax guesswork while keeping manual text/chip editing for OS-intercepted keys (`Win`, `Fn`).




2. **Real Native App Icons in `ActionComposer` Suggestions (Plus App Icon Auto-Match):**
* Right now, installed app suggestions in `ActionComposer` show generic icons and default `selected_icon` to `ActionRingIcon::Applications`. Rendering the actual OS app icon (already extracted by `appcatalog` for per-app profiles) next to each suggestion in the dropdown makes scanning installed apps dramatically faster.




3. **"Test / Run Now" Button in `ShortcutBuilderModal` & `PowerUser` Editors:**
* When configuring a shell command, AppleScript, typed text, or workflow, users currently have to save, close the modal, and press the physical mouse button or key to verify it works. Adding a compact **"Test"** button in the editor footer (with a brief confirmation guard for `Shell` / `AppleScript`) lets users verify behavior immediately inside the modal.




4. **Safe Delete with Usage Count & Undo Toast (`C3` / `Q5`):**
* Clicking the `X` on a custom action row still deletes it immediately without warning if it is bound to 4 buttons/keys, nor offering an undo. Using `Config::count_custom_shortcut_usages(&id)` (already added in Phase 3) to show an inline confirmation when `usage_count > 0` (`"Used by N bindings — Delete?"`) plus a 6-second **"Undo"** toast banner protects against accidental data loss.




5. **Global "Custom Actions" Sidebar Management View (`C7`):**
* Custom actions live in global config, yet users can only inspect or manage them inside a device-specific button/key picker. Adding a lightweight **"Custom Actions"** view (or modal library manager) showing all saved custom actions, their kind badge (`Shortcut`, `App`, `Text`, `Script`, `Workflow`), a **"Used by"** badge (`M720 Back`, `F12`, `K270 Calc`), **Duplicate**, **Edit**, and **Delete** gives users a single home for their library.





---

### Ready-to-Use Prompt for Google Antigravity

Copy and paste the prompt below into Antigravity to execute this polish pass:

```markdown
Implement the **Phase 6 UX/UI Polish & Ergonomics Pass** across `crates/openlogi-desktop` and `crates/openlogi-ui`:

### Objective
Elevate the Custom Actions experience with five high-impact UX/UI improvements while preserving all existing IPC, schema, and i18n invariants:
1. **Interactive Chord Recorder & Keycap Preview Pill** in `ActionComposer`
2. **Native App Icons in `ActionComposer` Suggestions**
3. **"Test Action" Button** in `ShortcutBuilderModal` & `PowerUser` editors
4. **Safe Delete (Usage Warning + 6s Undo Toast)**
5. **Global "Custom Actions" Library Manager** (with "Used by" badges & Duplicate)

---

### Step-by-Step Implementation Plan

#### 1. Interactive Chord Recorder & Keycap Preview Pill (`crates/openlogi-desktop/src/ui/action_composer.rs`)
- In `ComposerTargetMode::Shortcut`:
  - Add a `"Record"` toggle button beside `key_input` (`tr!("Record")` / `tr!("Press keys…")` while recording).
  - When recording is active, capture keystrokes via `on_key_down`:
    - Pressing `Escape` without modifiers cancels recording.
    - Pressing `Backspace` without modifiers clears `key_input`.
    - Pressing any non-modifier key (with or without `Cmd`/`Ctrl`/`Alt`/`Shift`) formats the chord into `key_input`, updates the bi-directional modifier chip highlights, and exits recording mode.
  - Below `key_input`, render a live **Keycap Preview Pill** showing the parsed `KeyCombo::rendered_label()` when valid (e.g., `⌘ + ⇧ + P`) or a muted syntax hint (`e.g. Cmd+Shift+P, Win+V, Fn+F11`) when empty/invalid.

#### 2. Native App Icons in `ActionComposer` Suggestions (`crates/openlogi-desktop/src/ui/action_composer.rs`)
- When rendering the top 5 matching installed applications in `ComposerTargetMode::Application`:
  - Render each app's native icon from `appcatalog` (falling back to `"action-icons/applications.svg"` when unavailable) alongside the app display name and muted filesystem path.

#### 3. "Test Action" Button (`crates/openlogi-desktop/src/ui/shortcut_builder.rs` & `ui/power_user.rs`)
- Add a ghost `"Test"` (`tr!("Test")`) button on the left side of the modal/editor action footer (enabled whenever the composed target/action is valid):
  - For `Shortcut`, `Application`, `TypeText`, and `Workflow`: trigger a one-shot test execution via the existing agent IPC `test_action` / `execute_action` endpoint (or local `openlogi_inject::execute` fallback).
  - For `AppleScript` and `Shell`: require a second click (`tr!("Confirm Run?")`) within 4 seconds before executing, preventing accidental one-click script runs.

#### 4. Safe Delete with Usage Count & Undo Toast (`ui/action_library.rs` & `state/bindings.rs`)
- Before calling `state.delete_custom_shortcut(&id)`:
  - Check `state.custom_shortcut_usage_count(&id)` (using `Config::count_custom_shortcut_usages`).
  - If `usage_count > 0`, require a one-step inline confirmation on that row (`tr!("Used by %{count} · Delete?", count => usage_count)`) that auto-resets if unconfirmed after 4 seconds.
  - Store the deleted `(CustomShortcutEntry, previous_bindings_snapshot)` in transient memory on `AppState` for 6 seconds and display a floating bottom **Undo Toast** (`tr!("Deleted \"%{name}\"", name => entry.name)` + `tr!("Undo")` button) that restores both the library entry and its former bindings if clicked.

#### 5. Global "Custom Actions" Manager (`crates/openlogi-desktop/src/features/custom_actions/` & `app.rs`)
- Add a `"Custom Actions"` entry in the main app navigation (or top-level library drawer accessible from device headers) listing all entries in `config.custom_shortcuts`:
  - Search bar + kind filter pills (`All`, `Shortcut`, `App`, `Text`, `Script`, `Workflow`).
  - Each card/row shows: custom icon, action name, target preview summary, a `"Used by"` pill listing bound controls (e.g. `M720 · Back`, `F12`, `K270 · Calculator` — or `Unused`), and **Edit**, **Duplicate** (clones entry as `"<Name> Copy"`), and **Delete** buttons.

#### 6. Localization & Verification Gate
- Wrap all new UI labels in `tr!(...)` and add the new keys at identical line positions across all 22 `crates/openlogi-ui/locales/*.yml` files.
- Add unit tests for chord recording formatting, duplicate-entry cloning, usage summary formatting, and undo restoration.
- Run and pass:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p openlogi-ui locale`
  - `cargo test --workspace`

```
