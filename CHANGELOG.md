# Changelog

## v0.1.0 (2026-02-13)

初回リリース。

### 機能

- `mdtodo init` — デフォルトセクション付き TODO.md の初期化
- `mdtodo list [Section]` — セクション別タスク一覧（動的番号付き）
- `mdtodo add <Section> <text>` — タスク追加（セクション自動作成）
- `mdtodo done <Section:number>` — タスク完了（`✅ YYYY-MM-DD` 自動付与）
- `mdtodo undo <Section:number>` — 完了取り消し
- `mdtodo move <Section:number[,...]> <Dest>` — セクション間移動（複数指定可）
- `mdtodo archive <Section:number[,...]|all>` — 完了タスクを done_list.md にアーカイブ

### 設計

- Obsidian / Obsidian Tasks 互換の Markdown フォーマット
- セクションごとの動的番号付け（ファイルには書かない）
- `~/.config/mdtodo/config.toml` によるゼロ設定動作
- 未完了タスクのアーカイブ防止（安全機構）
