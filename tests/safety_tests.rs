use predictterm::safety::dangerous::{DangerousDetector, RiskLevel};
use predictterm::safety::sanitizer::SecretSanitizer;

#[test]
fn test_destructive_rm_detection() {
    let detector = DangerousDetector::default();

    let verdict = detector.inspect("rm -rf /");
    assert!(verdict.is_dangerous);
    assert_eq!(verdict.risk_level, RiskLevel::Critical);

    let verdict_home = detector.inspect("rm -rf ~");
    assert!(verdict_home.is_dangerous);
    assert_eq!(verdict_home.risk_level, RiskLevel::High);

    let safe_verdict = detector.inspect("rm temp.txt");
    assert!(!safe_verdict.is_dangerous);
}

#[test]
fn test_git_hard_reset_and_force_push() {
    let detector = DangerousDetector::default();

    let verdict_reset = detector.inspect("git reset --hard HEAD~1");
    assert!(verdict_reset.is_dangerous);
    assert_eq!(verdict_reset.risk_level, RiskLevel::High);

    let verdict_force = detector.inspect("git push origin main --force");
    assert!(verdict_force.is_dangerous);
    assert_eq!(verdict_force.risk_level, RiskLevel::High);

    let verdict_safe = detector.inspect("git commit -m 'feat: update'");
    assert!(!verdict_safe.is_dangerous);
}

#[test]
fn test_fork_bomb_detection() {
    let detector = DangerousDetector::default();
    let verdict = detector.inspect(":(){ :|:& };:");
    assert!(verdict.is_dangerous);
    assert_eq!(verdict.risk_level, RiskLevel::Critical);
}

#[test]
fn test_secret_sanitization() {
    let sanitizer = SecretSanitizer::new();

    let input = "export GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz1234567890";
    let sanitized = sanitizer.sanitize(input);
    assert!(!sanitized.contains("ghp_"));
    assert!(sanitized.contains("[REDACTED_SECRET]"));

    let input_auth = "curl -H 'Authorization: Bearer my-secret-token-xyz123' https://api.example.com";
    let sanitized_auth = sanitizer.sanitize(input_auth);
    assert!(!sanitized_auth.contains("my-secret-token-xyz123"));
}
