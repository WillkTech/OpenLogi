> Superseded by [custom_actions_ui_unification_plan.md](custom_actions_ui_unification_plan.md)

# Keys Tab — Custom Shortcuts & Power User: UI Investigation and Redevelopment Plan

Status: **draft for discussion** (no code changed by this document).
Scope: the **Keys tab** (`DetailTab::Keys` → `FunctionRowView`) and its two
custom-behavior systems — **Custom Shortcuts** (`SavedShortcut` /
`ShortcutBuilderModal`) and **Power User** (Type Text / Run AppleScript / Run
Shell Command / Workflow) — plus the shared components they should reuse.
Relationship: this plan **derives from and integrates**
[`shortcuts_power_user_ui_unification_plan.md`](shortcuts_power_user_ui_unification_plan.md)
(the "parent plan"). The parent's ground-truth audit, target architecture
(§2.1–2.4), phase model, and constraints were re-verified against the current
tree and remain accurate; everything relevant is adopted here with attribution.
This document re-sequences the work Keys-first and adds three findings the
parent plan missed (§3 duplication analysis, platform gating, the target-model
decision in §5.2).

All line numbers are anchors at the time of writing, not guarantees.

---

## 1. Motivation

A user binding a function key on the **Keys** tab today can:

- **see and pick** custom shortcuts created elsewhere (they are injected into
  every action list by `picker.rs::grouped_catalog` L33–64),
- **delete** them (the X renders unconditionally on `SavedShortcut` rows,
  `picker.rs` L259–271),
- but **cannot create or edit** them — `panel_action_rows` passes
  `on_edit: None` (`keyboard/function_row.rs` L859) and there is no
  `+ New Shortcut` button. Creating one requires owning a mouse and finding
  the button in the mouse button picker (`inspector.rs` L650).

Meanwhile the **Power User** section (Type Text / AppleScript / Shell /
Workflow) exists **only** on this tab — no other surface can assign those
actions — and its rows are shown on every platform even though two of the four
actions are **not implemented in the injector outside macOS** (§2.4).

The two systems also present two near-identical single-line text inputs that
mean very different things ("Target Key" chord vs. "Text to type"), which
prompts the obvious question: *do they overlap?* (§3 answers: functionally no;
visually yes; and the real redundancy is elsewhere.)

## 2. Verified ground truth (Keys-tab lens)

### 2.1 Surface matrix (re-verified from parent §1.1)

| # | Surface | List builder | Search | + New | Edit pencil | Delete X | Power User |
|---|---|---|---|---|---|---|---|
| S1 | Mouse popover picker (`inspector.rs` + `picker.rs`) | `action_rows_matching` | ✅ | ✅ | ✅ | ✅ | ❌ |
| S2 | Mouse inline `action_library` (`inspector.rs` L616–677) | `action_rows_matching` | ✅ | ✅ L650 | ✅ | ✅ | ❌ |
| S3 | Actions Ring editor (`action_ring/editor.rs`) | own sections | ❌ | ❌ | ✅ | own list | ❌ |
| **S4** | **Keys tab panel** (`keyboard/function_row.rs` `config_panel` L783, `panel_action_rows` L852) | `action_rows` (`on_edit = None`) | ❌ | ❌ | ❌ | ✅ unconditional | ✅ (only here) |
| S5 | K270 Media keys (`k270/view.rs`) | `action_rows_matching` | ✅ | ❌ | ❌ | ✅ unconditional | ❌ |

### 2.2 Keys-tab specifics (all re-confirmed)

- The panel list is the **full catalog including the Custom Shortcuts
  section** (`action_rows` → `action_rows_matching` → `grouped_catalog`), so
  keyboard users live next to a feature they cannot manage.
- Destructive-without-constructive (parent **C2**): X renders whenever the row
  is `SavedShortcut`; the pencil is `when_some(on_edit)` (`picker.rs` L240),
  and S4 passes `None`.
