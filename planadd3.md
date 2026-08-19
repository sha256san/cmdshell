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
