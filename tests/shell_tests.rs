use predictterm::shell::environment::EnvironmentBuilder;
use predictterm::shell::health::ShellHealthChecker;
use predictterm::shell::ShellResolver;

#[test]
fn test_shell_resolver_default() {
    let shell = ShellResolver::get_best_shell(None);
    assert!(!shell.name.is_empty());
    assert!(!shell.path.to_string_lossy().is_empty());
}

#[test]
fn test_environment_builder_contains_essentials() {
    let envs = EnvironmentBuilder::build_shell_environment(None);
    assert!(envs.contains_key("PATH"));
    
    #[cfg(windows)]
    {
        assert!(envs.contains_key("SystemRoot"));
        assert!(envs.contains_key("WINDIR"));
        assert!(envs.contains_key("SystemDrive"));
        assert!(envs.contains_key("ComSpec"));
    }

    #[cfg(not(windows))]
    {
        assert!(envs.contains_key("TERM"));
    }
}

#[test]
fn test_shell_health_checker_on_best_shell() {
    let best_shell = ShellResolver::get_best_shell(None);
    if best_shell.is_available {
        let health = ShellHealthChecker::check(&best_shell);
        assert!(health.is_healthy(), "Best shell should pass health check probe: {:?}", health);
    }
}