- No search input (parent **C4**); no `+ New Shortcut` (parent **C1**).
- Power User draft state (`active_editor`, `text_state`, `workflow_draft`) is
  welded to `FunctionRowView` (`function_row.rs` L119–133, L188–239) — the
  parent's **D6**; not reusable by any other host.
- Shortcut-builder host wiring is copy-pasted between `mouse/view.rs`
  L176–232 and `action_ring.rs` L61–115 (parent **D3**); the keyboard view has
  **no** copy at all, so it cannot even open the modal.

### 2.3 Hardcoded strings (parent **C5**, confirmed)

`ui/shortcut_builder.rs` ("New Custom Action"/"Edit Custom Action" L109–113,
"Name", "Cancel", "Save Action"), `ui/action_composer.rs` ("Target Key"
L342–344 and friends), `keyboard/editors.rs` (`PowerUserKind::heading` L46–53,
placeholders L56–63, "+ Add Step"/"Save Workflow"), and the Power User row
labels (`(*label).to_string()`, `function_row.rs` L918). Many locale keys
already exist (`locales/en.yml` L412–418). Ellipsis style drifts (`…` in rows,
bare in headings).

### 2.4 Platform support vs. UI gating (**new finding**)

| Power User row | macOS | Linux | Windows | UI gates it? |
|---|---|---|---|---|
| Type Text… | ✅ `post_unicode` (`inject/macos.rs`) | ❌ warn + no-op (`inject/linux.rs` L54–59) | ❌ warn + no-op (`inject/windows.rs`) | **No** |
| Run AppleScript… | ✅ | ❌ (`dispatch_script` warns) | ❌ | **No** |
| Run Shell Command… | ✅ | ✅ | ✅ | n/a |
| Workflow… | ✅ | ✅ (AppleScript steps inside warn) | ✅ | **No** |

The parent plan gated only `RunAppleScript` (§2.3); the audit shows
**`TypeText` is also macOS-only**. A Linux/Windows user can bind "Type Text"
to F1 and get silence — the row must be capability-gated or the injector
implemented (open question Q4).

### 2.5 Data-integrity risks R1–R3 (inherited from parent §1.5, unchanged)

Edit does not propagate (bindings freeze a copy of name/target/icon); delete
orphans bindings into zombie frozen copies; the denormalized
`Action::SavedShortcut { id, name, target, icon }` will drift as fields grow.
Confirmed: `commit_custom_shortcut` / `delete_custom_shortcut`
(`state/bindings.rs` L549–560) → `config.rs` `save_custom_shortcut` L1077 /
`delete_custom_shortcut` L1090 — no reference sweep anywhere. **Exposing
create/edit/delete on the Keys tab multiplies exposure of these bugs**, so
Phase 5 must not lag the affordance work by a full release.

## 3. Does "Type Text…" overlap the composer's "Target Key" input?

**No functional overlap — but the UI implies one, and the real redundancy sits
one level up.**

| | "Target Key" (`ActionComposer`, `action_composer.rs` L342) | "Type Text…" (`editors.rs`) |
|---|---|---|
| Parses | `key_str.parse::<KeyCombo>()` — a chord like `Cmd+Shift+P`; free prose fails validation | free text, no validation |
| Produces | `ActionTarget::Shortcut(KeyCombo)` inside a `SavedShortcut` | `Action::TypeText(String)` direct binding |
| Lowers to | `Effect::Key` (`effect.rs` L279–283) | `Effect::Text` (`effect.rs` L284) |
| Executes as | chord replay: `post_keycombo` / `press_combo` / `post_custom_shortcut` — layout-dependent keycodes | `post_unicode` — layout-independent unicode string (macOS only, §2.4) |

- The overlap is only at the degenerate edge (a bare printable key: chord "A"
  vs. text "a") and in the **visual grammar**: two identical single-line
  inputs, one of which rejects everything the other accepts. That is a
  labeling/discoverability problem, not a duplicated feature.
