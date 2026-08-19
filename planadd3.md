# cmdshell v0.1.3 — PowerShell `0xc0000142` 修正計画

## 1. 概要

`sha256san/cmdshell` v0.1.3 の Windows 環境において、PowerShell 起動時に以下のエラーが発生する問題を修正する。

```text
powershell.exe - アプリケーション エラー

アプリケーションを正しく起動できませんでした (0xc0000142)。
[OK] をクリックしてアプリケーションを閉じてください。
```

`0xc0000142` は Windows の `STATUS_DLL_INIT_FAILED` に相当し、プロセス生成後の DLL 初期化に失敗した場合などに発生する。

cmdshell の現在の実装では、Windows の環境変数を子プロセスへ引き渡す処理や、PowerShell の起動処理が複数箇所に分散している。

本修正では、単純に `powershell.exe` を `cmd.exe` に置き換えるのではなく、

```text
Shell Discovery
      ↓
Environment Normalization
      ↓
Shell-specific Arguments
      ↓
Shell Health Check
      ↓
PTY Spawn
      ↓
Terminal Session
```

という責務分離された構造へ改善する。

---

# 2. 対象バージョン

* 現在の対象: `v0.1.3`
* 次期修正版: `v0.1.4`
* 対象OS: Windows
* 主な対象Shell:

  * PowerShell 7 (`pwsh.exe`)
  * Windows PowerShell 5.1 (`powershell.exe`)
  * Command Prompt (`cmd.exe`)
  * Git Bash
  * WSL

---

# 3. 現在確認されている問題

## 3.1 PowerShell起動時の `0xc0000142`

現在の症状:

```text
cmdshell
    ↓
powershell.exe
    ↓
0xc0000142
```

この問題は `cmdshell` 自体のPTY処理だけでなく、以下の要素が関係する可能性がある。

* 子プロセスへ渡す環境変数
* PATH
* SystemRoot
* SystemDrive
* ComSpec
* TEMP
* TMP
* USERPROFILE
* PowerShell Profile
* PowerShellの起動引数
* ConPTY / PTY経由のプロセス生成
* PowerShell 5.1 / 7 の選択方法

---

# 4. 現在の実装上の問題

## 4.1 親プロセスの環境変数をそのままコピーしている

現在の `src/terminal/pty.rs` では、概念的に以下の処理を行っている。

```rust
for (k, v) in std::env::vars() {
    cmd.env(k, v);
}
```

これは一見問題ないように見えるが、Windows環境ではユーザー環境に依存したPATHや環境変数がそのままPTY上のShellへ渡される。

特にPATHには、

```text
存在しないディレクトリ
古いアプリケーション
アンインストール済みソフトウェア
独自DLLを含むディレクトリ
```

などが含まれている場合がある。

そのため、Windows Shell起動時に使用する環境を明示的に正規化する。

---

# 5. `ensure_essential_windows_env()` の改善

現在の実装では、Windowsで必要な環境変数を補完する処理が存在する。

しかし、これを単純な「不足している環境変数を追加する処理」から、

```text
Windows Shell Environment Builder
```

として独立させる。

---

## 5.1 必須環境変数

最低限、以下を扱う。

```text
SystemRoot
SystemDrive
ComSpec
PATH
TEMP
TMP
USERPROFILE
HOMEDRIVE
HOMEPATH
APPDATA
LOCALAPPDATA
```

---

## 5.2 SystemRoot

基本的には既存値を利用する。

```text
SystemRoot
```

が存在しない場合のみ、Windows標準の場所を使用する。

一般的には、

```text
C:\Windows
```

だが、ドライブレターを固定せず、Windows APIまたは既存環境変数から取得する。

---

## 5.3 SystemDrive

```text
SystemDrive
```

も既存値を優先する。

存在しない場合は `SystemRoot` から推測する。

例:

```text
C:\Windows
    ↓
C:
```

---

# 6. PATHの正規化

現在のPATHをそのまま利用するのではなく、Windows Shellに必要なパスを優先的に追加する。

最低限、以下を保証する。

```text
%SystemRoot%\System32
%SystemRoot%\System32\Wbem
%SystemRoot%\System32\WindowsPowerShell\v1.0
```

例:

```text
C:\Windows\System32
C:\Windows\System32\Wbem
C:\Windows\System32\WindowsPowerShell\v1.0
```

---

## 6.1 PATHの構築例

概念:

```rust
fn build_windows_path() -> String {
    let system_root = get_system_root();

    let mut paths = vec![
        system_root.join("System32"),
        system_root.join("System32").join("Wbem"),
        system_root
            .join("System32")
            .join("WindowsPowerShell")
            .join("v1.0"),
    ];

    if let Some(path) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&path));
    }

    std::env::join_paths(paths)
        .unwrap()
        .to_string_lossy()
        .into_owned()
}
```

実装時には、

* 重複除去
* 存在しないPATHの扱い
* PATHが長すぎる場合
* Windows固有のPATH形式

も考慮する。

---

# 7. PowerShell Profileを無効化

PTY内でPowerShellを起動する場合、ユーザーのPowerShell Profileが原因でShell起動に失敗する可能性がある。

そのため初期起動では、

```text
-NoLogo
-NoProfile
```

を使用する。

推奨:

```text
powershell.exe -NoLogo -NoProfile
```

