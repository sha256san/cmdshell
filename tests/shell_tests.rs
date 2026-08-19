use predictterm::shell::windows::{ensure_essential_windows_env, get_system_root};
use predictterm::shell::ShellResolver;
use std::collections::HashMap;

#[test]
fn test_shell_resolver_default() {
    let (name, path) = ShellResolver::get_default_shell(None);
    assert!(!name.is_empty());
    assert!(!path.to_string_lossy().is_empty());
}

#[test]
fn test_windows_essential_env_injection() {
    let mut map = HashMap::new();
    ensure_essential_windows_env(&mut |k, v| {
        map.insert(k.to_string(), v.to_string());
    });

    assert!(map.contains_key("SystemRoot"));
    assert!(map.contains_key("WINDIR"));
    assert!(map.contains_key("SystemDrive"));
    assert!(map.contains_key("ComSpec"));
    assert!(map.contains_key("PATH"));
}

#[test]
fn test_get_system_root() {
    let root = get_system_root();
    assert!(!root.to_string_lossy().is_empty());
}
