# Changelog

All notable changes to **PredictTerm** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.3] - 2026-08-19 (Pre-release)

### Added
- **Command-line Installer (`install.sh`)**:
  - Added one-line installer script for Linux & macOS via `curl`/`wget`.
  - Automatically detects OS and architecture (`x86_64`, `arm64`), downloads the latest release binary, and installs to `~/.local/bin`.
  - Updated `README.md` with installation guides for Linux, macOS, and Windows.

### Fixed
- **Windows Shell Spawning & Environment Inheritance**:
  - Prioritized native `cmd.exe` as the default Windows shell to prevent `0xc0000142` (`STATUS_DLL_INIT_FAILED`) CLR initialization errors.
  - Ensured complete environment variable inheritance from `std::env::vars()` into ConPTY subprocesses.

---

## [0.1.2] - 2026-08-19 (Pre-release)

### Fixed
- **Windows Environment Injection & Test Assertion**:
  - Fixed `ensure_essential_windows_env` to unconditionally inject all essential variables (`SystemRoot`, `WINDIR`, `SystemDrive`, `ComSpec`, `PATH`), preventing assertion failures on native Windows runners.
  - Added `-Force` flag to PowerShell `Compress-Archive` in GitHub Actions packaging step to avoid archive creation collision.

---

## [0.1.1] - 2026-08-19 (Pre-release)

### Fixed & Enhanced
- **Windows Shell Resolver (`src/shell/`)**:
  - Implemented secure Windows shell resolution with absolute path discovery (`pwsh.exe`, `powershell.exe`, `cmd.exe`, `bash.exe`).
  - Added essential Windows environment variable enforcement (`SystemRoot`, `WINDIR`, `SystemDrive`, `ComSpec`, `PATH`) to prevent `0xc0000142` (`STATUS_DLL_INIT_FAILED`) DLL initialization crashes.
  - Added multi-tier automatic fallback when primary shell cannot be spawned.
- **Enhanced Diagnostics (`predictterm doctor`)**:
  - Added diagnostic checks for Windows environment variables and available shell binaries.
- **CI/CD Optimization**:
  - Optimized macOS Intel build runner to fast Apple Silicon `macos-14` cross-compilation.
  - Configured GitHub Releases to publish as Pre-release.

---

## [0.1.0] - 2026-08-19

### Added
- **Terminal Core Engine**:
  - Hardware-accelerated terminal cell and grid data structures with scrollback ring buffer.
  - ANSI / VT100 / Xterm escape sequence parser (`vte`) with 24-bit TrueColor and SGR styles.
  - PTY process manager (`portable-pty`) with asynchronous reader thread.
- **Prediction Engine & Candidate Providers**:
  - Multi-factor ranking engine with prefix match and fuzzy scoring.
  - Contextual candidate providers: `CommandProvider`, `HistoryProvider`, `GitProvider`, `ProjectProvider`, `FilesystemProvider`, `OptionProvider`, and `AiProvider`.
  - Inline **Ghost Text** preview for rapid completion with `Tab`.
- **Database & Security**:
  - Embedded SQLite database (`rusqlite`) for tracking command history and usage statistics.
  - Dangerous command interception filter (`rm -rf /`, `git reset --hard`, fork bombs, `mkfs`).
  - Secret sanitizer automatically redacting API keys and tokens from history storage.
- **UI & Presentation Models**:
  - Presentation models for Window, TabBar, TerminalGrid, Suggestions popup, Status bar, and Confirmation dialogs.
  - Curated themes (*Tokyo Night*, *Catppuccin Mocha*).
- **CLI Subcommands**:
  - `predictterm run`, `predictterm doctor`, `predictterm config`, `predictterm history`, `predictterm stats`.
- **Multi-Platform CI/CD**:
  - GitHub Actions matrix workflow compiling Linux, Windows, and macOS (Intel & Apple Silicon) binaries and publishing release assets.