PowerShell 7の場合:

```text
pwsh.exe -NoLogo -NoProfile
```

必要に応じて、

```text
-NonInteractive
```

も使用する。

ただし、通常の対話型ターミナルとして動作させる場合、最終的なShellセッションでは `-NonInteractive` を付けない。

---

# 8. Shellごとの起動引数を管理する

Shellによって必要な引数が異なる。

そのため、Shellのパスだけを管理するのではなく、

```rust
enum ShellKind {
    Cmd,
    WindowsPowerShell,
    PowerShell,
    GitBash,
    Wsl,
}
```

を追加する。

---

## 8.1 ShellInfo

現在の概念:

```rust
pub struct ShellInfo {
    pub name: String,
    pub path: PathBuf,
    pub is_available: bool,
}
```

を以下のように拡張する。

```rust
pub struct ShellInfo {
    pub name: String,
    pub path: PathBuf,
    pub kind: ShellKind,
    pub is_available: bool,
}
```

---

# 9. Shell別起動設定

## PowerShell 7

```text
pwsh.exe -NoLogo -NoProfile
```

## Windows PowerShell 5.1

```text
powershell.exe -NoLogo -NoProfile
```

## CMD

```text
cmd.exe /Q
```

## Git Bash

Git Bashの実装に合わせた起動引数を使用する。

例:

```text
bash.exe --login -i
```

## WSL

WSLの場合は、

```text
wsl.exe
```

を基本とし、必要に応じてDistributionを指定する。

---

# 10. PowerShell 7を優先

Windowsでは、可能ならPowerShell 7を優先する。

推奨順:

```text
1. pwsh.exe
2. powershell.exe
3. cmd.exe
4. Git Bash
5. WSL
```

ただし、これは「存在する順」ではなく、実際に起動可能かを確認して決定する。

---

# 11. Shell Health Check

現在の問題で特に重要な修正。

`CreateProcess` が成功しただけではShellが正常とは限らない。

以下の2段階を分離する。

```text
Process Spawn
    ↓
Shell Initialization
```

---

## 11.1 PowerShell Health Check

以下のようなコマンドを実行する。

```text
powershell.exe -NoLogo -NoProfile -Command "exit 0"
```

PowerShell 7:

```text
pwsh.exe -NoLogo -NoProfile -Command "exit 0"
```

終了コード:

```text
0
```

なら正常と判定する。

---

# 12. `0xc0000142` の検出

Shell Health Checkで、

```text
0xc0000142
```

を検出した場合は、そのShellを利用可能Shellから除外する。

例:

```text
PowerShell 7
    ↓
FAIL
0xc0000142
    ↓
PowerShell 5.1
    ↓
PASS
```

または、

```text
PowerShell 7
    ↓
FAIL

PowerShell 5.1
    ↓
FAIL

CMD
    ↓
PASS
```

とする。

---

# 13. `pty.rs` の改善

現在の問題:

```text
spawn_command()
    ↓
成功したらspawned = true
```

だけでは、Shell内部の初期化失敗を検出できない可能性がある。

そのため、

```text
Shell Discovery
      ↓
Health Check
      ↓
正常なShellだけをPTYへ渡す
```

という構造に変更する。

---

# 14. 推奨アーキテクチャ

```text
ShellResolver
       │
       ▼
Shell Discovery
       │
       ▼
Shell Environment Builder
       │
       ▼
Shell Health Check
       │
       ├── FAIL
       │     │
       │     └── 次のShell
       │
       ▼
Shell Launch Configuration
       │
       ▼
PTY / ConPTY
       │
       ▼
TerminalSession
       │
       ▼
cmdshell UI
```

---

# 15. `src/shell/windows.rs`

主な修正内容:

* [ ] PowerShell 7検出
* [ ] Windows PowerShell 5.1検出
* [ ] CMD検出
* [ ] Git Bash検出
* [ ] WSL検出
* [ ] ShellKind追加
* [ ] PATH正規化
* [ ] Windows必須環境変数の補完
* [ ] Shell Health Check
* [ ] Shell優先順位の整理
* [ ] PowerShell起動引数の定義

---

# 16. `src/shell/mod.rs`

主な修正内容:

* [ ] `ShellKind` を追加
* [ ] `ShellInfo` に `kind` を追加
* [ ] Shellごとの起動設定を管理
* [ ] Shell Health Check APIを追加
* [ ] Shell ResolverとShell Launcherを分離

推奨構造:

```text
src/shell/
├── mod.rs
├── windows.rs
├── unix.rs
├── resolver.rs
├── health.rs
└── environment.rs
```

---

# 17. `src/terminal/pty.rs`

主な修正内容:

* [ ] 親環境変数の無条件コピーを見直す
* [ ] `environment.rs` の環境変数を利用
* [ ] ShellKindに応じた引数を使用
* [ ] Health Check済みShellのみ起動
* [ ] Shell起動失敗時の詳細ログ追加
* [ ] Windows固有のエラーコードを表示

---

# 18. エラー表示

現在:

```text
PowerShell failed to start
```

だけでは原因を特定しにくい。

改善後:

```text
Shell startup failed

Shell:
  PowerShell 5.1

Executable:
  C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe

Error:
  0xc0000142

Meaning:
  STATUS_DLL_INIT_FAILED

Environment:
  SystemRoot: OK
  SystemDrive: OK
  ComSpec: OK
  PATH: OK
  TEMP: OK
  TMP: OK

Fallback:
  Trying CMD...
```