- The **actual redundancy** is that both are parallel "define your own
  behavior" flows with **disjoint payloads**:
  - `ActionTarget` = `Shortcut | Application` only
    (`openlogi-core/src/binding/action_target.rs` L10–15), so a **saved
    shortcut can never carry text or a script** — "type my email signature"
    cannot be created once and reused from a mouse button, ring slot, or K270
    key;
  - Power User actions bind **directly to one key** and are excluded from the
    catalog (`not_pickable`, `action.rs` L356–373), so they can never be a
    named, reusable, cross-device entry either.
- Conclusion for the plan: do **not** merge the two inputs; **unify at the
  library level** — make the shortcut library the single home for named
  reusable actions and let its target set grow (§5.2), while disambiguating
  the two inputs' labels (§5.4).

## 4. Assessment of the parent plan

| Parent item | Verdict | Where it lands here |
|---|---|---|
| §2.1 `ActionLibrary` component (kills D1/D2/D5) | **Adopt** — still the right center of gravity | Phase 2 |
| §2.2 App-level `ShortcutBuilder` host (kills D3, enables C1) | **Adopt** — prerequisite for + New/edit on S4 | Phase 3 |
| §2.3 Shared `PowerUserEditorState` (kills D6) | **Adopt**, extended with platform capability filtering | Phase 4 |
| §2.4 `SavedShortcut` resolve-by-id vs. propagate-on-write | **Adopt as open decision** — becomes more urgent once S4/S5 get edit/delete | Phase 5, Q1 |
| Phase 1 quick wins (i18n, hide-delete-when-no-edit, search on S4) | **Adopt** | Phase 1 |
| Phase 4 "Power User everywhere" | **Defer** — this plan is Keys-first; making Power User assignable from mice stays open (Q3) | Q3 |
| §4 backlog (chord recorder, per-step workflow editing, monospace script fields, empty-state CTA, undo/confirm) | **Adopt selectively** — chord recorder and per-step editing directly relieve the §3 confusion | Phase 6 |
| §5 constraints | **Adopt verbatim** | §7 |
| AppleScript-only platform gating | **Extend** — TypeText is also unimplemented off-macOS (§2.4) | Phase 1/4 |
| (absent) target-model overlap between shortcut payloads and Power User payloads | **New** | §5.2, Q2 |

## 5. Target design

### 5.1 Keys-tab parity (the headline change)

The Keys-tab config panel (`config_panel`) gains, in this order:

1. a **search input** (the list already supports `query`; only the input is
   missing);
2. a header **`+ New Shortcut`** button identical to S2's (`inspector.rs`
   L650), opening the shared modal via the host service (§5.3);
3. **edit pencil** on `SavedShortcut` rows (pass a real `on_edit` into
   `action_rows`);
4. **delete hidden until edit exists** (interim rule from parent Phase 1.2 —
   kills C2 the safe way, then both appear together in Phase 3);
5. **capability-filtered Power User section** (Type Text / AppleScript rows
   only where the injector implements them).

### 5.2 Named-action target model (decision — new, complements parent §2.4)

The §3 analysis suggests the library, not the binding site, should own
"custom behavior". Options:

| Option | Description | Cost / risk |
|---|---|---|
| **A. Keep payloads disjoint** (status quo) | Shortcuts = chord/app; Power User = direct key bindings. | Zero cost; keeps the two-silo confusion and the un-reusable Power User actions. |
| **B. Extend `ActionTarget` with `Text(String)`** (recommended candidate) | `SavedShortcut` can carry a typed-text target; the composer gains a third mode ("Type text") next to Shortcut/Application; Power User rows keep direct-binding for one-key use, and the library becomes the only place to build a *reusable* macro-ish action. | Wire + config change: append-only enum variant, `PROTOCOL_VERSION` bump + `wire_format` test (ipc rule); agent resolves the new target via existing `Effect::Text`; inherits the macOS-only `post_unicode` gap (§2.4) for shortcut targets too; `SCHEMA_VERSION` migration not required for append but old agents must reject-or-ignore gracefully. |
| **C. Full macro model** (`Text` + `Script` + `Workflow` targets) | Shortcuts become named action objects. | Largest surface; effectively re-homes Power User; decide only if B proves insufficient. |

