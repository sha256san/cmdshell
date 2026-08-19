# cmdshell v0.1.5 GUI・クロスプラットフォーム化追加計画書

## 1. 目的

`cmdshell` v0.1.5を基準として、プロジェクトの方向性を大きく変更する。

これまでのCLI中心の設計を一旦停止し、

> **Rust + GPUIで動作する、Windows / macOS / Linux対応のネイティブGUIターミナル**

として開発を進める。

CLI機能は当面の開発対象から外す。

最優先するのは、

1. GPUIによるGUI
2. ネイティブアプリケーションとしての動作
3. Windows / macOS / Linux対応
4. ターミナルとしての安定性
5. GPUアクセラレーション
6. コマンド予測変換

である。

---

# 2. プロジェクトの方向性

## 変更前

```text
cmdshell
   │
   ├── CLI
   ├── PTY
   ├── Shell
   └── Prediction
```

## 変更後

```text
                    cmdshell
                       │
                ┌──────▼──────┐
                │     GPUI    │
                │ Application │
                └──────┬──────┘
                       │
                ┌──────▼──────┐
                │ Terminal UI │
                └──────┬──────┘
                       │
                ┌──────▼──────┐
                │ Terminal    │
                │ Core        │
                └──────┬──────┘
                       │
                    PTY
                       │
              ┌────────┼────────┐
              │        │        │
            Shell    Shell    Shell
```

CLIは一旦削除・停止し、GUIアプリケーションをプロジェクトの中心にする。

---

# 3. 最重要目標

最終的に以下を実現する。

```text
Windows
   ↓
cmdshell.exe
   ↓
GPUI Window
   ↓
Terminal
```

```text
macOS
   ↓
cmdshell.app
   ↓
GPUI Window
   ↓
Terminal
```

```text
Linux
   ↓
cmdshell
   ↓
GPUI Window
   ↓
Terminal
```

3つのOSで、基本的に同じUI・同じTerminal Core・同じ予測エンジンを利用する。

---

# 4. Zedのような設計を目標とする

cmdshellはUI設計・アーキテクチャ面でZedを参考にする。

ただし、Zedそのものをコピーするのではなく、

```text
Rust
GPUI
GPU accelerated UI
Cross-platform
Component-oriented architecture
```

という思想を参考にする。

ZedではGPUIを中心に、`gpui_windows`、`gpui_macos`、`gpui_linux`などのプラットフォーム層を分離している。cmdshellでも同様に、**共通UI/CoreとOS固有実装を分離する**。

---

# 5. GUIを最優先する理由

CLIは通常、

```text
cmdshell
```

を既存ターミナルから実行する。

しかしGUIアプリケーションでは、

```text
Explorer
Finder
Application Menu
Desktop
Taskbar
Dock
```

などから直接起動できる。

そのためcmdshellを、

> 「ターミナル上で動くターミナル」

ではなく、

> 「OS上で独立して動くターミナルアプリケーション」

として設計する。

---

# 6. CLIの扱い

現時点ではCLIを機能として維持しない。

## 一旦削除するもの

```text
cmdshell --cli
cmdshell doctor
cmdshell --version
```

などのCLI専用機能。

ただし、将来的に再追加できるよう、Core部分とGUI部分を分離しておく。

---

# 7. GUI専用アプリケーション構成

```text
cmdshell/
├── Cargo.toml
├── assets/
│   ├── icons/
│   └── fonts/
│
├── src/
│   ├── main.rs
│   │
│   ├── app/
│   │   ├── mod.rs
│   │   ├── application.rs
│   │   └── window.rs
│   │
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── terminal_view.rs
│   │   ├── tab_bar.rs
│   │   ├── suggestion_popup.rs
│   │   ├── status_bar.rs
│   │   └── command_palette.rs
│   │
│   ├── terminal/
│   │   ├── mod.rs
│   │   ├── session.rs
│   │   ├── pty.rs
│   │   ├── buffer.rs
│   │   ├── parser.rs
│   │   ├── input.rs
│   │   └── selection.rs
│   │
│   ├── shell/
│   │   ├── mod.rs
│   │   ├── resolver.rs
│   │   ├── health.rs
│   │   ├── environment.rs
│   │   ├── windows.rs
│   │   ├── macos.rs
│   │   └── linux.rs
│   │
│   ├── prediction/
│   │   ├── mod.rs
│   │   ├── engine.rs
│   │   ├── history.rs
│   │   └── ranking.rs
│   │
│   └── platform/
│       ├── mod.rs
│       ├── windows.rs
│       ├── macos.rs
│       └── linux.rs
│
└── tests/
    ├── terminal/
    ├── shell/
    └── prediction/
```