のようにする。

---

# 19. `doctor` の強化

既存のdoctor機能を利用して、Windows Shell診断を追加する。

実行:

```text
predictterm doctor
```

出力例:

```text
PredictTerm Doctor
────────────────────────────────────

Operating System
  Windows       OK
  Architecture x86_64
  SystemRoot   OK
  SystemDrive  OK

Environment
  PATH         OK
  TEMP         OK
  TMP          OK
  USERPROFILE  OK

Shells
  PowerShell 7
    pwsh.exe
    Status: OK

  Windows PowerShell
    powershell.exe
    Status: FAIL
    Error: 0xc0000142

  Command Prompt
    cmd.exe
    Status: OK

  Git Bash
    Status: OK

  WSL
    Status: OK

Recommended Shell:
  PowerShell 7
```

---

# 20. テスト

## 20.1 PowerShell 7

```text
pwsh.exe
```

* [ ] 起動する
* [ ] 入力できる
* [ ] 出力できる
* [ ] Ctrl+Cが動作する
* [ ] 終了できる

---

## 20.2 Windows PowerShell

```text
powershell.exe
```

* [ ] 起動する
* [ ] `-NoProfile` が有効
* [ ] 入力できる
* [ ] 出力できる
* [ ] Ctrl+Cが動作する

---

## 20.3 CMD

```text
cmd.exe
```

* [ ] 起動する
* [ ] 入力できる
* [ ] 出力できる
* [ ] Ctrl+Cが動作する

---

# 21. 環境変数テスト

以下のケースをテストする。

* [ ] PATHが正常
* [ ] PATHに存在しないディレクトリがある
* [ ] PATHが空
* [ ] SystemRootが存在しない
* [ ] TEMPが存在しない
* [ ] TMPが存在しない
* [ ] USERPROFILEが存在しない
* [ ] PATHに重複がある
* [ ] PATHが非常に長い

---

# 22. PowerShell Profileテスト

以下の状態をテストする。

```text
$PROFILEあり
$PROFILEなし
壊れたProfile
存在しないModuleをImport
oh-my-posh
starship
conda
git integration
```

初期起動では、

```text
-NoProfile
```

によってProfileの影響を受けないことを確認する。

---

# 23. `0xc0000142` 再現テスト

最重要テスト。

```text
cmdshell
    ↓
PowerShell
    ↓
0xc0000142
```

という環境を再現できる場合は、以下を比較する。

```text
通常のpowershell.exe
cmdshell → powershell.exe
cmdshell → pwsh.exe
cmdshell → cmd.exe
```

比較項目:

```text
PATH
SystemRoot
TEMP
TMP
USERPROFILE
Parent Process
Command Line
Working Directory
Environment Block
```

---

# 24. ログ

Windows版ではShell起動時にDebugログを追加する。

例:

```text
[DEBUG] Shell: PowerShell 7
[DEBUG] Executable: C:\Program Files\PowerShell\7\pwsh.exe
[DEBUG] Arguments: -NoLogo -NoProfile
[DEBUG] Working Directory: C:\Users\User
[DEBUG] SystemRoot: C:\Windows
[DEBUG] PATH: normalized
[DEBUG] Starting PTY...
[DEBUG] Process created
[DEBUG] Shell health check: PASS
```

失敗時:

```text
[ERROR] Shell startup failed
[ERROR] Shell: powershell.exe
[ERROR] Exit code: 0xc0000142
[ERROR] STATUS_DLL_INIT_FAILED
[INFO] Trying fallback shell: cmd.exe
```

---

# 25. セキュリティ上の注意

環境変数をログに出す場合、以下は値を完全に表示しない。

```text
TOKEN
PASSWORD
SECRET
API_KEY
AWS_SECRET_ACCESS_KEY
GITHUB_TOKEN
```

特に、

```text
std::env::vars()
```

をそのままログへ出力しない。

---

# 26. 修正優先順位

## P0 — 必須

* [ ] PowerShellを `-NoLogo -NoProfile` で起動
* [ ] Windows環境変数構築を修正
* [ ] PATHを正規化
* [ ] Shell Health Checkを追加
* [ ] `0xc0000142` を検出
* [ ] Shell起動失敗時にfallback

## P1 — 高優先度

* [ ] PowerShell 7を優先
* [ ] `ShellKind`を導入
* [ ] Shellごとの起動引数を管理
* [ ] `pty.rs` のShell起動処理を整理
* [ ] `doctor` のShell診断を強化

## P2 — 改善

* [ ] Shell起動ログ
* [ ] Windows固有エラーの日本語/詳細説明
* [ ] Shellごとの診断結果表示
* [ ] PATH重複検出
* [ ] 不正PATH検出

---

# 27. 推奨ディレクトリ構造

```text
src/
├── main.rs
│
├── shell/
│   ├── mod.rs
│   ├── resolver.rs
│   ├── health.rs
│   ├── environment.rs
│   ├── windows.rs
│   └── unix.rs
│
├── terminal/
│   ├── mod.rs
│   ├── pty.rs
│   └── session.rs
│
└── ui/
    └── ...
```

---

# 28. v0.1.4の完成条件

