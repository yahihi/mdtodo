# mdtodo - セクション付き Markdown TODO CLI (Rust)

Obsidian / Obsidian Tasks 互換の Markdown TODO ファイルを壊さずに編集する Rust CLI を実装せよ。

## 対象ファイル

- `TODO.md`: 現行タスク（セクション方式）
- `done_list.md`: 完了アーカイブ（単一ファイル）

## タスク表現（Markdown互換）

- 未完了: `- [ ] ...`
- 完了: `- [x] ...`
- 完了日付与: `✅ YYYY-MM-DD`（`done` 実行時に CLI が付与）

## セクション方式

`## Today` / `## Next` / `## Backlogs` / `## Someday` / `## Waiting` / `## Inbox` / `## PROJECT1` など任意セクション。

## 付番方式

- セクションごとに動的番号付与（実行時生成、ファイルに書かない）
- 完了・未完了を問わずセクション内のタスクに番号を振る

## コマンド

### `mdtodo list [Section]`
タスク一覧表示。セクション指定で絞り込み。番号付きで表示。

### `mdtodo add <Section> <text>`
指定セクションにタスク追加。

### `mdtodo done <Section>:<number>`
タスクを完了にする（`- [ ]` → `- [x]`）。`✅ YYYY-MM-DD` を付与。TODO.md 内に残す。

### `mdtodo undo <Section>:<number>`
完了を取り消す（`- [x]` → `- [ ]`）。`✅` を除去。

### `mdtodo move <Section>:<number> <DestSection>`
タスクをセクション間で移動する。これが本CLIの核心機能。
- 例: `mdtodo move Next:3 Today`（Next の 3番を Today に移動）
- 例: `mdtodo move Backlog:1,4 Today`（複数指定可）
- 移動先セクションが存在しなければ新規作成する
- 移動結果（タスクテキスト・移動元・移動先）を明示出力する

### `mdtodo archive <Section>:<numbers|all>`
完了済みタスクを TODO.md から done_list.md に移動。
- 個別: `mdtodo archive Today:3,7`
- 一括: `mdtodo archive Today:all`（完了済みのみ）
- 未完了は許可しない（事故防止）

### `mdtodo init`
TODO.md を初期テンプレートで作成（設定ファイルの todo_path の場所に作成）。

## done_list.md の構造

完了日（✅）基準 × 元セクション:

```md
# Done Log

## 2026-02-12

### Today
- [x] 契約書レビュー ✅ 2026-02-12

### PROJECT1
- [x] API設計 ✅ 2026-02-12
```

## 設定ファイル

`~/.config/mdtodo/config.toml` に配置。ゼロ設定で動作する（ファイルがなければデフォルト値を使う）。

```toml
todo_path = "~/Obsidian/TODO.md"
done_path = "~/Obsidian/done_list.md"
timezone = "Asia/Tokyo"
```

- `todo_path`: TODO.md のパス（デフォルト: `./TODO.md`）
- `done_path`: done_list.md のパス（デフォルト: `./done_list.md`）
- `timezone`: 完了日付のタイムゾーン（デフォルト: ローカル）
- `~` はホームディレクトリに展開する

## 安全性

- 操作結果は必ず対象タスクのテキストとセクションを明示出力する
- `archive` は未完了タスクを許可しない

## 技術要件

- Rust (cargo プロジェクト)
- CLI パーサー: clap
- テスト: `cargo test` で全テストパス
- `cargo build` が成功すること