---

# 8. GPUIをプロジェクトの中心にする

GPUIは単なる描画ライブラリとして扱わない。

アプリケーション全体のUI基盤として使用する。

```text
GPUI
├── Application
├── Window
├── View
├── Input
├── Focus
├── Event
├── Keyboard
├── Mouse
├── Clipboard
├── Layout
└── Rendering
```

GPUIの基本構造はApplicationからWindowを開き、Viewを描画する方式である。

---

# 9. GUIアプリケーションの基本構造

```rust
Application::new().run(|cx| {
    cx.open_window(...);
});
```

を基本構造として、

```text
Application
     │
     ▼
Main Window
     │
     ▼
Root View
     │
 ┌───┴────────────┐
 ▼                ▼
Tab Bar        Terminal View
                   │
                   ▼
              Terminal Core
```

とする。

---

# 10. Window

メインウィンドウはGPUIのWindowとして管理する。

初期サイズ:

```text
Width: 1200
Height: 800
```

を推奨する。

最小サイズ:

```text
Width: 640
Height: 400
```

程度とする。

---

# 11. タイトルバー

OS標準タイトルバーを利用するか、Zedのようなカスタムタイトルバーを実装するかは、Phase 1ではOS標準を優先する。

理由:

* Windows対応を安定させる
* macOSのTraffic Lightとの互換性
* LinuxのWindow Managerとの互換性
* ドラッグ・リサイズ問題を減らす

GUIが安定した後、カスタムタイトルバーを検討する。

---

# 12. 3OS共通UI

以下は完全に共通化する。

```text
TerminalView
TabBar
SuggestionPopup
StatusBar
CommandPalette
Settings
Theme
Prediction
TerminalBuffer
```

---

# 13. OS固有実装

以下だけをOS固有コードにする。

## Windows

```text
src/platform/windows.rs
src/shell/windows.rs
```

担当:

* ConPTY
* Windows Shell
* PowerShell
* CMD
* Windows path
* Windows clipboard integration
* Windows application packaging

---

## macOS

```text
src/platform/macos.rs
src/shell/macos.rs
```

担当:

* PTY
* zsh
* bash
* fish
* macOS application bundle
* macOS clipboard
* macOS specific shortcuts

---

## Linux

```text
src/platform/linux.rs
src/shell/linux.rs
```

担当:

* PTY
* bash
* zsh
* fish
* Wayland
* X11
* Linux clipboard
* desktop integration

---

# 14. Terminal CoreはOS非依存にする

理想:

```text
Terminal Core
     │
     ├── Windows
     │     └── ConPTY
     │
     ├── macOS
     │     └── Unix PTY
     │
     └── Linux
           └── Unix PTY
```

UI側から、

```rust
terminal.write(input);
terminal.resize(cols, rows);
terminal.read();
```

のように利用できるAPIにする。

---

# 15. PTY抽象化

```rust
pub trait PtyBackend {
    fn spawn(&mut self, shell: ShellConfig) -> Result<()>;
    fn write(&mut self, data: &[u8]) -> Result<()>;
    fn resize(&mut self, cols: u16, rows: u16) -> Result<()>;
    fn kill(&mut self) -> Result<()>;
}
```

OS固有実装:

```text
Windows
    └── ConPTYBackend

macOS
    └── UnixPtyBackend

Linux
    └── UnixPtyBackend
```

---

# 16. Shell Resolver

OSごとに標準Shellが異なる。

## Windows

```text
PowerShell 7
PowerShell 5.1
CMD
Git Bash
WSL
```

## macOS

```text
zsh
bash
fish
```

## Linux

```text
bash
zsh
fish
dash
```

Shell Resolverが環境を調べて適切なShellを選択する。

---

# 17. デフォルトShell

## Windows

```text
PowerShell 7
↓
PowerShell 5.1
↓
CMD
```

## macOS

```text
zsh
↓
bash
```

## Linux

```text
ユーザーの$SHELL
↓
bash
↓
zsh
```

---

# 18. Shell Health Check

