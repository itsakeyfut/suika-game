# スイカゲーム - クイックリファレンス

## クレート構成

### ディレクトリとクレート名の対応

| ディレクトリ | クレート名 | 説明 |
|------------|-----------|------|
| `app/core/` | `suika-game-core` | コアゲームロジック |
| `app/ui/` | `suika-game-ui` | UI実装 |
| `app/audio/` | `suika-game-audio` | オーディオ管理 |
| `app/assets/` | `suika-game-assets` | アセット管理 |
| `app/suika-game/` | `suika-game` | メインバイナリ |

## 依存関係グラフ

```
suika-game (main binary)
    │
    ├─→ suika-game-core
    ├─→ suika-game-ui ────→ suika-game-core
    ├─→ suika-game-audio ─→ suika-game-core
    └─→ suika-game-assets (独立)
```

## よく使うコマンド

### ビルド
```bash
# ワークスペース全体をビルド
cargo build

# リリースビルド
cargo build --release

# 特定のクレートのみビルド
cargo build -p suika-game-core
cargo build -p suika-game
```

### テスト
```bash
# 全テスト実行
cargo test

# 特定のクレートのテスト
cargo test -p suika-game-core
```

### 実行
```bash
# ゲームを実行
cargo run

# リリースモードで実行
cargo run --release
```

### クリーンアップ
```bash
# ビルドキャッシュをクリア
cargo clean
```

## クレート間のインポート

### suika-game (main) から他のクレートを使う
```rust
use suika_game_core::GameCorePlugin;
use suika_game_ui::GameUIPlugin;
use suika_game_audio::GameAudioPlugin;
use suika_game_assets::GameAssetsPlugin;
```

### suika-game-ui から core を使う
```rust
use suika_game_core::{
    components::Fruit,
    resources::GameState,
    states::AppState,
};
```

### suika-game-audio から core を使う
```rust
use suika_game_core::{
    states::AppState,
    resources::GameState,
};
```

## 新しい機能の追加手順

### 1. コア機能の追加
```bash
# app/core/src/ に新しいモジュールを追加
# app/core/src/lib.rs で公開
```

### 2. UI機能の追加
```bash
# app/ui/src/screens/ に新しい画面を追加
# app/ui/src/lib.rs のプラグインに登録
```

### 3. オーディオ機能の追加
```bash
# app/audio/src/ に新しいサウンドシステムを追加
```

### 4. アセットの追加
```bash
# assets/ にファイルを配置
# app/assets/src/ でローディングコードを追加
```

## トラブルシューティング

### ビルドエラー: クレートが見つからない
```bash
# Cargo.lockを削除して再ビルド
rm Cargo.lock
cargo build
```

### 依存関係のエラー
```bash
# クリーンビルド
cargo clean
cargo build
```

### クレート名の変更後のエラー
- `Cargo.toml` の `[workspace.dependencies]` を確認
- 各クレートの `Cargo.toml` の `name` を確認
- `main.rs` の `use` 文を確認

## ファイル構成チェックリスト

- [ ] `Cargo.toml` (workspace): members と dependencies が正しい
- [ ] `app/core/Cargo.toml`: name = "suika-game-core"
- [ ] `app/ui/Cargo.toml`: name = "suika-game-ui"
- [ ] `app/audio/Cargo.toml`: name = "suika-game-audio"
- [ ] `app/assets/Cargo.toml`: name = "suika-game-assets"
- [ ] `app/suika-game/Cargo.toml`: dependencies が正しい
- [ ] `app/suika-game/src/main.rs`: use 文が正しい

---

**作成日**: 2026-02-15
**バージョン**: 1.0
