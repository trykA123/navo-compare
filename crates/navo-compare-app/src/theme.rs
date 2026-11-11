//! Maps the window's appearance to a token mode (NS19); the shell follows the system theme with it.

use gpui_kit::WindowAppearance;
use navo_ui::tokens::Mode;

/// Dark and vibrant dark give `Mode::Dark`; the two light appearances give `Mode::Light`.
pub fn mode_of(appearance: WindowAppearance) -> Mode {
    match appearance {
        WindowAppearance::Dark | WindowAppearance::VibrantDark => Mode::Dark,
        WindowAppearance::Light | WindowAppearance::VibrantLight => Mode::Light,
    }
}
