//! Regression tests for OS-hook state and dispatch policy.

use super::*;
use openlogi_core::binding::{GESTURE_SWIPE_THRESHOLD, LongPressBinding};

fn token(id: u64, button: ButtonId) -> PressToken {
    PressToken::hook_for_test(id, button)
}

// The mid-swipe gate itself is unit-tested on `SwipeAccumulator` in
// `openlogi-core`; these cover only what `HoldState` adds on top — tagging a
// commit with the exact press and held button, and matching the release.

#[test]
fn accumulate_tags_a_committed_swipe_with_the_held_press() {
    let mut hold = HoldState::default();
    let press = token(1, ButtonId::Back);
    hold.begin(ButtonId::Back, press.clone());
    hold.swipe.backdate_hold_for_test();

    assert_eq!(
        hold.accumulate(GESTURE_SWIPE_THRESHOLD + 10, 0),
        Some((press.clone(), ButtonId::Back, GestureDirection::Right))
    );
    assert_eq!(
        hold.accumulate(50, 0),
        None,
        "commits at most once per hold"
    );
    assert_eq!(hold.end(ButtonId::Back), Some((press, false)));
}

#[test]
fn a_same_button_repress_restarts_the_stale_hold() {
    let mut hold = HoldState::default();
    let old = token(1, ButtonId::Back);
    assert!(matches!(
        hold.prepare_begin(ButtonId::Back),
        HoldAdmission::Begin
    ));
    hold.begin(ButtonId::Back, old);

    let replacement = token(2, ButtonId::Back);
    assert!(
        matches!(hold.prepare_begin(ButtonId::Back), HoldAdmission::Begin),
        "a same-button re-press is proof of a lost release"
    );
    hold.begin(ButtonId::Back, replacement.clone());
    hold.swipe.backdate_hold_for_test();
    assert_eq!(
        hold.accumulate(GESTURE_SWIPE_THRESHOLD + 10, 0),
        Some((replacement, ButtonId::Back, GestureDirection::Right))
    );
}

#[test]
fn an_aged_hold_yields_to_a_new_buttons_press() {
    let mut hold = HoldState::default();
    hold.begin(ButtonId::Back, token(1, ButtonId::Back));
    hold.backdate_for_test();

    let replacement = token(2, ButtonId::Forward);
    let HoldAdmission::Replace(stale) = hold.prepare_begin(ButtonId::Forward) else {
        panic!("an aged hold must yield to a new press");
    };
    assert_eq!(stale, token(1, ButtonId::Back));
    hold.begin(ButtonId::Forward, replacement.clone());
    hold.swipe.backdate_hold_for_test();
    assert_eq!(
        hold.accumulate(GESTURE_SWIPE_THRESHOLD + 10, 0),
        Some((replacement, ButtonId::Forward, GestureDirection::Right))
    );
}

#[test]
fn begin_is_first_wins_while_a_hold_is_active() {
    let mut hold = HoldState::default();
    let first = token(1, ButtonId::Back);
    hold.begin(ButtonId::Back, first.clone());
    hold.swipe.backdate_hold_for_test();
    assert!(
        matches!(hold.prepare_begin(ButtonId::Forward), HoldAdmission::Refuse),
        "a second press must not hijack the active hold"
    );

    assert_eq!(
        hold.accumulate(GESTURE_SWIPE_THRESHOLD + 10, 0),
        Some((first.clone(), ButtonId::Back, GestureDirection::Right))
    );
    assert_eq!(hold.end(ButtonId::Forward), None);
    assert_eq!(hold.end(ButtonId::Back), Some((first, false)));
}

#[test]
fn end_matches_the_held_button_and_returns_its_token() {
    let mut hold = HoldState::default();
    let press = token(1, ButtonId::Back);
    hold.begin(ButtonId::Back, press.clone());
    assert_eq!(hold.end(ButtonId::Forward), None);
    assert_eq!(hold.end(ButtonId::Back), Some((press, true)));
}