以下をすべて満たした時点で修正完了とする。

```text
[ ] Windowsでcmdshellを起動できる
[ ] PowerShell 7を検出できる
[ ] PowerShell 5.1を検出できる
[ ] CMDを検出できる
[ ] PATHを正規化できる
[ ] 必須環境変数を補完できる
[ ] PowerShell Profileの影響を受けない
[ ] Shell Health Checkが動作する
[ ] 0xc0000142を検出できる
[ ] Shell起動失敗時にfallbackできる
[ ] CMD fallbackが正常に動作する
[ ] PTY上でPowerShellが正常に動作する
[ ] Ctrl+Cが正常に動作する
[ ] 日本語入力が正常に動作する
[ ] doctorでShell状態を確認できる
[ ] Windows x86_64ビルドが成功する
[ ] GitHub Release用バイナリが起動する
```

---

# 29. 最終的な目標

v0.1.3では、

```text
Shellを見つける
    ↓
そのままPTYで起動
```

に近い構造になっている。

v0.1.4では、

```text
Shellを検出
    ↓
Shellを分類
    ↓
環境を正規化
    ↓
起動引数を決定
    ↓
Health Check
    ↓
正常ならPTY起動
    ↓
失敗なら次のShellへfallback
```

という構造へ変更する。

これにより、単なる `0xc0000142` 対策だけではなく、

* 壊れたPATH
* 壊れたPowerShell Profile
* PowerShell 7未インストール
* Windows PowerShellのみ存在
* Git Bashのみ存在
* WSLのみ利用可能
* Shell起動失敗
* Windows環境変数の異常

などにも対応できる。

---

# 30. 最重要修正箇所

今回の問題について、最初に修正するべきファイルは以下の2つ。

```text
src/terminal/pty.rs
src/shell/windows.rs
```

特に、

```text
src/terminal/pty.rs
```

の

```rust
std::env::vars()
```

による環境変数の無条件コピーと、

```text
src/shell/windows.rs
```

のWindows環境変数・Shell探索処理を見直す。

その後、

```text
src/shell/health.rs
src/shell/environment.rs
```

を追加して責務を分離する。

---

# 31. 修正方針の要約

今回の `0xc0000142` は、

```text
「PowerShellが壊れている」
```

と決めつけるのではなく、

```text
cmdshell
   ↓
PTY
   ↓
Windows Process Creation
   ↓
PowerShell
   ↓
Environment / DLL Initialization
```

というプロセス全体を診断する。

最終的には、

```text
PowerShell 7
      ↓
  Health Check
      ↓
    PASS
      ↓
     PTY
```

を正常系とし、

```text
PowerShell 7
      ↓
    FAIL
      ↓
PowerShell 5.1
      ↓
    FAIL
      ↓
    CMD
      ↓
    PASS
```

のような自動fallbackを実装する。

これを `cmdshell v0.1.4` のWindows Shell起動基盤として実装する。

# cmdshell アプリケーションウィンドウ化 追加計画書

## 1. 目的

`cmdshell` を現在の「ターミナルから起動して使用するCLIツール」から、**Windows上で独立したアプリケーションウィンドウとして起動するターミナルアプリケーション**へ変更する。

最終的には、ユーザーが

```text
cmdshell.exe
```

をダブルクリックするだけで、

```text
┌─────────────────────────────────────────────────────────────┐
│ cmdshell                                      ─ □ ×       │
├─────────────────────────────────────────────────────────────┤
│  PowerShell                                                 │
├─────────────────────────────────────────────────────────────┤
│ PS C:\Users\User> git sta▌                                  │
│                                                             │
│                                                             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

のような独立したGUIアプリケーションを起動できる状態を目標とする。

---

# 2. 現在の問題

現在の `cmdshell` はターミナルプロセスから起動することを前提としている。

概念的には、

```text
cmd.exe / PowerShell
        ↓
    cmdshell
        ↓
       PTY
        ↓
   PowerShell
```

となっている。

この構造では、

* cmdshell自身のコンソールウィンドウが存在する
* GUIとしてのウィンドウ管理ができない
* メニューバーを持たせにくい
* タブ機能を実装しにくい
* 設定画面を作りにくい
* ウィンドウサイズとPTYサイズの同期が複雑
* Windowsアプリケーションとしての配布性が低い

という問題がある。

---

# 3. 新しいアーキテクチャ

アプリケーションウィンドウ化後は以下の構造にする。

```text
cmdshell.exe
      │
      ▼
┌───────────────────┐
│ Application       │
│                   │
│ ┌───────────────┐ │
│ │ Terminal UI   │ │
│ └───────┬───────┘ │
└─────────┼─────────┘
          │
          ▼
      Terminal Core
          │
          ▼
        PTY
          │
          ▼
     Shell Process
          │
     ┌────┴─────┐
     ▼          ▼
 PowerShell     CMD
```

重要なのは、

```text
GUI
Terminal Core
PTY
Shell
```

を分離することである。

---

# 4. GUIフレームワーク

GUI部分には **GPUI** を使用する。

既存のプロジェクト方針として、ターミナル描画をGPUアクセラレーション可能なGUIで実装する。

構成:

```text
GPUI
  │
  ├── Window
  ├── Terminal View
  ├── Tab Bar
  ├── Command Input
  ├── Suggestion Popup
  └── Settings
