# PredictTerm — GPUIベース インテリジェント予測ターミナル設計書

## 1. プロジェクト概要

### 1.1 プロジェクト名

**PredictTerm**

### 1.2 コンセプト

PredictTermは、RustとGPUIで実装する次世代ターミナルエミュレータである。

単純にコマンドを実行するだけではなく、

* コマンド予測
* ファイル・ディレクトリ予測
* Gitブランチ予測
* コマンド履歴解析
* 使用頻度によるランキング
* プロジェクト認識
* Ghost Text
* AIによるコマンド予測
* 危険コマンド検出

などを統合する。

GUI描画には**GPUI**を使用する。

---

# 2. 基本方針

PredictTermは以下の3層に分離する。

```text
┌────────────────────────────────────────┐
│                GPUI UI                 │
│                                        │
│ Terminal View / Input / Suggestion     │
└──────────────────┬─────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────┐
│          Prediction Engine             │
│                                        │
│ Parser / History / Git / Ranking       │
└──────────────────┬─────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────┐
│          Terminal Backend              │
│                                        │
│ PTY / Shell / ConPTY                  │
└────────────────────────────────────────┘
```

UIと予測ロジック、PTY処理を完全に分離する。

---

# 3. 技術スタック

## 3.1 言語

```text
Rust
```

## 3.2 GUI

```text
GPUI
```

## 3.3 非同期処理

```text
Tokio
```

## 3.4 PTY

```text
portable-pty
```

## 3.5 Git

```text
git2
```

## 3.6 データベース

```text
SQLite
```

Rust側では、

```text
rusqlite
```

または

```text
sqlx
```

を使用する。

## 3.7 シリアライズ

```text
serde
toml
```

---

# 4. GPUIを採用する理由

PredictTermでは、従来のTUIライブラリではなくGPUIを採用する。

### 理由

* GPUアクセラレーションを利用できる
* Rustとの親和性が高い
* GUIベースのターミナルを構築できる
* 高度なレイアウトが可能
* アニメーションを実装しやすい
* カスタムUIを作りやすい
* 将来的なAI UIとの統合に向いている

---

# 5. UIアーキテクチャ

GPUI側では以下のViewを基本とする。

```text
Application
    │
    └── MainWindow
          │
          ├── TitleBar
          │
          ├── TabBar
          │
          ├── TerminalView
          │     ├── TerminalOutput
          │     ├── Prompt
          │     ├── Input
          │     └── SuggestionPopup
          │
          └── StatusBar
```

---

# 6. GPUI View構成

```text
src/ui/
├── mod.rs
├── app.rs
├── window.rs
├── title_bar.rs
├── tab_bar.rs
├── terminal_view.rs
├── terminal_output.rs
├── input.rs
├── suggestion.rs
├── ghost_text.rs
├── status_bar.rs
└── theme.rs
```

---

# 7. MainWindow

アプリケーション全体を管理する。

```text
MainWindow
│
├── TitleBar
├── TabBar
├── TerminalView
└── StatusBar
```

責務：

* Window生成
* View配置
* キーボードイベント処理
* タブ管理
* テーマ管理

---

# 8. TerminalView

ターミナル1枚分を管理する。

```text
TerminalView
│
├── Output
│
├── Prompt
│
├── Input
│
├── GhostText
│
└── SuggestionPopup
```

---

# 9. TerminalOutput

PTYから受信したデータを描画する。

```text
Shell
  │
  ▼
PTY
  │
  ▼
TerminalBackend
  │
  ▼
TerminalOutput
  │
  ▼
GPUI
```

出力例：

```text
user@pc:~/project$ cargo build
   Compiling predictterm v0.1.0
    Finished dev [unoptimized + debuginfo]
```

---

# 10. ANSIエスケープシーケンス

シェルはANSIエスケープシーケンスを出力する。

そのためTerminalOutputでは、

```text
ANSI Parser
```

を実装する。

対応対象：

* 色
* 太字
* 下線
* 背景色
* カーソル移動
* 画面クリア
* 行クリア
* スクロール
* 256色
* TrueColor

