pub mod linux;
pub mod macos;
pub mod windows;

pub trait PlatformIntegration {
    fn platform_name(&self) -> &'static str;
    fn default_font_family(&self) -> &'static str;
    fn is_dark_mode_preferred(&self) -> bool;
}

pub fn get_platform() -> Box<dyn PlatformIntegration> {
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsPlatform)
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOsPlatform)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Box::new(linux::LinuxPlatform)
    }
}