```

---

# 5. アプリケーション起動方式

## 5.1 Windows

最終的には、

```text
cmdshell.exe
```

をExplorerからダブルクリックするだけでGUIを起動する。

理想的にはコンソールウィンドウを表示しない。

RustのWindowsビルドではGUI subsystemを利用する。

Cargo設定・Windows linker設定を適切に行い、

```text
SUBSYSTEM:WINDOWS
```

となる構成を検討する。

---

# 6. CLIモードも維持する

GUI化してもCLI機能を完全に削除しない。

以下の2モードを提供する。

```text
GUI Mode
cmdshell.exe

CLI Mode
cmdshell.exe --cli
```

または、

```text
cmdshell.exe --help
cmdshell.exe doctor
cmdshell.exe --version
```

などの管理系コマンドを提供する。

---

# 7. GUIとCLIの責務

## GUI

GUIは以下を担当する。

* ウィンドウ
* 描画
* キーボード入力
* マウス入力
* タブ
* メニュー
* 設定
* Terminal表示
* 候補表示

## CLI

CLIは以下を担当する。

* `doctor`
* `version`
* 設定確認
* デバッグ
* 診断
* ログ出力

## Terminal Core

GUIとCLIの間に共通のTerminal Coreを置く。

```text
GUI
 │
 ├──────┐
 │      │
 ▼      ▼
Terminal Core
 │
 ▼
PTY
 │
 ▼
Shell
```

これによりGUI専用のターミナルロジックにならないようにする。

---

# 8. Terminal Core

Terminal Coreでは以下を管理する。

```text
TerminalSession
TerminalBuffer
TerminalParser
TerminalInput
TerminalOutput
TerminalSize
TerminalCursor
TerminalSelection
```

---

# 9. TerminalSession

```rust
pub struct TerminalSession {
    shell: ShellInfo,
    pty: PtySession,
    buffer: TerminalBuffer,
    cursor: CursorState,
}
```

責務:

* Shell起動
* PTY管理
* 入力送信
* 出力受信
* サイズ変更
* Shell終了検出

---

# 10. TerminalBuffer

GUIとShellを完全に分離するため、ターミナルの状態をバッファとして保持する。

```text
TerminalBuffer
├── rows
├── columns
├── cells
├── cursor
├── selection
└── scrollback
```

各Cellは最低限、

```text
character
foreground
background
attributes
```

を持つ。

---

# 11. ANSI / VTエスケープシーケンス

PowerShellやCMDは単なる文字列ではなく、ANSI/VTシーケンスを出力する。

そのため、

```text
Shell Output
     ↓
VT Parser
     ↓
Terminal State
     ↓
GPUI Renderer
```

とする。

対応対象:

* ANSI color
* cursor movement
* cursor visibility
* clear screen
* clear line
* scroll
* bold
* underline
* inverse
* 256 colors
* true color
* alternate screen

---

# 12. GPUI描画

GPUIではTerminalBufferを直接描画する。

```text
TerminalBuffer
      ↓
Visible Rows
      ↓
Visible Cells
      ↓
Text Rendering
      ↓
GPU
      ↓
Window
```

大量の文字を1文字ずつ独立したUIコンポーネントとして生成しない。

代わりに、

```text
Terminal Grid
```

として効率的に描画する。

---

# 13. フォント

初期フォント:

```text
Cascadia Mono
Consolas
JetBrains Mono
```

などを候補とする。

ユーザーが設定画面から変更できるようにする。

必要な設定:

```text
Font Family
Font Size
Line Height
Letter Spacing
```

---

# 14. ウィンドウ構成

基本UI:

```text
┌────────────────────────────────────────────────────┐
│ cmdshell                              ─ □ ×       │
├────────────────────────────────────────────────────┤
│ + │ PowerShell                         ×           │
├────────────────────────────────────────────────────┤
│                                                    │
│ PS C:\Users\User> git status                       │
│                                                    │
│ On branch main                                     │
│                                                    │
│ PS C:\Users\User> ▌                               │
│                                                    │
├────────────────────────────────────────────────────┤
│ PowerShell                         UTF-8   120×40 │
└────────────────────────────────────────────────────┘
```

---

# 15. タブ機能

将来的には複数Shellを同一ウィンドウで管理する。

```text
┌───────────────────────────────────────────────┐
│ + │ PowerShell │ CMD │ WSL │ Git Bash        │
├───────────────────────────────────────────────┤
│                                               │
│                                               │
└───────────────────────────────────────────────┘
```

各タブは独立した、

```text
TerminalSession
```

を持つ。

---

# 16. 新しいターミナル

`+` ボタンから新しいShellを起動する。

例:

```text
New Terminal
├── PowerShell 7
├── Windows PowerShell
├── Command Prompt
├── Git Bash
└── WSL
```

---

# 17. Shell選択

デフォルト:

```text
PowerShell 7
```

PowerShell 7が存在しない場合:

```text
Windows PowerShell
```

それも失敗した場合:

```text
CMD
```

というfallbackを行う。

これは `FIX_0.1.3.md` で定義したShell Health Checkと統合する。

---

# 18. ウィンドウサイズとPTYサイズ

GUIターミナルでは非常に重要。

ウィンドウサイズ:

```text
Width
Height
```

から、

```text
Character Width
Character Height
```

を計算する。

例えば、

```text
Window
1200 × 800 px

