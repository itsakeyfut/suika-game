# スイカゲーム - クレートアーキテクチャ設計書

## 1. ワークスペース構成概要

本プロジェクトは、Cargoワークスペースを使用して5つのクレートに分割されています。

### 1.1 クレート分割の目的
- **コンパイル時間の短縮**: 変更があったクレートのみ再コンパイル
- **責務の明確化**: 各クレートが明確な役割を持つ
- **保守性の向上**: モジュール境界が明確で、変更の影響範囲が限定的
- **再利用性**: 将来的に他のプロジェクトで一部クレートを再利用可能
- **テストの分離**: クレート単位でのテストが容易

## 2. クレート一覧

| クレート名 | 種類 | 説明 | 主要な依存先 |
|-----------|------|------|------------|
| `suika-game` | バイナリ | メインアプリケーション | 全ての内部クレート |
| `suika-game-core` | ライブラリ | コアゲームロジック | Bevy, Rapier2D |
| `suika-game-ui` | ライブラリ | UI実装 | suika-game-core, suika-game-assets |
| `suika-game-audio` | ライブラリ | オーディオ管理 | suika-game-core, suika-game-assets |
| `suika-game-assets` | ライブラリ | アセット管理 | Bevy |

## 3. ディレクトリ構造

```
suika-game/
├── Cargo.toml                      # ワークスペース定義
├── Cargo.lock                      # ワークスペース全体のロック
├── README.md
├── .gitignore
│
├── app/                            # 全クレートを格納
│   ├── suika-game-core/                  # コアロジッククレート
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── fruit.rs
│   │       ├── physics.rs
│   │       ├── components.rs
│   │       ├── resources.rs
│   │       ├── states.rs
│   │       ├── constants.rs
│   │       └── systems/
│   │           ├── mod.rs
│   │           ├── spawn.rs
│   │           ├── collision.rs
│   │           ├── boundary.rs
│   │           ├── score.rs
│   │           ├── effects.rs
│   │           └── camera_shake.rs
│   │
│   ├── suika-game-ui/                    # UIクレート
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── styles.rs           # カラー、フォントサイズ定義
│   │       ├── components.rs       # UIコンポーネント
│   │       └── screens/
│   │           ├── mod.rs
│   │           ├── title.rs
│   │           ├── hud.rs
│   │           └── game_over.rs
│   │
│   ├── suika-game-audio/                 # オーディオクレート
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── bgm.rs
│   │       └── sfx.rs
│   │
│   ├── suika-game-assets/                # アセット管理クレート
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── sprites.rs
│   │       ├── sounds.rs
│   │       └── fonts.rs
│   │
│   └── suika-game/                 # メインバイナリ
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
│
├── assets/                         # ゲームアセット
│   ├── sprites/
│   ├── sounds/
│   └── fonts/
│
├── docs/                           # ドキュメント
└── save/                           # セーブデータ
```

## 4. 各クレートの詳細

### 4.1 suika-game-core

**責務**: ゲームの中核となるロジックと物理演算

#### 公開API
```rust
// lib.rs - 公開する主要な型とモジュール
pub mod fruit;          // FruitType, フルーツパラメータ
pub mod components;     // Fruit, Container, 等
pub mod resources;      // GameState, NextFruitType
pub mod states;         // AppState
pub mod constants;      // ゲーム定数
pub mod systems;        // 全システム

// プラグイン形式でエクスポート
pub struct GameCorePlugin;
impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        // システムとリソースの登録
    }
}
```

#### 含まれる機能
1. **フルーツシステム**
   - `FruitType` enum（11種類）
   - フルーツパラメータ（半径、質量、得点）
   - フルーツの進化ロジック

2. **物理システム**
   - Rapier2D統合
   - 衝突検出システム
   - 合体ロジック
   - 境界チェック

3. **ゲーム状態管理**
   - `GameState` リソース（スコア、コンボ、時間）
   - `AppState` 列挙型（Title, Playing, GameOver）
   - ハイスコア保存/読み込み

4. **エフェクトシステム**
   - パーティクル生成・更新
   - カメラシェイク
   - スケールアニメーション

5. **コンポーネント・リソース定義**
   - 全ゲームコンポーネント
   - 全ゲームリソース
   - イベント定義

#### 依存関係
```toml
[dependencies]
bevy = "0.17.3"
bevy_rapier2d = "0.32.0"
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

### 4.2 suika-game-ui

**責務**: ユーザーインターフェースの実装

#### 公開API
```rust
// lib.rs
pub mod styles;         // カラーパレット、フォント定数
pub mod components;     // UIコンポーネント（ボタン等）
pub mod screens;        // 各画面の実装

