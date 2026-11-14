//! Geometry and hotspot calculations for the K270 media keys vector diagram.
//!
//! Based on the original SVG viewBox of 1024 × 120. All hotspot bounds are
//! stored as normalized fractions of the viewBox so they scale proportionally
//! with the rendered element size without letterboxing drift or aspect distortion.

use openlogi_core::device::k270::K270MediaKey;

/// Original SVG artwork viewBox dimensions.
pub const K270_VIEWBOX_W: f32 = 1024.0;
/// Original SVG artwork viewBox height.
pub const K270_VIEWBOX_H: f32 = 120.0;
/// Aspect ratio of the K270 media keys diagram (1024 / 120 = 8.5333...).
pub const K270_ASPECT_RATIO: f32 = K270_VIEWBOX_W / K270_VIEWBOX_H;

/// A normalized interaction hotspot for one K270 media key.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct K270Hotspot {
    /// Logical key variant.
    pub key: K270MediaKey,
    /// Left offset as a fraction of total width [0.0..1.0].
    pub x_frac: f32,
    /// Top offset as a fraction of total height [0.0..1.0].
    pub y_frac: f32,
    /// Width as a fraction of total width [0.0..1.0].
    pub w_frac: f32,
    /// Height as a fraction of total height [0.0..1.0].
    pub h_frac: f32,
}

impl K270Hotspot {
    /// Calculate pixel bounds `(left, top, width, height)` for a rendered dimension.
    #[must_use]
    pub fn rect(&self, rendered_w: f32, rendered_h: f32) -> (f32, f32, f32, f32) {
        (
            self.x_frac * rendered_w,
            self.y_frac * rendered_h,
            self.w_frac * rendered_w,
            self.h_frac * rendered_h,
        )
    }

    /// Check if a coordinate `(px, py)` in rendered diagram space falls inside this hotspot.
    #[must_use]
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "coordinate lookup and hit-testing API for K270 diagram"
        )
    )]
    pub fn contains(&self, rendered_w: f32, rendered_h: f32, px: f32, py: f32) -> bool {
        let (left, top, w, h) = self.rect(rendered_w, rendered_h);
        px >= left && px <= (left + w) && py >= top && py <= (top + h)
    }
}

/// The eight key interaction regions mapped to the 1024 × 120 viewBox coordinates:
/// - Key 1: x in [60.0, 171.0], y in [35.0, 77.0] -> Play/Pause
/// - Key 2: x in [172.0, 282.0], y in [35.0, 77.0] -> Mute
/// - Key 3: x in [283.0, 394.0], y in [35.0, 79.0] -> Volume Down
/// - Key 4: x in [394.0, 511.5], y in [35.0, 79.0] -> Volume Up
/// - Key 5: x in [511.5, 629.0], y in [35.0, 79.0] -> Home
/// - Key 6: x in [629.0, 740.0], y in [35.0, 79.0] -> Email
/// - Key 7: x in [741.0, 851.0], y in [35.0, 77.0] -> Power
/// - Key 8: x in [852.0, 963.0], y in [35.0, 77.0] -> Calculator
pub const K270_HOTSPOTS: [K270Hotspot; 8] = [
    K270Hotspot {
        key: K270MediaKey::PlayPause,
        x_frac: 60.0 / K270_VIEWBOX_W,
        y_frac: 35.0 / K270_VIEWBOX_H,
        w_frac: 111.0 / K270_VIEWBOX_W,
        h_frac: 42.0 / K270_VIEWBOX_H,
    },
    K270Hotspot {
        key: K270MediaKey::Mute,
        x_frac: 172.0 / K270_VIEWBOX_W,
        y_frac: 35.0 / K270_VIEWBOX_H,
        w_frac: 110.0 / K270_VIEWBOX_W,
        h_frac: 42.0 / K270_VIEWBOX_H,
    },
    K270Hotspot {
        key: K270MediaKey::VolumeDown,
        x_frac: 283.0 / K270_VIEWBOX_W,
        y_frac: 35.0 / K270_VIEWBOX_H,
        w_frac: 111.0 / K270_VIEWBOX_W,
        h_frac: 44.0 / K270_VIEWBOX_H,
    },
    K270Hotspot {
        key: K270MediaKey::VolumeUp,
        x_frac: 394.0 / K270_VIEWBOX_W,
        y_frac: 35.0 / K270_VIEWBOX_H,
        w_frac: 117.5 / K270_VIEWBOX_W,
        h_frac: 44.0 / K270_VIEWBOX_H,
    },
    K270Hotspot {
        key: K270MediaKey::Home,
        x_frac: 511.5 / K270_VIEWBOX_W,
        y_frac: 35.0 / K270_VIEWBOX_H,
        w_frac: 117.5 / K270_VIEWBOX_W,
        h_frac: 44.0 / K270_VIEWBOX_H,
    },
    K270Hotspot {
        key: K270MediaKey::Email,
        x_frac: 629.0 / K270_VIEWBOX_W,
        y_frac: 35.0 / K270_VIEWBOX_H,
        w_frac: 111.0 / K270_VIEWBOX_W,
        h_frac: 44.0 / K270_VIEWBOX_H,
    },
    K270Hotspot {
        key: K270MediaKey::Power,
        x_frac: 741.0 / K270_VIEWBOX_W,
        y_frac: 35.0 / K270_VIEWBOX_H,
        w_frac: 110.0 / K270_VIEWBOX_W,
        h_frac: 42.0 / K270_VIEWBOX_H,
    },
    K270Hotspot {
        key: K270MediaKey::Calculator,
        x_frac: 852.0 / K270_VIEWBOX_W,
        y_frac: 35.0 / K270_VIEWBOX_H,
        w_frac: 111.0 / K270_VIEWBOX_W,
        h_frac: 42.0 / K270_VIEWBOX_H,
    },
];