v0.1.3で発生したPowerShell `0xc0000142` 問題への対策として、Shell Health Checkを共通機能にする。

```text
Shell Discovery
      ↓
Health Check
      ↓
PASS
      ↓
PTY
```

失敗:

```text
PowerShell 7
    ↓
FAIL
    ↓
PowerShell 5.1
    ↓
PASS
```

とする。

---

# 19. Terminal Rendering

Terminal Viewは、

```text
TerminalBuffer
      ↓
Visible Cells
      ↓
Text Layout
      ↓
GPUI
      ↓
GPU
```

という構造にする。

---

# 20. GPUアクセラレーション

大量の文字を表示してもGUIが固まらないようにする。

特に、

```text
cargo build
docker logs
git log
find /
cat large_file
```

などを実行した際の描画性能を重視する。

---

# 21. TerminalBuffer

```rust
pub struct TerminalBuffer {
    rows: Vec<Row>,
    cursor: Cursor,
    selection: Option<Selection>,
    scrollback: Scrollback,
}
```

各Cell:

```rust
pub struct Cell {
    pub character: char,
    pub foreground: Color,
    pub background: Color,
    pub attributes: Attributes,
}
```

---

# 22. VT/ANSI Parser

対応:

* ANSI Color
* 256 Color
* True Color
* Cursor Movement
* Clear Screen
* Clear Line
* Scroll
* Alternate Screen
* Cursor Visibility
* Bold
* Italic
* Underline
* Inverse

を実装する。

---

# 23. 入力

GUIイベント:

```text
GPUI Keyboard Event
        ↓
Terminal Input Translator
        ↓
PTY
```

対応:

```text
文字入力
Enter
Tab
Backspace
Delete
Arrow
Home
End
PageUp
PageDown
Ctrl+C
Ctrl+D
Ctrl+Z
```

---

# 24. 日本語入力

クロスプラットフォーム対応で重要な機能。

```text
Windows
IME

macOS
IME

Linux
IBus / Fcitx
```

を考慮する。

特に日本語IME使用時に、

```text
変換中の文字
確定文字
Backspace
Enter
```

がターミナル入力と競合しないようにする。

---

# 25. Clipboard

共通API:

```text
copy()
paste()
```

OS固有の実装をGPUI / Platform層へ隠蔽する。

---

# 26. テキスト選択

以下を実装する。

```text
クリック
ドラッグ
Shift + Arrow
Ctrl/Cmd + A
```

OSごとのショートカット差:

```text
Windows/Linux
Ctrl

macOS
Cmd
```

をKeymap層で吸収する。

---

# 27. コマンド予測

cmdshell最大の特徴として維持する。

入力:

```text
git st
```

候補:

```text
git status
git stash
git stage
```

UI:

```text
┌─────────────────────┐
│ git status          │
│ git stash           │
│ git stage           │
└─────────────────────┘
```

---

# 28. 予測エンジン

予測エンジンはOS非依存。

入力:

```text
Current Input
History
Working Directory
Shell
```

から候補を生成する。

---

# 29. OSごとの予測

Shellによって候補を変える。

Windows:

```text
PowerShell commands
CMD commands
Git
Docker
Cargo
```

macOS/Linux:

```text
bash
zsh
fish
Git
Docker
Cargo
```

---

# 30. タブ

GUI版ではタブを最初から設計に含める。

```text
┌──────────────────────────────────────────┐
│ + │ PowerShell │ WSL │ CMD              │
├──────────────────────────────────────────┤
│                                          │
│ terminal                                 │
│                                          │
└──────────────────────────────────────────┘
```

各タブ:

```text
TerminalSession
```

を持つ。

---

# 31. Split Pane

v0.2以降で実装する。

```text
┌─────────────────────┬─────────────────────┐
│ PowerShell          │ WSL                 │
│                     │                     │
│                     │                     │
├─────────────────────┼─────────────────────┤
│ CMD                 │ PowerShell          │
│                     │                     │
└─────────────────────┴─────────────────────┘
```

ただし、v0.1.5では実装せず、アーキテクチャのみ対応する。

---

# 32. テーマ

OS共通のThemeシステムを作る。

```text
Theme
├── Background
├── Foreground
├── Cursor
├── Selection
├── ANSI Colors
├── Tab
└── Suggestion
```

初期:

```text
Dark
Light
```