Phases 1–4 proceed unchanged under any option; Phase 5 implements the choice
together with the parent's R1–R3 model decision (the two interact: both change
what a `SavedShortcut` binding *is*).

### 5.3 Shared components (adopted from parent §2.1–2.3, Keys-first)

- `ui/action_library.rs` — one row/section/search/affordance builder;
  surfaces become adapters supplying a commit closure
  (`commit_keyboard_binding` for S4 vs `commit_binding` for devices).
- Host service on `AppState` (or a root-level view):
  `open_shortcut_builder(entry)` / `close_shortcut_builder()`; modal renders
  once at app root. Deletes the two copy-pasted wirings and gives S4/S5 the
  feature for free.
- `PowerUserEditorState` extracted from `FunctionRowView` + `editors.rs` cards
  moved to `ui/power_user.rs`, with a `capabilities() -> &'static
  [PowerUserKind]` filter (macOS: all four; Linux/Windows: Shell + Workflow
  today, Type Text as soon as `inject` implements it).
- `action_icon_path` dedup (parent D5) stays in `openlogi-desktop` unless the
  overlay needs it — then it goes to `openlogi-ui` (gui rule: overlay links
  `openlogi-ui`, never `openlogi-desktop`).

### 5.4 Composer disambiguation (cheap, ship early)

- Rename the chord field label to make the semantics unmistakable (e.g.
  "Key combination" with placeholder `Cmd+Shift+P`) and keep Power User's
  field "Text to type…" — both wrapped in `tr!`;
- standardize the `…` ellipsis across rows/headings/placeholders;
- chord recorder (parent backlog P1) replaces toggle-buttons-plus-typed-chord
  and removes most of the residual confusion — schedule it as the first
  Phase 6 item.

## 6. Phased execution (Keys-first re-sequencing of parent §3)

Each phase independently shippable; sizes S/M/L relative. Verification per
phase = affected-package tier of the local gate (`AGENTS.md`), widened by
`cargo tree --invert` since `openlogi-desktop`/`openlogi-core` fan out widely.

### Phase 0 — Characterization & audit (S)
- Tests pinning: affordance matrix of `action_rows_matching` per flags;
  `commit_custom_shortcut` **non-propagation** documented as a failing-today
  test (parent R1); `wire_format` snapshot baseline.
- Audit agent-side `SavedShortcut` resolution (embedded copy vs. re-resolve
  by id) — feeds §5.2 and parent §2.4.
- Inventory hardcoded strings in `shortcut_builder.rs`, `action_composer.rs`,
  `editors.rs`, `panel_action_rows`; diff against `locales/en.yml`.
- **Gate:** tests only, no behavior change.

### Phase 1 — Keys-tab quick wins (S, no architecture change)
1. Search input in `config_panel` (reuse `picker.search` pattern from S2).
2. Hide the delete X on `SavedShortcut` rows when `on_edit` is `None`
   (fixes C2 on S4/S5 immediately).
3. Capability-gate Power User rows: macOS shows all four; Linux/Windows show
   Shell + Workflow (cfg-gated list; cross-lint per `.claude/rules/cross-platform.md`).
4. i18n wrap of every string in §2.3; keys added at the same position in
   **all** `locales/*.yml`; ellipsis standardized;
   `cargo test -p openlogi-desktop i18n` green.
- **Acceptance:** no destructive-only affordance anywhere; no hardcoded English
  in the touched files; Linux/Windows users cannot bind a no-op action.

### Phase 2 — Extract `ActionLibrary` (M)
- Build parent §2.1 from the mouse inline variant, but migrate **S4 first**
  (it is the motivating surface and the thinnest), then S2 → S1 → S5 → S3.