Font
10 × 20 px
```

なら、

```text
Columns = 120
Rows    = 40
```

となる。

ウィンドウサイズ変更時には、

```text
GPUI Window Resize
       ↓
Terminal Size Calculation
       ↓
PTY Resize
       ↓
Shell receives SIGWINCH / Windows equivalent
```

を行う。

---

# 19. キーボード入力

最低限以下をサポートする。

```text
文字入力
Enter
Backspace
Delete
Tab
Ctrl+C
Ctrl+D
Ctrl+Z
Ctrl+L
Arrow Keys
Home
End
PageUp
PageDown
Insert
```

Windows固有:

```text
Ctrl+Shift+C
Ctrl+Shift+V
```

をコピー・貼り付けに使用する。

---

# 20. マウス操作

最低限:

* テキスト選択
* コピー
* 貼り付け
* スクロール
* 右クリックメニュー

を実装する。

---

# 21. スクロールバック

ターミナルの過去出力を保持する。

設定例:

```text
Scrollback Lines
1000
5000
10000
Unlimited
```

初期値:

```text
10000
```

を推奨する。

---

# 22. コマンド予測変換

cmdshellの主要機能である予測変換をGUI側に統合する。

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

表示:

```text
┌──────────────────────┐
│ git status           │
│ git stash            │
│ git stage            │
└──────────────────────┘
```

---

# 23. 予測候補の操作

```text
↑ / ↓
```

で候補選択。

```text
Tab
```

または

```text
Right Arrow
```

で候補を確定。

```text
Esc
```

で候補を閉じる。

---

# 24. 予測変換とShellの分離

予測エンジンはShellプロセスに直接組み込まない。

```text
Terminal Input
      │
      ├──────────────┐
      ▼              ▼
Prediction Engine   Shell
      │              │
      ▼              ▼
Suggestion UI      PTY
```

とする。

これにより、

```text
PowerShell
CMD
Bash
WSL
```

すべてで共通の予測エンジンを利用できる。

---

# 25. コマンド履歴

履歴を予測に利用する。

例:

```text
git status
git add .
git commit
cargo build
cargo test
docker compose up
```

頻繁に使用するコマンドほど候補上位に表示する。

---

# 26. 設定

設定画面を追加する。

```text
Settings
├── Appearance
│   ├── Theme
│   ├── Font
│   ├── Font Size
│   └── Cursor
│
├── Terminal
│   ├── Default Shell
│   ├── Scrollback
│   └── Bell
│
├── Prediction
│   ├── Enabled
│   ├── History
│   └── Suggestions
│
└── Advanced
    ├── Environment
    ├── Debug Logging
    └── PTY
```

---

# 27. テーマ

初期テーマ:

```text
Dark
Light
```

将来的には、

```text
Dracula
Nord
Tokyo Night
Catppuccin
```

などを追加できる構造にする。

---

# 28. アプリケーションアイコン

Windows Explorer上で、

```text
cmdshell.exe
```

が通常のアプリケーションとして認識されるようにする。

必要:

```text
.ico
```

を用意する。

アプリケーションウィンドウにもアイコンを設定する。

---

# 29. Windowsタスクバー

GUIアプリケーション化後はタスクバーから、

```text
cmdshell
```

として起動・切り替えできるようにする。

将来的には、

```text
タスクバー右クリック
    ↓
New Terminal
New PowerShell
New CMD
```

なども検討する。

---

# 30. CLIコンソールの非表示

GUIモードでは、

```text
cmdshell.exe
```

起動時に黒いコンソールウィンドウが表示されないようにする。

一方、

```text
cmdshell.exe --cli
cmdshell.exe doctor
```

ではCLIとして実行できるようにする。

WindowsではGUI subsystemとCLI subsystemの扱いに注意する。

---

# 31. エラーウィンドウ

Shell起動失敗時にコンソールへエラーを出すのではなく、GUIダイアログを表示する。

例:

```text
┌───────────────────────────────────────────┐
│ Shell Startup Failed                     │
├───────────────────────────────────────────┤
│ PowerShell 7 could not be started.       │
│                                           │
│ Error: 0xc0000142                        │
│ STATUS_DLL_INIT_FAILED                   │
│                                           │
│ [Try another shell] [Diagnostics] [OK]   │
└───────────────────────────────────────────┘
```

---

# 32. Diagnostics

`Diagnostics`を押した場合、

```text
Shell
Executable
Environment
PATH
SystemRoot
PTY
Exit Code
```

を確認できるようにする。

これを既存の `doctor` 機能と共通化する。

---

# 33. プロジェクト構造

最終的には以下を目標とする。

```text
src/
├── main.rs
│
├── app/
│   ├── mod.rs
│   ├── application.rs
│   ├── window.rs
│   └── settings.rs
│
├── ui/
│   ├── mod.rs
│   ├── terminal_view.rs
│   ├── tab_bar.rs
│   ├── suggestion_popup.rs
│   ├── status_bar.rs
│   └── dialogs.rs
│
├── terminal/
│   ├── mod.rs
│   ├── session.rs
│   ├── pty.rs
│   ├── buffer.rs
│   ├── parser.rs
│   ├── input.rs
│   └── selection.rs
│
├── shell/
│   ├── mod.rs
│   ├── resolver.rs
│   ├── health.rs
│   ├── environment.rs
│   ├── windows.rs
│   └── unix.rs
│
├── prediction/
│   ├── mod.rs
│   ├── engine.rs
│   ├── history.rs
│   └── ranking.rs
│
└── cli/
    ├── mod.rs
    ├── doctor.rs
    └── commands.rs
