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

## 🚀 Quick Start

### Build & Test

```bash
# Run all unit and integration test suites
cargo test

# Launch the terminal
cargo run -- run

# Run environment health check
cargo run -- doctor

# Inspect configuration
cargo run -- config

# View command statistics
cargo run -- stats
```

---

## 📂 Project Architecture

```text
predictterm/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entrypoint & subcommands
│   ├── lib.rs               # Library root exposing all subsystems
│   ├── app/                 # State management & event dispatching
│   ├── config/              # TOML settings & color themes
│   ├── database/            # SQLite history storage engine
│   ├── predictor/           # Context builder, ranking engine, cache
│   ├── providers/           # Prediction candidate providers
│   ├── safety/              # Dangerous command detector & secret sanitizer
│   ├── terminal/            # PTY backend, ANSI/VT parser, grid & cell model
│   └── ui/                  # UI view models, ghost text, suggestions, dialogs
└── tests/
    ├── ansi_tests.rs
    ├── predictor_tests.rs
    ├── safety_tests.rs
    └── terminal_tests.rs
```

---

## 📜 License

MIT OR Apache-2.0
