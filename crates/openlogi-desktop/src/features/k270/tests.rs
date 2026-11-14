use openlogi_core::binding::{Action, ApplicationTarget};
use openlogi_core::device::k270::{self, K270MediaKey};

use super::geometry::{K270_HOTSPOTS, hotspot_for_key};

#[test]
fn k270_media_key_defaults_match_specification() {
    assert_eq!(K270MediaKey::PlayPause.default_action(), Action::PlayPause);
    assert_eq!(K270MediaKey::Mute.default_action(), Action::MuteVolume);
    assert_eq!(
        K270MediaKey::VolumeDown.default_action(),
        Action::VolumeDown
    );
    assert_eq!(K270MediaKey::VolumeUp.default_action(), Action::VolumeUp);
    assert_eq!(
        K270MediaKey::Home.default_action(),
        Action::OpenApplication(ApplicationTarget::new("https://", "Home").expect("valid target"))
    );
    assert_eq!(
        K270MediaKey::Email.default_action(),
        Action::OpenApplication(ApplicationTarget::new("mailto:", "Email").expect("valid target"))
    );
    assert_eq!(K270MediaKey::Power.default_action(), Action::Sleep);
    assert_eq!(
        K270MediaKey::Calculator.default_action(),
        Action::OpenApplication(
            ApplicationTarget::new("/System/Applications/Calculator.app", "Calculator")
                .expect("valid target")
        )
    );
}

#[test]
fn all_eight_hotspots_have_corresponding_media_key() {
    for spot in &K270_HOTSPOTS {
        assert!(hotspot_for_key(spot.key).is_some());
    }
}

#[test]
fn k270_hardware_classifications_and_suppressibility() {
    // Power is system-level, unsuppressible by user-space apps
    assert!(!K270MediaKey::Power.is_suppressible());
    assert!(
        K270MediaKey::Power
            .hardware_event_type()
            .contains("System Control")
    );

    // Media keys on macOS are consumer-control handled by mediaremoted
    assert!(!K270MediaKey::PlayPause.is_suppressible());
    assert!(!K270MediaKey::Mute.is_suppressible());
    assert!(!K270MediaKey::VolumeDown.is_suppressible());
    assert!(!K270MediaKey::VolumeUp.is_suppressible());

    // Application shortcuts are interceptible
    assert!(K270MediaKey::Home.is_suppressible());
    assert!(K270MediaKey::Email.is_suppressible());
    assert!(K270MediaKey::Calculator.is_suppressible());
}

#[test]
fn k270_identification_matches_wpid_and_names() {
    assert!(k270::is_k270(None, Some("Wireless Keyboard K270"), None));
    assert!(k270::is_k270(None, Some("Logitech K270"), None));
    assert!(k270::is_k270(None, None, Some(k270::UNIFYING_WPID)));
    assert!(k270::is_k270(None, None, Some(k270::ALT_WPID)));

    // Non-K270 keyboards and devices are rejected
    assert!(!k270::is_k270(None, Some("MX Keys"), None));
    assert!(!k270::is_k270(None, Some("Craft Advanced Keyboard"), None));
    assert!(!k270::is_k270(None, Some("MX Master 3S"), None));
    assert!(!k270::is_k270(None, None, Some(0x4082))); // MX Master 3S
}