---

# 11. Terminal Grid

ターミナル表示は内部的にGridとして管理する。

```text
TerminalGrid
│
├── Row
│    ├── Cell
│    ├── Cell
│    └── Cell
│
├── Row
└── Row
```

Cell：

```rust
struct Cell {
    character: char,
    foreground: Color,
    background: Color,
    bold: bool,
    italic: bool,
    underline: bool,
}
```

---

# 12. GPUIによるTerminal描画

TerminalGridをGPUIのElementとして描画する。

```text
TerminalGrid
      │
      ▼
GPUI Element
      │
      ▼
GPU Renderer
      │
      ▼
Window
```

大量の文字を個別の複雑なViewとして管理せず、描画効率を考慮した構造にする。

---

# 13. 入力システム

ユーザーの入力は、

```text
Keyboard Event
      │
      ▼
InputManager
      │
      ├── Shell Input
      ├── Suggestion
      └── Navigation
```

として処理する。

---

# 14. 入力状態

```rust
struct InputState {
    text: String,
    cursor: usize,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
}
```

---

# 15. Ghost Text

GPUI上で入力文字列の後ろに予測結果を表示する。

例：

```text
$ git st▌atus
```

ユーザーが入力しているのは、

```text
git st
```

予測されているのは、

```text
atus
```

である。

Ghost Textは、

* 実際の入力ではない
* Shellへ送信しない
* Tabで確定できる

という仕様にする。

---

# 16. SuggestionPopup

候補一覧をGPUIのViewとして描画する。

```text
┌──────────────────────────────────┐
│ git status                       │
│ git stash                        │
│ git stage                        │
│ git show                         │
└──────────────────────────────────┘
```

各候補：

```text
CandidateItem
```

として管理する。

---

# 17. 候補UI

候補には以下の情報を表示可能にする。

```text
git status
────────────────────
Git status command
History: 143 times
Score: 0.94
```

ただし通常時は、

```text
git status
git stash
git stage
```

程度のシンプルな表示とする。

詳細情報はホバーまたはショートカットで表示する。

---

# 18. 候補選択

```text
↑
↓
```

で選択。

```text
Tab
```

で確定。

```text
Esc
```

で閉じる。

```text
Enter
```

は原則としてコマンド実行。

---

# 19. ターミナルタブ

GPUIのWindow内で複数ターミナルを管理する。

```text
┌───────────────────────────────────────────────┐
│ Terminal 1 │ Terminal 2 │ +                  │
├───────────────────────────────────────────────┤
│                                               │
│ user@pc:~$                                    │
│                                               │
└───────────────────────────────────────────────┘
```

各タブは独立したPTYを持つ。

```text
Tab
 │
 └── TerminalSession
       │
       ├── PTY
       ├── Shell
       ├── InputState
       └── TerminalGrid
```

---

# 20. TerminalSession

```rust
struct TerminalSession {
    id: SessionId,
    shell: ShellType,
    cwd: PathBuf,
    terminal: TerminalGrid,
    input: InputState,
    prediction: PredictionState,
}
```

---

# 21. PTYアーキテクチャ

```text
             GPUI
              │
              ▼
       TerminalSession
              │
              ▼
       TerminalBackend
              │
              ▼
             PTY
              │
              ▼
            Shell
```

Linux/macOS：

```text
PTY
```

Windows：

```text
ConPTY
```

---

# 22. PTY処理スレッド

PTYの読み書きはUIスレッドから分離する。

```text
GPUI Main Thread
       │
       │ Channel
       ▼
PTY Worker
       │
       ▼
Shell
```

これによりShellが停止した場合でもUIがフリーズしないようにする。

---

# 23. 非同期イベント

内部イベント：

```rust
enum TerminalEvent {
    Output(Vec<u8>),
    Exit(i32),
    Resize(u16, u16),
}
```

UIイベント：

```rust
enum UIEvent {
    InputChanged(String),
    CandidateSelected(usize),
    ExecuteCommand,
}
```