#[test]
fn resolve_gesture_click_prefers_explicit_then_falls_back_to_default() {
    let gestures = BTreeMap::from([(
        ButtonId::Back,
        BTreeMap::from([(GestureDirection::Click, Action::Copy)]),
    )]);
    assert_eq!(
        resolve_gesture_click(&gestures, ButtonId::Back),
        Action::Copy
    );

    let off = BTreeMap::from([(
        ButtonId::Back,
        BTreeMap::from([(GestureDirection::Click, Action::None)]),
    )]);
    assert_eq!(resolve_gesture_click(&off, ButtonId::Back), Action::None);
}

#[test]
fn fail_open_press_pairs_release() {
    let mut fail_open = HashSet::new();
    assert_eq!(
        remapped_press_disposition(ButtonId::Back, true, &mut fail_open),
        EventDisposition::Suppress
    );
    assert_eq!(
        remapped_release_disposition(ButtonId::Back, &mut fail_open),
        EventDisposition::Suppress
    );
    assert_eq!(
        remapped_press_disposition(ButtonId::Forward, false, &mut fail_open),
        EventDisposition::PassThrough
    );
    assert_eq!(
        remapped_release_disposition(ButtonId::Forward, &mut fail_open),
        EventDisposition::PassThrough
    );
    assert_eq!(
        remapped_release_disposition(ButtonId::Forward, &mut fail_open),
        EventDisposition::Suppress
    );
}

#[test]
fn rejected_key_edges_fail_open() {
    assert_eq!(queued_event_disposition(true), EventDisposition::Suppress);
    assert_eq!(
        queued_event_disposition(false),
        EventDisposition::PassThrough
    );
}

#[test]
fn scroll_interception_uses_the_button_source_safety_policy_and_skips_trackpads() {
    let logitech = EventDevice {
        vendor_id: Some(openlogi_hook::LOGITECH_VENDOR_ID),
        product_name: Some("Logitech MX Master".into()),
        ..EventDevice::default()
    };
    let trackpad = EventDevice {
        product_name: Some("Magic Trackpad".into()),
        ..EventDevice::default()
    };

    assert!(scroll_source_may_intercept(false, Some(&logitech)));
    assert!(!scroll_source_may_intercept(true, Some(&logitech)));
    assert!(!scroll_source_may_intercept(false, Some(&trackpad)));
    assert_eq!(
        scroll_source_may_intercept(false, None),
        !cfg!(target_os = "macos"),
        "only macOS requires callback-time device attribution"
    );
}

#[test]
fn rebound_horizontal_wheel_maps_to_thumbwheel_directions() {
    let maps = HookMaps {
        bindings: BTreeMap::from([
            (ButtonId::ThumbwheelScrollUp, Action::NextTab.into()),
            (ButtonId::ThumbwheelScrollDown, Action::PrevTab.into()),
        ]),
        gestures: BTreeMap::new(),
    };
    assert_eq!(
        rebound_thumbwheel_action(&maps, 1.0),
        Some((ButtonId::ThumbwheelScrollDown, Action::PrevTab))
    );
    assert_eq!(
        rebound_thumbwheel_action(&maps, -1.0),
        Some((ButtonId::ThumbwheelScrollUp, Action::NextTab))
    );
    assert_eq!(rebound_thumbwheel_action(&maps, 0.0), None);
}

#[test]
fn native_thumbwheel_scroll_stays_os_native() {
    let maps = HookMaps {
        bindings: BTreeMap::from([
            (
                ButtonId::ThumbwheelScrollUp,
                default_binding(ButtonId::ThumbwheelScrollUp).into(),
            ),
            (
                ButtonId::ThumbwheelScrollDown,
                default_binding(ButtonId::ThumbwheelScrollDown).into(),
            ),
        ]),
        gestures: BTreeMap::new(),
    };
    assert_eq!(rebound_thumbwheel_action(&maps, 1.0), None);
    assert_eq!(rebound_thumbwheel_action(&maps, -1.0), None);
}

