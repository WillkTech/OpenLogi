//! The settings app's GPUI [`AssetSource`].
//!
//! Composes the ring glyphs every frontend shares ([`openlogi_ui::action_icons`])
//! with the two sources only this app draws from: its embedded logo, and
//! gpui-component's bundled lucide set behind `IconName`. The overlay links
//! neither, which is why they live here and not in the shared crate.

use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};
use openlogi_ui::action_icons::ActionIcons;

/// Asset path [`AppAssets`] resolves to the embedded app logo.
pub const LOGO: &str = "openlogi.png";

/// Asset path [`AppAssets`] resolves to the K270 media keys vector diagram.
pub const K270_MEDIAKEYS: &str = "k270-mediakeys.svg";

/// The 1024×1024 app icon, embedded into the binary.
const LOGO_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../design/icon/openlogi.png"
));

/// The K270 media keys SVG artwork, embedded into the binary.
const K270_MEDIAKEYS_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../design/k270-mediakeys.svg"
));

/// GPUI asset source: the app logo, the shared ring glyphs, then
/// gpui-component's bundled icons for everything else.
pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path == LOGO {
            return Ok(Some(Cow::Borrowed(LOGO_BYTES)));
        }
        if path == K270_MEDIAKEYS {
            return Ok(Some(Cow::Borrowed(K270_MEDIAKEYS_BYTES)));
        }
        if let Some(bytes) = ActionIcons.load(path)? {
            return Ok(Some(bytes));
        }
        gpui_component_assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        gpui_component_assets::Assets.list(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_k270_mediakeys_svg() {
        let assets = AppAssets;
        let bytes = assets
            .load(K270_MEDIAKEYS)
            .expect("load result")
            .expect("found");
        assert!(bytes.starts_with(b"<svg"));
        assert!(bytes.ends_with(b"</svg>"));
        // Confirm currentColor styling
        let text = std::str::from_utf8(&bytes).expect("utf8");
        assert!(text.contains("stroke=\"currentColor\""));
        assert!(!text.contains("#222023"));
    }
}