を実装。

---

# 33. 設定

設定はOS共通。

```text
Settings
├── Appearance
│   ├── Theme
│   ├── Font
│   └── Font Size
│
├── Terminal
│   ├── Shell
│   ├── Scrollback
│   └── Cursor
│
├── Prediction
│   ├── Enabled
│   └── History
│
└── Keybindings
```

---

# 34. フォント

初期候補:

```text
JetBrains Mono
Cascadia Mono
Consolas
```

macOS/Linux/Windowsで利用可能なフォントを自動検出する。

フォントが存在しない場合はOSのmonospace fontへfallbackする。

---

# 35. アプリケーションとしての起動

## Windows

```text
cmdshell.exe
```

をダブルクリック。

## macOS

```text
cmdshell.app
```

をFinderから起動。

## Linux

```text
cmdshell
```

をDesktop Entryから起動。

---

# 36. Linux Desktop Integration

以下を用意する。

```text
/usr/share/applications/cmdshell.desktop
```

内容:

```text
[Desktop Entry]
Name=cmdshell
Exec=cmdshell
Type=Application
Categories=System;TerminalEmulator;
Terminal=false
```

---

# 37. macOS Bundle

最終的に、

```text
cmdshell.app
└── Contents
    ├── Info.plist
    ├── MacOS
    │   └── cmdshell
    └── Resources
        └── icon.icns
```

とする。

---

# 38. Windows Packaging

```text
cmdshell.exe
cmdshell.ico
```

を基本とする。

将来的には、

```text
MSIX
Installer
Portable ZIP
```

を提供する。

---

# 39. クロスコンパイル

開発環境と実行環境を分離する。

最低限、CIで、

```text
Windows x86_64
Linux x86_64
macOS x86_64
macOS ARM64
```

をビルドする。

将来的には、

```text
Linux ARM64
Windows ARM64
```

も検討する。

---

# 40. CI/CD

GitHub ActionsでOS別ビルドを行う。

```text
Push
  ↓
GitHub Actions
  ├── Windows
  ├── macOS Intel
  ├── macOS Apple Silicon
  └── Linux
        ↓
    Release Assets
```

---

# 41. クロスプラットフォームテスト

すべてのOSで共通テスト:

```text
TerminalBuffer
VT Parser
Prediction
History
Ranking
Selection
```

OS固有テスト:

```text
Windows
ConPTY
PowerShell
CMD

macOS
PTY
zsh

Linux
PTY
bash
Wayland
X11
```

---

# 42. Linux Wayland / X11

Linuxでは両方を意識する。

```text
Linux
├── Wayland
└── X11
```

GPUIのプラットフォーム構成でもLinux側にWayland/X11の機能分岐が存在するため、cmdshellもどちらか一方に固定しない。

---

# 43. macOS Apple Silicon

優先して対応する。

```text
aarch64-apple-darwin
```

Apple Silicon Macでネイティブ動作することを目標とする。

Intel Mac:

```text
x86_64-apple-darwin
```

もサポートする。

---

# 44. Windows x86_64

最優先Windowsターゲット。

```text
x86_64-pc-windows-msvc
```

を基本とする。

PowerShell / CMD / ConPTYが正常に動作することを確認する。

---

# 45. Linux x86_64

最優先Linuxターゲット。

```text
x86_64-unknown-linux-gnu
```

を基本とする。

Ubuntuを基準環境としてテストする。

---

# 46. macOS/Linux/Windowsの共通化率

目標:

```text
共通コード
80%以上

OS固有コード
20%以下
```

を目指す。

ただし、無理に共通化してPTYやWindowの品質を落とさない。

---

# 47. OS固有コードを隔離する

禁止:

```rust
#[cfg(target_os = "windows")]
// UI全体
```

のような巨大な条件分岐。

推奨:

```rust
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;
```

としてPlatform APIの境界を明確にする。

---

# 48. UIコードのOS依存禁止

以下は共通UIから直接呼び出さない。

```text
CreateProcess
fork
exec
ConPTY
NSWindow
X11
Wayland
```

UIは、

```text
TerminalSession
Shell
Clipboard
Window
```

などの抽象APIのみ利用する。

---

# 49. エラー処理

GUIアプリケーションではコンソールへエラーを出せないため、エラーをUIへ通知する。