/// Find the hotspot for a given key.
#[must_use]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "hotspot lookup API for K270 diagram")
)]
pub fn hotspot_for_key(key: K270MediaKey) -> Option<&'static K270Hotspot> {
    K270_HOTSPOTS.iter().find(|h| h.key == key)
}

/// Hit test given rendered dimensions and pixel coordinates.
#[must_use]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "hit-testing API for K270 diagram")
)]
pub fn hit_test(rendered_w: f32, rendered_h: f32, px: f32, py: f32) -> Option<K270MediaKey> {
    K270_HOTSPOTS
        .iter()
        .find(|h| h.contains(rendered_w, rendered_h, px, py))
        .map(|h| h.key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_eight_controls_are_registered_and_unique() {
        assert_eq!(K270_HOTSPOTS.len(), 8);
        let mut keys = std::collections::HashSet::new();
        for spot in &K270_HOTSPOTS {
            assert!(keys.insert(spot.key), "duplicate key: {:?}", spot.key);
            assert!(spot.x_frac >= 0.0 && spot.x_frac <= 1.0);
            assert!(spot.y_frac >= 0.0 && spot.y_frac <= 1.0);
            assert!(spot.w_frac > 0.0 && spot.w_frac <= 1.0);
            assert!(spot.h_frac > 0.0 && spot.h_frac <= 1.0);
            assert!(spot.x_frac + spot.w_frac <= 1.0);
            assert!(spot.y_frac + spot.h_frac <= 1.0);
        }
    }

    #[test]
    fn hotspots_do_not_overlap() {
        for (i, a) in K270_HOTSPOTS.iter().enumerate() {
            for b in K270_HOTSPOTS.iter().skip(i + 1) {
                // Check horizontal overlap
                let a_right = a.x_frac + a.w_frac;
                let b_right = b.x_frac + b.w_frac;
                let h_overlap = a.x_frac < b_right && a_right > b.x_frac;

                // Check vertical overlap
                let a_bottom = a.y_frac + a.h_frac;
                let b_bottom = b.y_frac + b.h_frac;
                let v_overlap = a.y_frac < b_bottom && a_bottom > b.y_frac;

                assert!(
                    !(h_overlap && v_overlap),
                    "Hotspots {:?} and {:?} overlap! A: [{}, {}], B: [{}, {}]",
                    a.key,
                    b.key,
                    a.x_frac,
                    a_right,
                    b.x_frac,
                    b_right
                );
            }
        }
    }

    #[test]
    fn hit_test_finds_correct_keys_at_center() {
        let (w, h) = (800.0, 800.0 / K270_ASPECT_RATIO);
        for spot in &K270_HOTSPOTS {
            let (left, top, width, height) = spot.rect(w, h);
            let center_x = left + width / 2.0;
            let center_y = top + height / 2.0;
            let hit = hit_test(w, h, center_x, center_y);
            assert_eq!(
                hit,
                Some(spot.key),
                "failed to hit center of {:?}",
                spot.key
            );
        }

        // Test outside hits
        assert_eq!(hit_test(w, h, 0.0, 0.0), None);
        assert_eq!(hit_test(w, h, w + 10.0, h / 2.0), None);
        assert_eq!(hit_test(w, h, w / 2.0, h + 10.0), None);
    }

    #[test]
    fn scaling_preserves_hotspot_alignment() {
        for scale in [0.5, 1.0, 1.5, 2.0] {
            let w = 500.0 * scale;
            let h = w / K270_ASPECT_RATIO;
            for spot in &K270_HOTSPOTS {
                let (left, top, width, height) = spot.rect(w, h);
                let hit = hit_test(w, h, left + width / 2.0, top + height / 2.0);
                assert_eq!(hit, Some(spot.key));
            }
        }
    }
}