```

---

# 34. 実装フェーズ

## Phase 1 — GUI基盤

* [ ] GPUI導入
* [ ] Application作成
* [ ] Window作成
* [ ] WindowsでGUI起動
* [ ] コンソールウィンドウ非表示
* [ ] アプリケーションアイコン

完成:

```text
cmdshell.exe
    ↓
GUI Window
```

---

## Phase 2 — Terminal Core統合

* [ ] PTY起動
* [ ] Shell起動
* [ ] stdout受信
* [ ] stdin送信
* [ ] TerminalBuffer
* [ ] VT Parser
* [ ] Cursor
* [ ] Resize

完成:

```text
GUI
 ↓
Terminal
 ↓
PowerShell
```

---

## Phase 3 — Windows Shell対応

* [ ] PowerShell 7
* [ ] PowerShell 5.1
* [ ] CMD
* [ ] Git Bash
* [ ] WSL
* [ ] Shell Health Check
* [ ] `0xc0000142` fallback

---

## Phase 4 — Terminal UI

* [ ] ターミナル描画
* [ ] カーソル
* [ ] 色
* [ ] ANSI
* [ ] 選択
* [ ] コピー
* [ ] 貼り付け
* [ ] スクロール
* [ ] Scrollback

---

## Phase 5 — タブ

* [ ] Tab UI
* [ ] New Terminal
* [ ] Close Terminal
* [ ] TerminalSession管理
* [ ] Shellごとの独立PTY

---

## Phase 6 — 予測変換

* [ ] Suggestion UI
* [ ] コマンド履歴
* [ ] 候補ランキング
* [ ] Tab補完
* [ ] Shell入力との統合

---

## Phase 7 — 設定

* [ ] Theme
* [ ] Font
* [ ] Font Size
* [ ] Default Shell
* [ ] Scrollback
* [ ] Prediction
* [ ] Keybindings

---

## Phase 8 — 診断

* [ ] GUI Doctor
* [ ] Shell診断
* [ ] PTY診断
* [ ] Environment診断
* [ ] Error Dialog
* [ ] Debug Log

---

# 35. パフォーマンス要件

ターミナルは大量の文字を処理するため、UIのパフォーマンスを重視する。

目標:

```text
通常入力:
< 16 ms

描画:
60 FPS

大量出力:
UIフリーズしない

100,000行出力:
クラッシュしない

Scrollback:
高速スクロール可能
```

特に、

```text
cat large_file
cargo build
git log
docker logs
```

などを実行した場合でもGUIが固まらないことを目標とする。

---

# 36. 非同期処理

Shell出力をUIスレッドで直接読み込まない。

```text
PTY Reader Thread
       ↓
Output Channel
       ↓
Terminal Core
       ↓
GPUI Event
       ↓
UI Update
```

とする。

UIスレッドをブロックしないこと。

---

# 37. 大量出力対策

大量の出力を一度にUIへ渡さない。

例えば、

```text
PTY
 ↓
Output Buffer
 ↓
Batch
 ↓
Terminal Parser
 ↓
UI
```

とする。

1回のイベントで大量の文字列を処理してUIが固まることを防ぐ。

---

# 38. 入力遅延対策

ユーザーが入力した文字は、

```text
Keyboard
 ↓
Prediction
 ↓
Terminal Input
```

を高速に処理する。

予測処理が重い場合は非同期化する。

```text
Input
 ├── Shell
 └── Prediction Worker
```

とする。

---

# 39. クラッシュ対策

ShellがクラッシュしてもGUI全体は終了させない。

```text
PowerShell
   ↓
Crash
   ↓
TerminalSession終了
   ↓
GUIは維持
   ↓
[Restart Shell]
```

とする。

---

# 40. Shell終了後のUI

Shell終了時:

```text
┌──────────────────────────────────────────┐
│ PowerShell                               │
├──────────────────────────────────────────┤
│                                          │
│ PS C:\Users\User> exit                   │
│                                          │
│ [Shell exited]                           │
│                                          │
│ [Restart]                                │
└──────────────────────────────────────────┘
```

---

# 41. GUIショートカット

推奨:

| ショートカット          | 機能       |
| ---------------- | -------- |
| `Ctrl+Shift+T`   | 新しいタブ    |
| `Ctrl+Shift+W`   | タブを閉じる   |
| `Ctrl+Tab`       | 次のタブ     |
| `Ctrl+Shift+Tab` | 前のタブ     |
| `Ctrl+Shift+C`   | コピー      |
| `Ctrl+Shift+V`   | 貼り付け     |
| `Ctrl+,`         | 設定       |
| `Ctrl+Shift+P`   | コマンドパレット |
| `Ctrl+L`         | 入力行クリア   |

---

# 42. コマンドパレット

将来的に、

```text
Ctrl+Shift+P
```

でコマンドパレットを表示する。

例:

```text
┌──────────────────────────────────────┐
│ > new terminal                       │
├──────────────────────────────────────┤
│ New PowerShell                       │
│ New CMD                              │
│ New WSL                              │
│ Open Settings                        │
│ Toggle Prediction                    │
│ Clear Terminal                       │
└──────────────────────────────────────┘
```

---

# 43. 将来的な拡張

アプリケーションウィンドウ化を基盤として、将来的に以下を実装できる。

* [ ] 複数タブ
* [ ] Split Pane
* [ ] 複数ウィンドウ
* [ ] コマンドパレット
* [ ] 高度な予測変換
* [ ] コマンド履歴検索
* [ ] AIによるコマンド候補
* [ ] SSH接続
* [ ] SFTP
* [ ] Dockerコンテナ接続
* [ ] WSL統合
* [ ] リモートターミナル
* [ ] テーマ
* [ ] プラグイン
* [ ] 設定同期

---

# 44. v0.2.0の目標

アプリケーションウィンドウ化を大きな機能として、

```text
v0.1.x
│
├── CLI
├── PTY
└── Shell
       ↓