---

# 24. Prediction Engine

UIとは独立したモジュールにする。

```text
src/predictor/
├── mod.rs
├── engine.rs
├── candidate.rs
├── context.rs
├── ranking.rs
└── cache.rs
```

---

# 25. PredictionContext

予測に必要な情報をまとめる。

```rust
struct PredictionContext {
    input: String,
    cwd: PathBuf,
    shell: ShellType,
    git: Option<GitContext>,
    project: Option<ProjectType>,
}
```

---

# 26. Candidate

```rust
struct Candidate {
    text: String,
    description: Option<String>,
    source: CandidateSource,
    score: f32,
}
```

---

# 27. CandidateSource

```rust
enum CandidateSource {
    History,
    Command,
    Filesystem,
    Git,
    Option,
    Project,
    AI,
}
```

---

# 28. Provider

```rust
trait CandidateProvider {
    fn suggest(
        &self,
        context: &PredictionContext
    ) -> Vec<Candidate>;
}
```

実装：

```text
HistoryProvider
CommandProvider
FilesystemProvider
GitProvider
OptionProvider
ProjectProvider
AIProvider
```

---

# 29. Candidate Aggregator

複数Providerの結果を統合する。

```text
HistoryProvider ─────┐
CommandProvider ─────┤
FilesystemProvider ──┤
GitProvider ─────────┤
ProjectProvider ─────┤
AIProvider ──────────┤
                     ▼
              Aggregator
                     │
                     ▼
              RankingEngine
```

---

# 30. Ranking Engine

候補をScore順に並べる。

基本Score：

```text
Score =
    PrefixScore
  + HistoryScore
  + FrequencyScore
  + ContextScore
  + ProjectScore
  + GitScore
  + RecentScore
```

---

# 31. 予測更新

入力が変更されるたびに予測する。

```text
User Input
    │
    ▼
InputChanged
    │
    ▼
PredictionContext
    │
    ▼
PredictionEngine
    │
    ▼
Candidates
    │
    ▼
GPUI State Update
    │
    ▼
Repaint
```

ただし、毎キー入力で重い処理を実行しない。

---

# 32. Debounce

予測処理には短いDebounceを入れる。

```text
Input
 ↓
10〜30ms待機
 ↓
Prediction
```

連続入力中の不要な予測を減らす。

---

# 33. キャッシュ

以下をキャッシュする。

```text
CommandCache
FilesystemCache
GitCache
HistoryCache
ProjectCache
```

特に、

```text
PATH解析
Git解析
Filesystem解析
```

は毎回実行しない。

---

# 34. プロジェクト検出

現在のディレクトリを解析する。

```text
Cargo.toml
package.json
pyproject.toml
go.mod
CMakeLists.txt
Makefile
```

などを検出する。

例：

```text
Cargo.toml
```

↓

```text
ProjectType::Rust
```

---

# 35. プロジェクトProvider

Rustの場合：

```text
cargo build
cargo run
cargo test
cargo check
cargo clippy
cargo fmt
```

Node.jsの場合：

```text
npm install
npm run dev
npm run build
npm test
```

などを候補に追加する。

---

# 36. Git Provider

Gitリポジトリ内では、

* branch
* remote
* modified files
* staged files
* tags

を取得する。

例：

```text
git checkout fea
```

↓

```text
feature/login
feature/api
feature/test
```

---

# 37. 履歴システム

SQLiteに保存する。

```sql
CREATE TABLE command_history (
    id INTEGER PRIMARY KEY,
    command TEXT NOT NULL,
    cwd TEXT,
    exit_code INTEGER,
    executed_at INTEGER
);
```

---

# 38. セキュリティ

履歴には、

```text
API Key
Password
Token
SSH情報
Authorization Header
```

などが含まれる可能性がある。

そのため、

* Secret検出
* 自動マスキング
* 保存除外
* 履歴削除
* 履歴無効化

を実装する。

---

# 39. 危険コマンド検出

以下などを検出する。