```text
Terminal Error
Shell Error
PTY Error
Rendering Error
Configuration Error
```

を統一する。

---

# 50. Shell起動失敗

例えばWindowsで、

```text
PowerShell 7
     ↓
0xc0000142
```

が発生した場合、

```text
PowerShell 5.1
```

へfallback。

それも失敗した場合、

```text
CMD
```

へfallback。

GUI:

```text
┌────────────────────────────────────┐
│ PowerShell 7 could not be started │
│ Error: 0xc0000142                 │
│                                    │
│ Trying Windows PowerShell...       │
└────────────────────────────────────┘
```

---

# 51. ログ

CLIがなくなるため、ファイルログを用意する。

例:

```text
cmdshell
└── logs
    └── cmdshell.log
```

ただし、ユーザーの秘密情報をログに保存しない。

---

# 52. Crash Recovery

GUI自体がクラッシュしない設計を優先する。

Shellが終了:

```text
Shell
 ↓
TerminalSession終了
 ↓
GUI維持
```

PTYが壊れた:

```text
PTY Error
 ↓
TerminalSession Error
 ↓
Restart Shell
```

とする。

---

# 53. v0.1.5開発フェーズ

## Phase 1 — GPUI Window

最初に完成させる。

```text
[ ] GPUI導入
[ ] Application起動
[ ] Window表示
[ ] Windows
[ ] macOS
[ ] Linux
```

完成条件:

```text
3OSで空のcmdshell Windowが表示される。
```

---

# 54. Phase 2 — Terminal View

```text
[ ] TerminalView
[ ] TerminalBuffer
[ ] Text Rendering
[ ] Cursor
[ ] Keyboard Input
```

完成:

```text
GPUI Window
    ↓
Terminal
```

---

# 55. Phase 3 — PTY

```text
[ ] Windows ConPTY
[ ] macOS PTY
[ ] Linux PTY
[ ] Shell spawn
[ ] stdout
[ ] stdin
[ ] resize
```

完成:

```text
GPUI
 ↓
PTY
 ↓
Shell
```

---

# 56. Phase 4 — ANSI / VT

```text
[ ] ANSI Color
[ ] Cursor
[ ] Clear
[ ] Scroll
[ ] Alternate Screen
[ ] 256 Color
[ ] True Color
```

---

# 57. Phase 5 — 入力・選択

```text
[ ] Keyboard
[ ] Mouse
[ ] Selection
[ ] Copy
[ ] Paste
[ ] IME
```

---

# 58. Phase 6 — Prediction

```text
[ ] Command History
[ ] Suggestion Engine
[ ] Suggestion Popup
[ ] Ranking
[ ] Tab Completion
```

ここからcmdshell独自の特徴を強化する。

---

# 59. Phase 7 — Tabs

```text
[ ] TabBar
[ ] New Tab
[ ] Close Tab
[ ] Switch Tab
[ ] Independent PTY
```

---

# 60. Phase 8 — Settings

```text
[ ] Theme
[ ] Font
[ ] Font Size
[ ] Shell
[ ] Scrollback
[ ] Prediction
[ ] Keybindings
```

---

# 61. Phase 9 — Packaging

```text
[ ] Windows exe
[ ] Windows icon
[ ] macOS app bundle
[ ] macOS icon
[ ] Linux binary
[ ] Linux desktop entry
```

---

# 62. Phase 10 — Release

GitHub Release:

```text
cmdshell-v0.1.5-windows-x86_64.zip
cmdshell-v0.1.5-macos-x86_64.tar.gz
cmdshell-v0.1.5-macos-aarch64.tar.gz
cmdshell-v0.1.5-linux-x86_64.tar.gz
```

を作成する。

---

# 63. v0.1.5の完成条件

## GUI

* [ ] GPUIで描画される
* [ ] 独立したアプリケーションウィンドウとして起動する
* [ ] CLIを必要としない
* [ ] Windows対応
* [ ] macOS対応
* [ ] Linux対応

## Terminal

* [ ] PTYが動作する
* [ ] Shellが起動する
* [ ] 入力できる
* [ ] 出力できる
* [ ] ANSIカラーが表示される
* [ ] カーソルが動作する
* [ ] ウィンドウリサイズに対応する
* [ ] コピー・貼り付けができる
* [ ] スクロールできる

## Shell

