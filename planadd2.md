# PredictTerm — Windows シェル起動エラー (0xc0000142) 対策 & 堅牢化設計書 (planadd2.md)

## 1. 概要とエラー分析

### 1.1 エラーの概要
Windows 環境において PredictTerm を起動、またはターミナルセッションを新規作成した際に以下のダイアログが表示される事象：
```text
powershell.exe - アプリケーション エラー
アプリケーションを正しく起動できませんでした (0xc0000142)。[OK] をクリックしてアプリケーションを閉じてください。
```

### 1.2 0xc0000142 (STATUS_DLL_INIT_FAILED) のメカニズムと根本原因
NTSTATUS `0xC0000142` は、プロセス起動時に読み込まれる Dynamic Link Library (DLL) の初期化ルーチン (`DllMain`) が失敗したことを示します。ターミナルエミュレータ・PTY経由での起動で本エラーが発生する主因は以下の4点です：

1. **必須環境変数の欠落 (Missing Essential Environment Variables)**
   - PTY プロセス生成時に子プロセスに渡す環境変数から `SystemRoot` (例: `C:\Windows`)、`WINDIR`、`SystemDrive`、`PATH`、`USERPROFILE`、`LOCALAPPDATA` が欠落または破損している場合、`ntdll.dll` / `kernel32.dll` / `.NET CLR` の初期化が失敗し、即座に `0xc0000142` が送出される。
2. **シェル実行可能ファイルの相対パス解決と App Execution Aliases の競合**
   - 単に `"powershell.exe"` と指定して起動した場合、`PATH` の探索順序や UWP/AppX Execution Aliases、または管理者権限/サンドボックス境界との不整合により初期化が失敗する。
3. **ConPTY (PseudoConsole) のハンドル継承・バッファ初期化シーケンスの不備**
   - PTY の初期ウィンドウサイズ (0x0) や、標準入出力ハンドルの接続タイミングによるプロセス早期クラッシュ。
4. **PowerShell のバージョン混在と代替シェルの未フォールバック**
   - システム環境によって `PowerShell 7 (pwsh.exe)`、`Windows PowerShell 5.1 (powershell.exe)`、またはグループポリシーにより一部のシェルが制限されている場合に、安全な代替シェル (`cmd.exe`) へのフォールバック機構が存在しない。

---

## 2. 解決方針とアーキテクチャ

