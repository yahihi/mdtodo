# mdtodo - セクション付き Markdown TODO CLI (Rust)

Obsidian / Obsidian Tasks 互換の Markdown TODO ファイルを壊さずに編集する Rust CLI。

## 特徴

- ✅ セクション方式のタスク管理
- ✅ Obsidian / Obsidian Tasks 互換
- ✅ Markdown ファイルの構造を維持
- ✅ 動的な番号付け（実行時生成）
- ✅ 完了タスクのアーカイブ機能
- ✅ ゼロ設定で動作

## インストール

```bash
cargo install --path .
```

## 使い方

### 初期化

```bash
mdtodo init
```

デフォルトのセクション（Today, Next, Backlogs, Someday, Waiting, Inbox）を含む TODO.md を作成します。

### タスク一覧表示

```bash
# 全セクションのタスクを表示
mdtodo list

# 特定セクションのみ表示
mdtodo list Today
```

### タスクの追加

```bash
mdtodo add Today "契約書レビュー"
mdtodo add PROJECT1 "API設計"
```

### タスクの完了

```bash
mdtodo done Today:1
```

完了マークと完了日（`✅ YYYY-MM-DD`）が自動付与されます。

### 完了の取り消し

```bash
mdtodo undo Today:1
```

### タスクの移動（核心機能）

```bash
# 単一タスクを移動
mdtodo move Next:3 Today

# 複数タスクを一度に移動
mdtodo move Backlog:1,4 Today
```

移動先セクションが存在しない場合は自動作成されます。

### タスクのアーカイブ

```bash
# 個別アーカイブ
mdtodo archive Today:3,7

# セクション内の完了済みタスクを一括アーカイブ
mdtodo archive Today:all
```

完了済みタスクのみアーカイブ可能です（未完了タスクはエラーになります）。

## ファイル構造

### TODO.md

```markdown
# TODO

## Today
- [ ] タスク 1
- [x] タスク 2 ✅ 2026-02-13

## Next
- [ ] タスク 3

## PROJECT1
- [ ] API設計
```

### done_list.md

完了日とセクションで整理されます：

```markdown
# Done Log

## 2026-02-13

### Today
- [x] タスク 2 ✅ 2026-02-13

### PROJECT1
- [x] API設計 ✅ 2026-02-13

## 2026-02-12

### Today
- [x] 古いタスク ✅ 2026-02-12
```

## 設定

`~/.config/mdtodo/config.toml` に配置（オプション）：

```toml
todo_path = "~/Obsidian/TODO.md"
done_path = "~/Obsidian/done_list.md"
timezone = "Asia/Tokyo"
```

設定ファイルがない場合はデフォルト値（`./TODO.md` と `./done_list.md`）を使用します。

## 開発

### ビルド

```bash
cargo build
```

### テスト

```bash
cargo test
```

### リリースビルド

```bash
cargo build --release
```

## ライセンス

MIT