* [ ] Windows: PowerShell
* [ ] Windows: CMD
* [ ] macOS: zsh
* [ ] Linux: bash
* [ ] Shell fallback

## Prediction

* [ ] コマンド履歴
* [ ] 候補表示
* [ ] 候補選択
* [ ] Tab補完

## Platform

* [ ] Windows x86_64
* [ ] macOS x86_64
* [ ] macOS ARM64
* [ ] Linux x86_64

---

# 64. v0.1.5でやらないこと

開発範囲を広げすぎないため、以下は後回しにする。

```text
[ ] AIチャット
[ ] SSH
[ ] SFTP
[ ] Docker UI
[ ] プラグイン
[ ] クラウド同期
[ ] Split Pane
[ ] 高度なテーママーケット
[ ] GUIからのGit操作
[ ] GUIからのファイル管理
```

最初に、

> **「3OSで高速に動く、まともなGUIターミナル」**

を完成させる。

---

# 65. v0.1.6以降

v0.1.5が安定した後、

```text
v0.1.6
├── Split Pane
├── Command Palette
└── Advanced Prediction

v0.1.7
├── SSH
├── Remote Terminal
└── Session Management

v0.2.0
├── AI Prediction
├── Plugin Architecture
├── Advanced Theme
└── Workspace
```

などを検討する。

---

# 66. 最終アーキテクチャ

最終的なcmdshellは以下を目標とする。

```text
                         cmdshell
                            │
                    ┌───────▼───────┐
                    │      GPUI     │
                    │ Application   │
                    └───────┬───────┘
                            │
                    ┌───────▼───────┐
                    │      UI       │
                    │               │
                    │ TabBar        │
                    │ TerminalView  │
                    │ Prediction    │
                    │ Settings      │
                    └───────┬───────┘
                            │
                    ┌───────▼───────┐
                    │ Terminal Core │
                    │               │
                    │ Buffer        │
                    │ VT Parser     │
                    │ Input         │
                    │ Selection     │
                    └───────┬───────┘
                            │
                    ┌───────▼───────┐
                    │ PTY Abstraction│
                    └───────┬───────┘
                            │
             ┌──────────────┼──────────────┐
             │              │              │
             ▼              ▼              ▼
         Windows          macOS          Linux
         ConPTY           PTY            PTY
             │              │              │
             ▼              ▼              ▼
        PowerShell          zsh           bash
        CMD                 bash          zsh
```

---

# 67. 最重要方針

cmdshell v0.1.5では、機能を増やすことよりも、

```text
GPUI
 ↓
Cross Platform
 ↓
Terminal Core
 ↓
PTY
 ↓
Shell
```

という基盤を完成させることを優先する。

特に、

```text
Windows
macOS
Linux
```

の3OSで同じコードベースからビルドできることを重要な完成条件とする。

---

# 68. プロジェクトの最終的な位置付け

cmdshellは、

```text
単なるCLIツール
```

ではなく、

```text
Native Terminal Application
```

として開発する。

技術スタック:

```text
Language:
    Rust

GUI:
    GPUI

Rendering:
    GPU accelerated

Terminal:
    PTY / ConPTY

Platforms:
    Windows
    macOS
    Linux

Shell:
    PowerShell
    CMD
    zsh
    bash
    fish

Unique Feature:
    Command Prediction
```

最終的なコンセプト:

> **ZedのようなRust + GPUIの設計思想で作る、クロスプラットフォーム・GPUアクセラレーション・コマンド予測付きターミナル。**

GUIを中心に設計し、Terminal CoreとPlatform層を明確に分離することで、Windowsだけに依存した実装にならないようにする。

---

# 69. 開発上の最優先順位

```text
1. GPUI
   ↓
2. Windows / macOS / Linux
   ↓
3. Window
   ↓
4. Terminal Rendering
   ↓
5. PTY
   ↓
6. Shell
   ↓
7. Input
   ↓
8. ANSI / VT
   ↓
9. Prediction
   ↓
10. Tabs
   ↓
11. Settings
   ↓
12. Packaging
```

**CLIはこの期間の開発対象外とする。**

まずは、

```text
Windows
macOS
Linux
```

の3環境で、

```text
cmdshell.exe / cmdshell.app / cmdshell
```

を起動すると、GPUIによる同一コンセプトのターミナルウィンドウが表示され、Shellを操作できる状態を最初の大きなマイルストーンとする。