```text
rm -rf
mkfs
dd
shutdown
reboot
git reset --hard
git clean -fd
```

実行前にGPUIダイアログを表示する。

```text
┌──────────────────────────────────────┐
│ Potentially Destructive Command      │
│                                      │
│ git reset --hard                     │
│                                      │
│ This may discard uncommitted changes │
│                                      │
│ [ Cancel ]          [ Execute ]      │
└──────────────────────────────────────┘
```

---

# 40. GPUI UIテーマ

テーマシステムを独立させる。

```text
src/ui/theme.rs
```

設定：

```toml
[theme]
name = "dark"

[theme.colors]
background = "#101010"
foreground = "#E0E0E0"
accent = "#7AA2F7"
```

ただし、色設定は将来的にGPUIのテーマシステムに合わせて拡張可能にする。

---

# 41. UIレイアウト

基本構成：

```text
┌────────────────────────────────────────────────────┐
│ PredictTerm                            ─ □ ×       │
├────────────────────────────────────────────────────┤
│ Terminal 1 │ Terminal 2 │ +                        │
├────────────────────────────────────────────────────┤
│                                                    │
│ user@pc:~/project$ cargo build                     │
│    Compiling predictterm                           │
│     Finished dev                                   │
│                                                    │
│ user@pc:~/project$ cargo bu▌                       │
│                         ild                         │
│                                                    │
│ ┌──────────────────────────────────────────────┐   │
│ │ cargo build                                  │   │
│ │ cargo bundle                                 │   │
│ │ cargo bump                                   │   │
│ └──────────────────────────────────────────────┘   │
│                                                    │
├────────────────────────────────────────────────────┤
│ bash │ ~/project │ Rust │ Git: feature/login       │
└────────────────────────────────────────────────────┘
```

---

# 42. GPUIコンポーネント

主要コンポーネント：

```text
MainWindow
TerminalTabBar
TerminalView
TerminalGrid
TerminalCursor
Prompt
CommandInput
GhostText
SuggestionPopup
SuggestionItem
StatusBar
ConfirmDialog
SettingsView
```

---

# 43. Cursor

カーソルはGPUI上で描画する。

状態：

```text
Blinking
Visible
Hidden
```

カーソル点滅はTimer/Eventによって制御する。

---

# 44. Terminal Selection

マウス選択に対応する。

```text
Mouse Down
    ↓
Selection Start
    ↓
Mouse Move
    ↓
Selection Update
    ↓
Mouse Up
```

コピー：

```text
Ctrl+C
```

貼り付け：

```text
Ctrl+V
```

---

# 45. マウス操作

GPUIを利用することで、

* テキスト選択
* 右クリック
* タブ操作
* スクロール
* UIボタン
* ドラッグ

などを実装できる。

---

# 46. 設定画面

GPUIで設定画面も実装する。

```text
Settings
│
├── General
├── Terminal
├── Prediction
├── History
├── Git
├── AI
├── Security
└── Appearance
```

---

# 47. CLI

GUIアプリケーションとして起動：

```bash
predictterm
```

設定：

```bash
predictterm config
```

診断：

```bash
predictterm doctor
```

履歴：

```bash
predictterm history
```

統計：

```bash
predictterm stats
```

---

# 48. 起動処理

```text
main()
 │
 ├── Load Config
 │
 ├── Initialize Database
 │
 ├── Initialize Prediction Engine
 │
 ├── Initialize GPUI Application
 │
 ├── Create Window
 │
 └── Start Terminal Session
```

---

# 49. 推奨ディレクトリ構成

