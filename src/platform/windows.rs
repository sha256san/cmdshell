use crate::platform::PlatformIntegration;

pub struct WindowsPlatform;

impl PlatformIntegration for WindowsPlatform {
    fn platform_name(&self) -> &'static str {
        "Windows (Win32 / ConPTY)"
    }

    fn default_font_family(&self) -> &'static str {
        "Cascadia Mono, Consolas, Courier New"
    }

    fn is_dark_mode_preferred(&self) -> bool {
        true
    }
}