pub struct GameUIPlugin;
impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        // UI関連システムの登録
    }
}
```

#### 含まれる機能
1. **画面実装**
   - タイトル画面（スタートボタン、ハイスコア表示）
   - ゲーム中HUD（スコア、次のフルーツ、タイマー、コンボ）
   - ゲームオーバー画面（最終スコア、リトライボタン）

2. **UIコンポーネント**
   - ボタンコンポーネント（ホバー、クリック状態）
   - テキスト表示コンポーネント
   - レイアウトヘルパー

3. **UIスタイル定義**
   - カラーパレット定数
   - フォントサイズ定数
   - レイアウト定数

#### 依存関係
```toml
[dependencies]
bevy = "0.17.3"
suika-game-core = { path = "../suika-game-core" }
suika-game-assets = { path = "../suika-game-assets" }
```

---

### 4.3 suika-game-audio

**責務**: BGMと効果音の管理

#### 公開API
```rust
// lib.rs
pub mod bgm;    // BGM管理
pub mod sfx;    // 効果音管理

pub struct GameAudioPlugin;
impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        // オーディオシステムの登録
    }
}
```

#### 含まれる機能
1. **BGM管理**
   - 状態に応じたBGM切り替え
   - フェードイン/フェードアウト
   - ループ再生

2. **効果音管理**
   - イベントベースの効果音再生
   - ピッチ・音量調整
   - 同時再生数制限

#### 依存関係
```toml
[dependencies]
bevy = "0.17.3"
bevy_kira_audio = "0.24.0"
suika-game-core = { path = "../suika-game-core" }
suika-game-assets = { path = "../suika-game-assets" }
```

---

### 4.4 suika-game-assets

**責務**: 全アセットの読み込みと管理

#### 公開API
```rust
// lib.rs
pub mod sprites;    // スプライトローディング
pub mod sounds;     // サウンドローディング
pub mod fonts;      // フォントローディング

// リソース
pub struct FruitSprites {
    pub sprites: HashMap<FruitType, Handle<Image>>,
}

pub struct AudioAssets {
    pub bgm_title: Handle<AudioSource>,
    pub bgm_game: Handle<AudioSource>,
    // ...
}

pub struct GameAssetsPlugin;
impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        // アセットローディングシステムの登録
    }
}
```

#### 含まれる機能
1. **スプライト管理**
   - フルーツスプライトの読み込み
   - UIスプライトの読み込み
   - プレースホルダー生成

2. **サウンド管理**
   - BGMファイルの読み込み
   - 効果音ファイルの読み込み

3. **フォント管理**
   - フォントファイルの読み込み

#### 依存関係
```toml
[dependencies]
bevy = "0.17.3"
suika-game-core = { workspace = true }  # FruitType等の共通型を使用
```

**注**: suika-game-assets は suika-game-core に依存してFruitType等の共通型を使用します。これにより、フルーツの種類に応じたアセット管理が可能になります。

---

### 4.5 suika-game (バイナリ)

**責務**: アプリケーションのエントリポイントと統合

#### 実装
```rust
// main.rs
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_kira_audio::AudioPlugin;

use game_core::GameCorePlugin;
use game_ui::GameUIPlugin;
use game_audio::GameAudioPlugin;
use game_assets::GameAssetsPlugin;

fn main() {
    App::new()
        // Bevyデフォルトプラグイン
        .add_plugins(DefaultPlugins)

        // 外部プラグイン
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(AudioPlugin)

        // ゲームプラグイン（内部クレート）
        .add_plugins(GameAssetsPlugin)  // 最初にアセットをロード
        .add_plugins(GameCorePlugin)
        .add_plugins(GameUIPlugin)
        .add_plugins(GameAudioPlugin)

        .run();
}
```

#### 依存関係
```toml
[dependencies]
bevy = "0.17.3"
bevy_rapier2d = "0.32.0"
bevy_kira_audio = "0.24.0"

# 内部クレート
suika-game-core = { path = "../suika-game-core" }
suika-game-ui = { path = "../suika-game-ui" }
suika-game-audio = { path = "../suika-game-audio" }
suika-game-assets = { path = "../suika-game-assets" }
```

## 5. クレート間の依存関係図

```
┌─────────────┐
│ suika-game  │  (バイナリ)
│   (main)    │
└──────┬──────┘
       │
       ├─────────────────────┬──────────────┬─────────────┐
       ↓                     ↓              ↓             ↓
┌────────────┐        ┌───────────┐  ┌────────────┐  ┌─────────────┐
│ suika-game-core  │←───────│  suika-game-ui  │  │suika-game-audio  │  │suika-game-assets  │
│            │        │           │  │            │  │             │
│  - fruit   │        │           │  │            │  └──────┬──────┘
│  - physics │        │           │  │            │         ↓
│  - states  │        └───────────┘  └────────────┘    (FruitType等)
│  - systems │               ↓              ↓
└──────┬─────┘               └──────────────┴────────────────┘
       ↓
  ┌─────────┐
  │  Bevy   │
  │ Rapier  │
  └─────────┘
