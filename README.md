# PredictTerm ⚡

> **Intelligent, GPU-accelerated, predictive terminal emulator implemented in Rust with modular decoupled architecture.**

---

## 🌟 Key Features

* **⚡ GPU & High Performance Architecture**: Cleanly decoupled layers across UI, Prediction Engine, and PTY Terminal Backend.
* **🔮 Multi-Source Intelligent Prediction**:
  * **Command Provider**: Fast binary search across `$PATH` and common built-in tools.
  * **History Provider**: SQLite-backed persistent command database with frequency and recency weighting.
  * **Git Provider**: Dynamic context-aware suggestions for branches, remotes, and subcommands.
  * **Project Provider**: Automatic project detection (`Rust`, `Node.js`, `Python`, `Go`, `CMake`, `Make`) with contextual build/test scripts.
  * **Option & Flag Provider**: Autocompletion for command-line flags and arguments.
  * **Filesystem Provider**: Fuzzy path resolution and directory traversal.
* **👻 Ghost Text**: Subdued inline text preview that can be accepted with `Tab` or right-arrow.
* **🛡️ Security & Safety Filter**:
  * Intercepts dangerous / destructive commands (`rm -rf /`, `git reset --hard`, `mkfs`, fork bombs).
  * Automatically redacts API keys, secrets, and bearer tokens before SQLite history persistence.
* **🎨 Themes & Aesthetics**: Pre-configured with *Tokyo Night* and *Catppuccin Mocha* palettes.

---

## 📥 Installation

### 🐧 Linux & 🍎 macOS (One-line Quick Install)

Install PredictTerm instantly via curl or wget:

```bash
# Using curl
curl -fsSL https://raw.githubusercontent.com/sha256san/cmdshell/main/install.sh | bash

# Or using wget
wget -qO- https://raw.githubusercontent.com/sha256san/cmdshell/main/install.sh | bash
```

### 🪟 Windows

1. Download `predictterm-windows-x86_64.zip` from the [Latest Releases](https://github.com/sha256san/cmdshell/releases).
2. Extract the archive and add the directory to your system `PATH` (or double-click `predictterm.exe`).

### 🦀 Using Cargo

```bash
cargo install --git https://github.com/sha256san/cmdshell.git
```

---

## 🚀 Quick Start

### Basic Commands

```bash
# Launch the terminal
predictterm run

# Run environment health check and shell diagnosis
predictterm doctor

# Inspect active configuration
predictterm config

# View command usage statistics
predictterm stats

# Inspect command history
predictterm history
```

### Development & Testing

```bash
# Run all automated unit and integration test suites
cargo test

# Launch in dev mode
cargo run -- run
```

---

## 📂 Project Architecture

```text
predictterm/
├── Cargo.toml
├── install.sh               # One-line installer for Linux & macOS
├── src/
│   ├── main.rs              # CLI entrypoint & subcommands
│   ├── lib.rs               # Library root exposing all subsystems
│   ├── app/                 # State management & event dispatching
│   ├── config/              # TOML settings & color themes
│   ├── database/            # SQLite history storage engine
│   ├── predictor/           # Context builder, ranking engine, cache
│   ├── providers/           # Prediction candidate providers
│   ├── safety/              # Dangerous command detector & secret sanitizer
│   ├── shell/               # Windows & Unix shell resolver & environment manager
│   ├── terminal/            # PTY backend, ANSI/VT parser, grid & cell model
│   └── ui/                  # UI view models, ghost text, suggestions, dialogs
└── tests/
    ├── ansi_tests.rs
    ├── predictor_tests.rs
    ├── safety_tests.rs
    ├── shell_tests.rs
    └── terminal_tests.rs
```

---

## 📜 License

MIT OR Apache-2.0