```text
┌─────────────────────────────────────────────────────────────┐
│                     TerminalSession                         │
└──────────────────────────────┬──────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                 WindowsShellResolver                        │
│                                                             │
│  1. 優先順位に応じたシェル検出 (Config -> pwsh -> powershell -> cmd)│
│  2. 絶対パスの解決 (System32 / Program Files)               │
│  3. 必須環境変数の完全補完 (SystemRoot, PATH, etc.)         │
└──────────────────────────────┬──────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    Robust PtyBackend                        │
│                                                             │
│  - ConPTY 安全起動 (適切な初期サイズ 80x24)                  │
│  - 0xc0000142 / 起動失敗時の多段階自動フォールバック        │
│  - エラーをクラッシュさせずに UI 通知する機構               │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. 具体的な実装設計

### 3.1 必須環境変数の保護と自動補完 (`src/terminal/backend.rs` または `src/terminal/pty.rs`)
Windows で PTY を起動する際、親プロセスの環境変数を確実に継承し、万一欠落している場合でも以下の最小必須変数を補完する：

```rust
pub fn sanitize_windows_environment(cmd: &mut CommandBuilder) {
    #[cfg(windows)]
    {
        let system_root = std::env::var("SystemRoot")
            .unwrap_or_else(|_| "C:\\Windows".to_string());
        cmd.env("SystemRoot", &system_root);
        cmd.env("WINDIR", &system_root);
        
        if std::env::var("SystemDrive").is_err() {
            cmd.env("SystemDrive", "C:");
        }
        if std::env::var("ComSpec").is_err() {
            cmd.env("ComSpec", format!("{}\\System32\\cmd.exe", system_root));
        }
    }
}
```

### 3.2 シェル検出・絶対パス解決 (`src/shell/mod.rs` & `src/shell/windows.rs`)
相対名ではなく、ディスク上に実在する安全な絶対パスを順次検証して選択する：

```text
[検出優先順位]
1. ユーザー設定の `terminal.shell`
2. PowerShell 7+ (`%ProgramFiles%\PowerShell\7\pwsh.exe`)
3. Windows PowerShell (`%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe`)
4. Command Prompt (`%SystemRoot%\System32\cmd.exe`) - 最も安定・軽量
5. Git Bash (`%ProgramFiles%\Git\bin\bash.exe`) / WSL (`%SystemRoot%\System32\wsl.exe`)
```

### 3.3 多段階フォールバック (Multi-tier Fallback)
1番目のシェルが `0xc0000142` や `NotFound` 等で失敗した場合、即座に次善のシェル（例: `cmd.exe`）で PTY 起動を再試行し、ユーザーに警告バナーを表示する。

### 3.4 診断コマンド (`predictterm doctor`) の機能拡張
- Windows 環境における各シェル実行可能ファイル（PowerShell, pwsh, cmd, WSL）の存在チェックと起動テスト。
- `SystemRoot` / `PATH` 等の環境変数の整合性チェック。

---

## 4. 変更対象ファイル一覧

| 変更区分 | ファイルパス | 内容 |
| :--- | :--- | :--- |
| **[NEW]** | `src/shell/mod.rs` | シェル管理・抽象化モジュール |
| **[NEW]** | `src/shell/windows.rs` | Windows 向け絶対パス解決・環境変数補正 |
| **[MODIFY]** | `src/terminal/pty.rs` | 環境変数のサニタイズ・多段階フォールバック実装 |
| **[MODIFY]** | `src/main.rs` | `predictterm doctor` に Windows シェル診断を追加 |
| **[MODIFY]** | `src/config/settings.rs` | シェル設定の柔軟化 |
| **[NEW]** | `tests/shell_tests.rs` | シェル検出・フォールバックの単体テスト |
| **[MODIFY]** | `CHANGELOG.md` | バージョン更新履歴の記載 |

---

## 5. ロードマップとタスク

- [ ] **Task 1**: `src/shell/` モジュールを新設し、クロスプラットフォームの安全なシェル解決ロジックを実装
- [ ] **Task 2**: `PtyBackend::spawn` に環境変数補正と起動失敗時のフォールバック処理を統合
- [ ] **Task 3**: `tests/shell_tests.rs` でシェル検出および環境変数補正のテストケースを追加
- [ ] **Task 4**: `doctor` サブコマンドに Windows 固有の環境変数・シェル整合性診断を追加
---

## 6. GitHub Actions CI/CD Windows ビルド (0xc0000142 / exit code 1) トラブルシューティング

### 6.1 現象
GitHub Actions における `Build (x86_64-pc-windows-msvc)` ジョブが `exit code 1` で失敗。

### 6.2 根本原因の特定
1. **`test_windows_essential_env_injection` のアサーション失敗**:
   - `src/shell/windows.rs` の `ensure_essential_windows_env` 内で `if std::env::var_os("SystemDrive").is_none()` という条件判定を行っていた。
   - Linux ランナー上では `SystemDrive` / `ComSpec` が未設定のためブロックが実行されテストが通過していたが、実機 Windows ランナー上では既に環境変数が存在するため `is_none()` が false となり、`env_setter` が呼び出されなかった。
   - その結果、`tests/shell_tests.rs` の `assert!(map.contains_key("SystemDrive"))` が Windows 上で panic を引き起こしていた。

2. **PowerShell `Compress-Archive` の上書き競合**:
   - `Compress-Archive` において `-Force` オプションが不足しており、アーティファクト生成時の例外要因となっていた。

### 6.3 適用した修正
- `ensure_essential_windows_env` を改修し、環境変数の有無に関わらず安全な値（既存値またはフォールバック）を確実にセットするよう修正。
- `Compress-Archive` に `-Force` フラグを追加。
- バージョンを `0.1.2` (Pre-release) にインクリメント。