```

### 依存関係の説明
- **suika-game**: 全ての内部クレートに依存
- **suika-game-ui**: suika-game-core と suika-game-assets に依存
- **suika-game-audio**: suika-game-core と suika-game-assets に依存
- **suika-game-assets**: suika-game-core に依存（FruitType等の共通型を使用）
- **suika-game-core**: 外部クレート（Bevy, Rapier）のみに依存

### 循環依存の回避
- 依存関係は一方向: core ← assets ← ui/audio ← main
- 共通型（FruitType等）は suika-game-core に配置
- suika-game-assets は core の型を使用してアセット管理を実装
- suika-game-ui と suika-game-audio は core と assets の両方を使用

## 6. プラグインパターン

各クレートはBevyプラグインとして実装され、main.rsで統合されます。

### 6.1 プラグインの利点
- **モジュール性**: 各機能が独立したプラグインとして実装
- **設定の一元化**: プラグイン内でシステムとリソースを登録
- **テスト容易性**: プラグイン単位でのテストが可能
- **有効化/無効化**: プラグインの追加/削除が容易

### 6.2 プラグイン実装例

```rust
// suika-game-core/src/lib.rs
pub struct GameCorePlugin;

impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        app
            // 状態管理
            .init_state::<AppState>()

            // リソース
            .init_resource::<GameState>()
            .init_resource::<NextFruitType>()

            // イベント
            .add_event::<FruitMergeEvent>()
            .add_event::<GameOverEvent>()

            // Startupシステム
            .add_systems(Startup, (
                setup_camera,
                setup_container,
            ))

            // Updateシステム
            .add_systems(Update, (
                spawn::handle_fruit_spawn_input,
                collision::detect_fruit_collision,
                collision::handle_fruit_merge,
                boundary::check_game_over_boundary,
                score::update_score_on_merge,
                effects::update_particles,
                camera_shake::update_camera_shake,
            ).run_if(in_state(AppState::Playing)));
    }
}
```

## 7. ビルドとテスト

### 7.1 ワークスペース全体のビルド

```bash
# ワークスペース全体をビルド
cargo build

# リリースビルド
cargo build --release

# 特定のクレートのみビルド
cargo build -p suika-game-core
cargo build -p suika-game
```

### 7.2 テスト

```bash
# ワークスペース全体のテスト
cargo test

# 特定のクレートのテスト
cargo test -p suika-game-core
cargo test -p suika-game-ui

# 特定のテストのみ実行
cargo test -p suika-game-core fruit::tests
```

### 7.3 実行

```bash
# メインバイナリの実行
cargo run

# リリースモードで実行
cargo run --release

# 特定のバイナリを指定（複数バイナリがある場合）
cargo run -p suika-game
```

## 8. 開発ワークフロー

### 8.1 機能追加時の流れ

1. **コア機能追加**: suika-game-core で実装
2. **UI追加**: suika-game-ui で画面を実装
3. **サウンド追加**: suika-game-audio で効果音を追加
4. **アセット追加**: suika-game-assets でローディング実装
5. **統合**: suika-game の main.rs で確認

### 8.2 変更の影響範囲

| 変更内容 | 再コンパイルが必要なクレート |
|---------|---------------------------|
| suika-game-core の変更 | core, ui, audio, main |
| suika-game-ui の変更 | ui, main |
| suika-game-audio の変更 | audio, main |
| suika-game-assets の変更 | assets, ui, audio, main |
| main.rs の変更 | main のみ |

### 8.3 段階的な開発

Phase 1-3: suika-game-core を中心に実装
- 物理環境
- フルーツシステム
- 基本ロジック

Phase 4-6: suika-game-ui を実装
- 各画面
- HUD

Phase 7: suika-game-audio を実装
- BGM
- 効果音

Phase 8-10: 統合と調整
- 全クレートの連携
- パフォーマンス最適化

## 9. コーディング規約（クレート固有）

### 9.1 公開API
- 各クレートの `lib.rs` で明示的に `pub` を使って公開
- 内部実装の詳細は非公開に保つ
- プラグイン形式でエクスポート

### 9.2 モジュール構成
```rust
// lib.rs の基本構造
pub mod module1;
pub mod module2;

// 再エクスポート
pub use module1::ImportantType;

// プラグイン
pub struct MyPlugin;
impl Plugin for MyPlugin { ... }
```

### 9.3 ドキュメント
- 各クレートの `lib.rs` にクレートレベルのドキュメント
- 公開APIには必ずドキュメントコメント
- 内部実装にも適切なコメント

## 10. パフォーマンス考慮

### 10.1 クレート分割による利点
- **並列コンパイル**: 独立したクレートは並列コンパイル可能
- **増分コンパイル**: 変更されたクレートのみ再コンパイル
- **リンク時最適化**: リリースビルドで LTO を有効化

### 10.2 Cargo.toml 設定

```toml
# ワークスペース Cargo.toml
[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3

[profile.release]
lto = "thin"  # リンク時最適化
codegen-units = 1
```

---

**バージョン**: 2.0（クレート分割版）
**最終更新**: 2026-02-15
**ステータス**: 承認済み