v0.2.0
│
├── GPUI
├── Application Window
├── Terminal UI
├── Shell
├── PTY
├── Prediction
└── Tabs
```

を目標とする。

---

# 45. 完成イメージ

最終的なWindows版cmdshellは、

```text
                    cmdshell.exe
                         │
                         ▼
              ┌─────────────────────┐
              │     cmdshell        │
              ├─────────────────────┤
              │ + │ PowerShell │ ×  │
              ├─────────────────────┤
              │                     │
              │ PS C:\Users\User>   │
              │ git st              │
              │                     │
              │ ┌─────────────────┐ │
              │ │ git status      │ │
              │ │ git stash       │ │
              │ │ git stage       │ │
              │ └─────────────────┘ │
              │                     │
              ├─────────────────────┤
              │ PowerShell │ 120×40 │
              └─────────────────────┘
```

という、通常のWindowsアプリケーションとして利用できるターミナルを目指す。

---

# 46. 最重要設計原則

この変更では、GUIコードとTerminal Coreを混在させない。

```text
❌ 悪い構造

GPUI
 └── PTY
      └── PowerShell
```

ではなく、

```text
推奨:

┌───────────────┐
│      UI       │
│     GPUI      │
└───────┬───────┘
        │
┌───────▼───────┐
│ Terminal Core │
└───────┬───────┘
        │
┌───────▼───────┐
│      PTY      │
└───────┬───────┘
        │
┌───────▼───────┐
│     Shell     │
└───────────────┘
```

とする。

この構造にすることで、将来的にWindowsだけでなくLinux/macOSにもGUIターミナルを展開できる。

---

# 47. 実装順序

実装は以下の順番を推奨する。

```text
1. GPUI Application
        ↓
2. Window
        ↓
3. TerminalBuffer
        ↓
4. VT Parser
        ↓
5. PTY
        ↓
6. PowerShell
        ↓
7. Keyboard Input
        ↓
8. Resize
        ↓
9. Selection / Copy / Paste
        ↓
10. Prediction UI
        ↓
11. Tabs
        ↓
12. Settings
        ↓
13. Doctor
        ↓
14. Packaging
```

---

# 48. v0.2.0 完成条件

以下をすべて満たした時点でGUI版の初期完成とする。

```text
[ ] cmdshell.exeをダブルクリックして起動できる
[ ] コンソールウィンドウが表示されない
[ ] GPUIウィンドウが表示される
[ ] PowerShell 7を起動できる
[ ] PowerShell 5.1を起動できる
[ ] CMDを起動できる
[ ] PTYが正常に動作する
[ ] ANSIカラーが表示される
[ ] カーソルが表示される
[ ] キーボード入力ができる
[ ] Ctrl+Cが動作する
[ ] ウィンドウリサイズにPTYが追従する
[ ] テキスト選択ができる
[ ] コピーできる
[ ] 貼り付けできる
[ ] スクロールできる
[ ] Scrollbackが動作する
[ ] コマンド予測が表示される
[ ] Tab補完が動作する
[ ] 複数タブを開ける
[ ] Shell終了後にGUIがクラッシュしない
[ ] Shell起動失敗時にfallbackする
[ ] 0xc0000142を検出できる
[ ] DoctorからShell状態を確認できる
[ ] Windows用Release Buildが作成できる
[ ] Explorerから直接起動できる
```

---

# 49. 最終目標

`cmdshell` は単なるCLIツールではなく、

> **Rust + GPUIで構築された、予測変換機能を持つネイティブターミナルアプリケーション**

を目指す。

中心となる設計は、

```text
                 cmdshell
                     │
          ┌──────────┴──────────┐
          │                     │
       GUI Mode              CLI Mode
          │                     │
        GPUI                 Commands
          │                     │
          └──────────┬──────────┘
                     │
              Terminal Core
                     │
                    PTY
                     │
              Shell Resolver
                     │
        ┌────────────┼────────────┐
        │            │            │
    PowerShell      CMD          WSL
```

とする。

既存の `0xc0000142` 修正計画はこのTerminal Core / Shell Resolver層に組み込み、GUI化によって別実装を作るのではなく、**同じTerminal CoreをGUIから利用する設計**にする。

これにより、v0.1.3の不具合修正とv0.2.0のGUI化を別々のコードとして実装するのではなく、今後のcmdshellの基盤として統合できる。