```text
predictterm/
├── Cargo.toml
├── README.md
├── SPEC.md
├── TODO.md
├── CHANGELOG.md
├── LICENSE
│
├── src/
│   ├── main.rs
│   │
│   ├── app/
│   │   ├── mod.rs
│   │   ├── state.rs
│   │   └── events.rs
│   │
│   ├── terminal/
│   │   ├── mod.rs
│   │   ├── backend.rs
│   │   ├── pty.rs
│   │   ├── session.rs
│   │   ├── ansi.rs
│   │   ├── grid.rs
│   │   └── cell.rs
│   │
│   ├── predictor/
│   │   ├── mod.rs
│   │   ├── engine.rs
│   │   ├── context.rs
│   │   ├── candidate.rs
│   │   ├── ranking.rs
│   │   └── cache.rs
│   │
│   ├── providers/
│   │   ├── mod.rs
│   │   ├── history.rs
│   │   ├── command.rs
│   │   ├── filesystem.rs
│   │   ├── git.rs
│   │   ├── project.rs
│   │   ├── option.rs
│   │   └── ai.rs
│   │
│   ├── database/
│   │   ├── mod.rs
│   │   ├── history.rs
│   │   └── statistics.rs
│   │
│   ├── shell/
│   │   ├── mod.rs
│   │   ├── bash.rs
│   │   ├── zsh.rs
│   │   └── powershell.rs
│   │
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── main_window.rs
│   │   ├── title_bar.rs
│   │   ├── tab_bar.rs
│   │   ├── terminal_view.rs
│   │   ├── terminal_grid.rs
│   │   ├── input.rs
│   │   ├── ghost_text.rs
│   │   ├── suggestion.rs
│   │   ├── status_bar.rs
│   │   ├── dialog.rs
│   │   └── theme.rs
│   │
│   └── config/
│       ├── mod.rs
│       └── settings.rs
│
└── tests/
    ├── parser_tests.rs
    ├── predictor_tests.rs
    ├── ranking_tests.rs
    └── terminal_tests.rs
```

---

# 50. 依存関係

概念的には以下の依存関係にする。

```text
                 ┌──────────────┐
                 │     GPUI     │
                 └──────┬───────┘
                        │
                        ▼
                 ┌──────────────┐
                 │      UI      │
                 └──────┬───────┘
                        │
          ┌─────────────┴─────────────┐
          ▼                           ▼
 ┌────────────────┐          ┌────────────────┐
 │ TerminalState  │          │ PredictionState│
 └───────┬────────┘          └───────┬────────┘
         │                           │
         ▼                           ▼
 ┌────────────────┐          ┌────────────────┐
 │ Terminal       │          │ Prediction     │
 │ Backend        │          │ Engine         │
 └───────┬────────┘          └───────┬────────┘
         │                           │
         ▼                           ▼
       PTY                    Providers
```

UIから直接GitやSQLiteを操作しない。

---

# 51. GPUIと予測エンジンの通信

UIとPrediction Engineは直接密結合させない。

```text
InputChanged
     │
     ▼
Prediction Request
     │
     ▼
Background Worker
     │
     ▼
Prediction Result
     │
     ▼
GPUI State Update
```

---

# 52. バックグラウンド処理

以下はUIスレッドで実行しない。

```text
Git解析
Filesystem Scan
History検索
PATH Scan
AI推論
```

これらはWorkerで処理する。

```text
GPUI
 │
 ├── UI Thread
 │
 └── Worker
      ├── Prediction
      ├── Git
      ├── Filesystem
      └── AI
```

---

# 53. MVP開発フェーズ

## Phase 1 — GPUI Window

* [ ] Rustプロジェクト作成
* [ ] GPUI導入
* [ ] Window生成
* [ ] MainView作成
* [ ] テーマ作成
* [ ] キーボード入力
* [ ] マウス入力

---

## Phase 2 — Terminal Core

* [ ] portable-pty導入
* [ ] Bash起動
* [ ] PTY入力
* [ ] PTY出力
* [ ] ANSI Parser
* [ ] Terminal Grid
* [ ] Cursor
* [ ] Scrollback

---

## Phase 3 — GPUI Terminal Renderer

* [ ] TerminalGrid描画
* [ ] Text描画
* [ ] 色
* [ ] Bold
* [ ] Underline
* [ ] Cursor
* [ ] Selection
* [ ] Copy
* [ ] Paste

---

## Phase 4 — Prediction