#[test]
fn long_press_never_passes_through_as_a_native_click() {
    let binding = Binding::LongPress(LongPressBinding::new(
        default_binding(ButtonId::Back),
        Action::MissionControl,
    ));
    assert!(!binding_is_native_click(ButtonId::Back, &binding));
}

#[test]
fn resolve_gesture_click_falls_back_when_click_is_absent() {
    let no_click = BTreeMap::from([(
        ButtonId::Back,
        BTreeMap::from([(GestureDirection::Up, Action::Copy)]),
    )]);
    assert_eq!(
        resolve_gesture_click(&no_click, ButtonId::Back),
        default_binding(ButtonId::Back)
    );

    let empty = BTreeMap::new();
    assert_eq!(
        resolve_gesture_click(&empty, ButtonId::Forward),
        default_binding(ButtonId::Forward)
    );
}

#[test]
fn handle_key_suppresses_and_executes_remapped_media_key() {
    use openlogi_core::binding::KeyCombo;
    use openlogi_core::device::K270MediaKey;

    let (tx, rx) = std::sync::mpsc::sync_channel(16);
    let (ring_tx, _ring_rx) = tokio::sync::mpsc::unbounded_channel();
    let runtime = crate::runtime::ActionRuntime::new(
        Arc::new(RwLock::new(crate::DpiCycles::default())),
        Arc::new(RwLock::new(None)),
        openlogi_hid::ChannelRegistry::default(),
        crate::receiver_access::ReceiverAccess::default(),
        ring_tx,
    )
    .expect("test action runtime");
    let dispatcher = runtime.dispatcher();

    let calc_trigger = K270MediaKey::Calculator.to_key_trigger();
    let pageup_combo = "pageup".parse::<KeyCombo>().expect("parse pageup");
    let pageup = Action::CustomShortcut(pageup_combo.clone());
    let bindings: SharedKeyboardBindings = Arc::new(RwLock::new(BTreeMap::from([(
        calc_trigger,
        pageup.clone(),
    )])));

    // 1. One-shot action (CustomShortcut): press is suppressed and queued to action worker
    let event_down = openlogi_hook::KeyEvent {
        keycode: 0x0192,
        pressed: true,
        modifiers: openlogi_hook::KeyModifiers::default(),
    };
    let disp = handle_key(event_down, &bindings, &tx, &dispatcher);
    assert_eq!(disp, EventDisposition::Suppress);
    assert_eq!(rx.try_recv(), Ok(pageup));

    // Release of one-shot action passes through without stale held state
    let event_up = openlogi_hook::KeyEvent {
        keycode: 0x0192,
        pressed: false,
        modifiers: openlogi_hook::KeyModifiers::default(),
    };
    assert_eq!(
        handle_key(event_up, &bindings, &tx, &dispatcher),
        EventDisposition::PassThrough
    );

    // 2. Held combo (HoldShortcut): both down and up edges are suppressed by lifecycle
    let email_trigger = K270MediaKey::Email.to_key_trigger();
    let hold_pageup = Action::HoldShortcut(pageup_combo);
    bindings
        .write()
        .expect("write bindings")
        .insert(email_trigger, hold_pageup);

    let email_down = openlogi_hook::KeyEvent {
        keycode: 0x018A,
        pressed: true,
        modifiers: openlogi_hook::KeyModifiers::default(),
    };
    assert_eq!(
        handle_key(email_down, &bindings, &tx, &dispatcher),
        EventDisposition::Suppress
    );

    let email_up = openlogi_hook::KeyEvent {
        keycode: 0x018A,
        pressed: false,
        modifiers: openlogi_hook::KeyModifiers::default(),
    };
    assert_eq!(
        handle_key(email_up, &bindings, &tx, &dispatcher),
        EventDisposition::Suppress
    );

    // 3. Unmapped key passes through untouched
    let unmapped_down = openlogi_hook::KeyEvent {
        keycode: 0x0223, // Home
        pressed: true,
        modifiers: openlogi_hook::KeyModifiers::default(),
    };
    assert_eq!(
        handle_key(unmapped_down, &bindings, &tx, &dispatcher),
        EventDisposition::PassThrough
    );
}
