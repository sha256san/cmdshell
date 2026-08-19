use crate::platform::PlatformIntegration;

pub struct LinuxPlatform;

impl PlatformIntegration for LinuxPlatform {
    fn platform_name(&self) -> &'static str {
        "Linux (Wayland/X11 / POSIX PTY)"
    }

    fn default_font_family(&self) -> &'static str {
        "DejaVu Sans Mono, Ubuntu Mono, Liberation Mono, Monospace"
    }

    fn is_dark_mode_preferred(&self) -> bool {
        true
    }
}