* [ ] CommandProvider
* [ ] FilesystemProvider
* [ ] HistoryProvider
* [ ] CandidateAggregator
* [ ] RankingEngine
* [ ] SuggestionPopup
* [ ] Ghost Text

---

## Phase 5 — Context

* [ ] GitProvider
* [ ] ProjectProvider
* [ ] OptionProvider
* [ ] Cache
* [ ] Project Detection

---

## Phase 6 — Windows

* [ ] Windowsビルド
* [ ] ConPTY
* [ ] PowerShell
* [ ] cmd.exe
* [ ] Windows filesystem対応

---

## Phase 7 — AI

* [ ] AIProvider
* [ ] Local LLM API
* [ ] Context Builder
* [ ] AI Candidate Ranking
* [ ] 自然言語コマンド生成
* [ ] 危険コマンド確認UI

---

# 54. MVP完成条件

以下を満たすことをMVP完成条件とする。

* [ ] GPUIでWindowが起動する
* [ ] LinuxでBashを起動できる
* [ ] コマンドを入力できる
* [ ] コマンドを実行できる
* [ ] stdout/stderrを表示できる
* [ ] ANSIカラーを表示できる
* [ ] カーソルが動作する
* [ ] スクロールできる
* [ ] テキスト選択できる
* [ ] コピー・ペーストできる
* [ ] コマンド候補を表示できる
* [ ] ファイル候補を表示できる
* [ ] ↑↓で候補を選択できる
* [ ] Tabで候補を確定できる
* [ ] Ghost Textを表示できる
* [ ] 予測処理でUIがフリーズしない

---

# 55. 将来の完成形

最終的なPredictTermは、

```text
┌─────────────────────────────────────────────────────────┐
│ PredictTerm                              ─ □ ×          │
├─────────────────────────────────────────────────────────┤
│ Terminal 1 │ Terminal 2 │ +                             │
├─────────────────────────────────────────────────────────┤
│                                                         │
│ user@pc:~/project$ git status                           │
│ On branch feature/login                                 │
│                                                         │
│ user@pc:~/project$ git pu▌sh                            │
│                           sh                            │
│                                                         │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ git push                                            │ │
│ │ git push origin feature/login                       │ │
│ │ git push --set-upstream origin feature/login        │ │
│ └─────────────────────────────────────────────────────┘ │
│                                                         │
├─────────────────────────────────────────────────────────┤
│ bash │ ~/project │ Rust │ Git: feature/login │ AI: OFF  │
└─────────────────────────────────────────────────────────┘
```

というGUIターミナルになる。

---

# 56. 最終アーキテクチャ

```text
                         PredictTerm
                              │
                    ┌─────────┴─────────┐
                    │                   │
                    ▼                   ▼
                  GPUI              Application
                    │                   │
        ┌───────────┼───────────┐       │
        │           │           │       │
        ▼           ▼           ▼       ▼
    Terminal      Input      Suggestion State
      View         View         View
        │           │           │
        └───────────┼───────────┘
                    │
                    ▼
             TerminalSession
                    │
          ┌─────────┴─────────┐
          │                   │
          ▼                   ▼
         PTY             Prediction Engine
          │                   │
          ▼          ┌────────┼────────┐
        Shell        │        │        │
                     ▼        ▼        ▼
                  History    Git    Filesystem
                              │
                              ▼
                         Project Context
                              │
                              ▼
                         Ranking Engine
                              │
                              ▼
                         AI Provider
```

## 設計上の最重要ポイント

**GPUIはあくまで描画・UI層として使用し、ターミナルエンジンと予測エンジンをGPUIから独立させる。**

これにより、

```text
GPUI
  ↓
高速なGUI描画

PTY
  ↓
本物のShell実行

Prediction Engine
  ↓
コマンド予測

AI Engine
  ↓
高度な予測
```

という責務分離ができる。

この構成なら、最初は**「GPUIで動く高速なターミナル」**として完成させ、その上に予測機能、Git連携、プロジェクト認識、最終的にローカルLLMを段階的に追加できる。