- **Acceptance:** `rg "fn action_rows"` finds exactly one definition; S4 keeps
  Phase 1 behavior through the new component.

### Phase 3 — `ShortcutBuilder` host service; create/edit on Keys (M)
- Parent §2.2: modal to app root; delete both per-view copies; wire
  `+ New Shortcut` and the pencil into the Keys panel (and the ring editor,
  cheaply, while the service is being wired).
- Pencil + X appear together on S4/S5 (Phase 1's hide-rule retires).
- **Acceptance:** one open/save/close wiring in the workspace; a keyboard-only
  user can create, edit, and assign a shortcut without touching a mouse
  surface.

### Phase 4 — Power User consistency (M)
- Parent §2.3: `PowerUserEditorState` + `ui/power_user.rs`; the Keys panel
  renders through it; capability filter is one function, not scattered cfg.
- Delete-confirm popover with "Used by N controls" (read-only reference sweep
  helper on `AppState`, parent Phase 1.3 promoted here since delete is now
  live on S4).
- **Deferred pending Q3:** exposing Power User on mouse/ring/K270.

### Phase 5 — Data integrity + target model (M–L; needs Q1+Q2 decisions)
- Parent §2.4 option A or B for `SavedShortcut` propagation.
- §5.2 option B if chosen: `ActionTarget::Text` + composer mode + agent
  lowering; `PROTOCOL_VERSION` bump and `cargo test -p openlogi-ipc --test
  wire_format`; migration notes; deleted-id fallback (`Action::None` +
  warning) and stale-binding state in rows.
- If Type Text targets ship, the Linux/Windows injector gap (§2.4) becomes a
  hard prerequisite: implement unicode text injection for evdev/uinput and
  SendInput, or keep those targets capability-hidden.

### Phase 6 — Backlog (small PRs, parent §4 trimmed to what fits)
P1 chord recorder (also resolves §3/§5.4); P2 per-step workflow editing +
reorder; P2 undo toast after delete; P3 monospace growing textarea for
AppleScript/Shell; P3 empty-state CTA in the Custom Shortcuts section;
P3 duplicate-name warning; P3 accessibility pass (aria labels on
pencil/X, search labelling).

## 7. Constraints (inherited from parent §5, restated)

- **IPC** (`.claude/rules/ipc-protocol.md`): `Action`, `ActionTarget`,
  `CustomShortcutEntry` ride the wire — Phase 5 = append-only +
  `PROTOCOL_VERSION` bump + `wire_format` green.
- **i18n** (`.claude/rules/i18n.md`): new keys in **all** locales, same
  position; `cargo test -p openlogi-desktop i18n`.
- **GUI** (`.claude/rules/gui.md`): overlay-shared pieces go to `openlogi-ui`.
- **Cross-platform** (`.claude/rules/cross-platform.md`): every new cfg block
  (capability gating, AppleScript) → cross-lint or hand-audit Linux/Windows.
- **Verification**: affected-package tier per phase; Phase 5 adds the ipc
  tests and hardware-verification notes ("not runtime-tested on hardware"
  where true).

## 8. Open questions for the maintainer

1. **Q1 (inherited):** `SavedShortcut` resolve-by-id (A) vs.
   propagate-on-write (B)? Phase 0's agent-resolution audit decides.
2. **Q2 (new, §5.2):** extend `ActionTarget` with a `Text` payload so saved
   shortcuts can carry typed text (B), or keep shortcuts chord/app-only (A)?
3. **Q3 (inherited):** should Power User actions be assignable from mice/ring/
   K270 at all, or is keyboard-only deliberate? This plan defers it.
4. **Q4 (new, §2.4):** Linux/Windows "Type Text": implement unicode injection
   in `openlogi-inject`, or keep the row capability-hidden until someone does?
5. **Q5 (inherited):** delete UX — confirm popover, undo toast, or both?
