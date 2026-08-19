use crate::platform::PlatformIntegration;

pub struct MacOsPlatform;

impl PlatformIntegration for MacOsPlatform {
    fn platform_name(&self) -> &'static str {
        "macOS (Cocoa / POSIX PTY)"
    }

    fn default_font_family(&self) -> &'static str {
        "Menlo, Monaco, SF Mono, Courier"
    }

    fn is_dark_mode_preferred(&self) -> bool {
        true
    }
}
