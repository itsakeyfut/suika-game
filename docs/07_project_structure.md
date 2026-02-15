# スイカゲーム - プロジェクト構造書

## 1. プロジェクト概要

### 1.1 プロジェクト名
**suika-game** - Bevyで作るスイカゲームクローン

### 1.2 開発言語・フレームワーク
- **言語**: Rust (Edition 2021)
- **ゲームエンジン**: Bevy 0.18
- **物理エンジン**: Rapier2D (bevy_rapier2d 0.28)
- **オーディオ**: bevy_kira_audio 0.22

## 2. ディレクトリ構造

```
suika-game/
├── Cargo.toml                      # プロジェクト設定と依存関係
├── Cargo.lock                      # 依存関係のロックファイル
├── README.md                       # プロジェクト概要
├── .gitignore                      # Git除外設定
│
├── docs/                           # ドキュメント
│   ├── 01_specification.md         # ゲーム仕様書
│   ├── 02_architecture.md          # 技術アーキテクチャ設計書
│   ├── 03_physics_rendering.md     # 物理・レンダリング設計書
│   ├── 04_ui_ux.md                 # UI/UX設計書
│   ├── 05_audio.md                 # オーディオ設計書
│   ├── 06_implementation_plan.md   # 実装計画書
│   └── 07_project_structure.md     # プロジェクト構造書（本ファイル）
│
├── assets/                         # ゲームアセット
│   ├── sprites/                    # スプライト画像
│   │   ├── placeholder/            # プレースホルダー画像
│   │   └── fruits/                 # フルーツスプライト（後日追加）
│   ├── sounds/                     # サウンドファイル
│   │   ├── bgm/                    # BGM
│   │   │   ├── title_bgm.ogg
│   │   │   ├── game_bgm.ogg
│   │   │   └── gameover_bgm.ogg
│   │   └── sfx/                    # 効果音
│   │       ├── drop.wav
│   │       ├── merge_small.wav
│   │       ├── merge_medium.wav
│   │       ├── merge_large.wav
│   │       ├── watermelon.wav
│   │       ├── combo.wav
│   │       ├── gameover.wav
│   │       ├── button_click.wav
│   │       ├── button_hover.wav
│   │       └── warning.wav
│   └── fonts/                      # フォントファイル
│       └── PixelFont.ttf           # ピクセルフォント
│
├── save/                           # セーブデータ
│   └── highscore.json              # ハイスコア保存ファイル
│
├── src/                            # ソースコード
│   ├── main.rs                     # メインエントリポイント
│   ├── components.rs               # 全コンポーネント定義
│   ├── resources.rs                # グローバルリソース定義
│   ├── states.rs                   # ゲーム状態（State）定義
│   ├── constants.rs                # ゲーム定数とパラメータ
│   ├── fruit.rs                    # フルーツタイプと関連ロジック
│   ├── camera.rs                   # カメラセットアップ
│   │
│   ├── systems/                    # ゲームシステム
│   │   ├── mod.rs                  # システムモジュールルート
│   │   ├── spawn.rs                # フルーツスポーンシステム
│   │   ├── collision.rs            # 衝突検出と合体システム
│   │   ├── boundary.rs             # 境界チェックとゲームオーバー判定
│   │   ├── physics.rs              # 物理パラメータ調整
│   │   ├── score.rs                # スコアリングとコンボシステム
│   │   ├── effects.rs              # ビジュアルエフェクト
│   │   ├── camera_shake.rs         # 画面シェイクシステム
│   │   └── ui.rs                   # UI更新システム
│   │
│   ├── ui/                         # UI関連
│   │   ├── mod.rs                  # UIモジュールルート
│   │   ├── hud.rs                  # ゲーム中のHUD
│   │   ├── title.rs                # タイトル画面
│   │   └── game_over.rs            # ゲームオーバー画面
│   │
│   └── audio/                      # オーディオ関連
│       ├── mod.rs                  # オーディオモジュールルート
│       ├── bgm.rs                  # BGM管理
│       └── sfx.rs                  # 効果音管理
│
└── target/                         # ビルド成果物（gitignore）
```

## 3. ファイル詳細説明

### 3.1 ルートファイル

#### Cargo.toml
プロジェクトのメタデータと依存関係を定義。

```toml
[package]
name = "suika-game"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = "0.17.3"
bevy_rapier2d = "0.32.0"
bevy_kira_audio = "0.24.0"
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
```

#### .gitignore
Gitで追跡しないファイルを指定。

```
/target
Cargo.lock
/save/*.json
.DS_Store
```

#### README.md
プロジェクトの概要、ビルド方法、操作方法を記載。

### 3.2 src/ ディレクトリ

#### main.rs
**役割**: アプリケーションのエントリポイント

**内容**:
- Bevyアプリの初期化
- プラグインの追加（Rapier、KiraAudio）
- システムの登録（Startup, Update）
- 状態管理の設定
- リソースの初期化

**主要な責務**:
```rust
fn main() {
    App::new()
        // プラグイン
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin)
        .add_plugins(AudioPlugin)
        // 状態管理
        .init_state::<AppState>()
        // リソース
        .insert_resource(GameState::default())
        // システム
        .add_systems(Startup, (setup_camera, setup_container))
        .add_systems(Update, (...))
        .run();
}
```

#### components.rs
**役割**: 全てのカスタムECSコンポーネントを定義

**主要なコンポーネント**:
- `Fruit`: フルーツエンティティ
- `NextFruit`: 次のフルーツマーカー
- `Container`: コンテナ壁のマーカー
- `BoundaryLine`: 境界線マーカー
- `ParticleEffect`: パーティクルエフェクト
- `ScaleAnimation`: スケールアニメーション
- `CameraShake`: カメラシェイク

#### resources.rs
**役割**: グローバルリソースを定義

**主要なリソース**:
- `GameState`: ゲーム状態（スコア、ハイスコア、コンボ、時間）
- `NextFruitType`: 次のフルーツタイプ
- `AudioHandles`: オーディオアセットハンドル
- `FruitAssets`: フルーツスプライトハンドル
- `HighscoreData`: ハイスコア永続化用

#### states.rs
**役割**: ゲーム状態の遷移を管理

**状態定義**:
```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,     // タイトル画面
    Playing,   // ゲームプレイ中
    GameOver,  // ゲームオーバー画面
}
```

#### constants.rs
**役割**: ゲーム全体で使用する定数を一元管理

**定義内容**:
- 物理パラメータ（重力、容器サイズ）
- フルーツのパラメータ（半径、質量、得点）
- UI定数（色、フォントサイズ）
- エフェクトパラメータ

**例**:
```rust
// 物理
pub const GRAVITY: f32 = -980.0;
pub const CONTAINER_WIDTH: f32 = 600.0;

// フルーツパラメータ
pub struct FruitParams {
    pub radius: f32,
    pub mass: f32,
    pub restitution: f32,
    pub friction: f32,
    pub points: u32,
}

// カラーパレット
pub const BG_COLOR: Color = Color::srgb(0.95, 0.95, 0.90);
```

#### fruit.rs
**役割**: フルーツタイプの定義と関連ロジック

**内容**:
- `FruitType` enum（11種類）
- 各フルーツのパラメータ取得
- フルーツの進化ロジック
- 出現可能フルーツの定義
- プレースホルダー色の定義

**例**:
```rust
impl FruitType {
    pub fn next(&self) -> Option<FruitType> { ... }
    pub fn parameters(&self) -> FruitParams { ... }
    pub fn spawnable_fruits() -> [FruitType; 5] { ... }
    pub fn placeholder_color(&self) -> Color { ... }
}
```

#### camera.rs
**役割**: 2.5Dカメラのセットアップ

**内容**:
- OrthographicProjectionの設定
- カメラの初期位置と向き
- カメラの設定関数

### 3.3 src/systems/ ディレクトリ

#### mod.rs
システムモジュールのルート。全てのサブモジュールを公開。

```rust
pub mod spawn;
pub mod collision;
pub mod boundary;
pub mod physics;
pub mod score;
pub mod effects;
pub mod camera_shake;
pub mod ui;
```

#### spawn.rs
**役割**: フルーツのスポーンとプレイヤー入力処理

**主要なシステム**:
- `handle_fruit_spawn_input`: マウス/キーボード入力
- `spawn_fruit`: フルーツエンティティの生成
- `update_next_fruit`: 次のフルーツの選択

#### collision.rs
**役割**: 衝突検出と合体処理

**主要なシステム**:
- `detect_fruit_collision`: CollisionEventのリスニング
- `handle_fruit_merge`: 合体処理（エンティティ削除と新規生成）

**イベント**:
- `FruitMergeEvent`: 合体イベント

#### boundary.rs
**役割**: 境界チェックとゲームオーバー判定

**主要なシステム**:
- `check_game_over_boundary`: 境界線超過チェック
- `update_warning`: 警告表示の更新
- `trigger_game_over`: ゲームオーバー処理

#### physics.rs
**役割**: 物理パラメータの動的調整（必要に応じて）

#### score.rs
**役割**: スコアリングとコンボ管理

**主要なシステム**:
- `update_score_on_merge`: 合体時のスコア加算
- `update_combo`: コンボの管理
- `update_timer`: 経過時間の更新

#### effects.rs
**役割**: ビジュアルエフェクトの生成と更新

**主要なシステム**:
- `spawn_merge_particles`: 合体時のパーティクル生成
- `update_particles`: パーティクルの更新と削除
- `update_scale_animation`: スケールアニメーション

#### camera_shake.rs
**役割**: カメラシェイクエフェクト

**主要なシステム**:
- `update_camera_shake`: カメラ振動の更新
- `trigger_shake_on_merge`: 合体時のシェイクトリガー

#### ui.rs
**役割**: UI要素の更新

**主要なシステム**:
- `update_score_text`: スコア表示の更新
- `update_timer_text`: タイマー表示の更新
- `update_combo_text`: コンボ表示の更新
- `update_next_fruit_preview`: 次のフルーツプレビューの更新

### 3.4 src/ui/ ディレクトリ

#### mod.rs
UIモジュールのルート。

#### hud.rs
**役割**: ゲーム中のHUD（Heads-Up Display）

**内容**:
- スコア表示
- 次のフルーツプレビュー
- タイマー表示
- ハイスコア表示
- コンボ表示

**主要な関数**:
- `setup_hud`: HUDの初期セットアップ
- `cleanup_hud`: HUDのクリーンアップ

#### title.rs
**役割**: タイトル画面のUI

**内容**:
- タイトルテキスト
- スタートボタン
- 操作方法ボタン
- ハイスコア表示

**主要な関数**:
- `setup_title_screen`: タイトル画面のセットアップ
- `handle_title_buttons`: ボタンインタラクション処理

#### game_over.rs
**役割**: ゲームオーバー画面のUI

**内容**:
- "GAME OVER"テキスト
- 最終スコア表示
- 新記録通知（条件付き）
- リトライボタン
- タイトルに戻るボタン

**主要な関数**:
- `setup_game_over_screen`: ゲームオーバー画面のセットアップ
- `handle_game_over_buttons`: ボタンインタラクション処理

### 3.5 src/audio/ ディレクトリ

#### mod.rs
オーディオモジュールのルート。

#### bgm.rs
**役割**: BGMの管理と切り替え

**内容**:
- BGMアセットのロード
- 状態に応じたBGM切り替え
- フェードイン/フェードアウト

**主要なシステム**:
- `load_bgm_assets`: BGMのロード
- `switch_bgm_on_state_change`: 状態変更時のBGM切り替え

#### sfx.rs
**役割**: 効果音の管理と再生

**内容**:
- 効果音アセットのロード
- イベントに応じた効果音再生
- ピッチ・音量調整

**主要なシステム**:
- `load_sfx_assets`: 効果音のロード
- `play_merge_sfx`: 合体音の再生
- `play_combo_sfx`: コンボ音の再生
- `play_ui_sfx`: UI効果音の再生

## 4. モジュール間の依存関係

```
main.rs
  ├─> components.rs
  ├─> resources.rs
  ├─> states.rs
  ├─> constants.rs
  ├─> fruit.rs
  ├─> camera.rs
  ├─> systems/
  │     ├─> spawn.rs       (depends: fruit, components, resources)
  │     ├─> collision.rs   (depends: fruit, components)
  │     ├─> boundary.rs    (depends: components, resources, states)
  │     ├─> score.rs       (depends: resources)
  │     ├─> effects.rs     (depends: components)
  │     ├─> camera_shake.rs (depends: components)
  │     └─> ui.rs          (depends: resources)
  ├─> ui/
  │     ├─> hud.rs         (depends: resources, components)
  │     ├─> title.rs       (depends: states, resources)
  │     └─> game_over.rs   (depends: states, resources)
  └─> audio/
        ├─> bgm.rs         (depends: states)
        └─> sfx.rs         (depends: components, resources)
```

## 5. ビルドとテスト

### 5.1 ビルド方法

#### 開発ビルド
```bash
cargo build
```

#### リリースビルド
```bash
cargo build --release
```

### 5.2 実行方法

#### 開発モード
```bash
cargo run
```

#### リリースモード
```bash
cargo run --release
```

### 5.3 テスト

#### 全テスト実行
```bash
cargo test
```

#### 特定モジュールのテスト
```bash
cargo test --lib fruit
```

### 5.4 その他のコマンド

#### コードフォーマット
```bash
cargo fmt
```

#### Lintチェック
```bash
cargo clippy
```

#### ドキュメント生成
```bash
cargo doc --open
```

## 6. デバッグとプロファイリング

### 6.1 デバッグビルド
開発中は自動的にデバッグビルドが使用されます。

**特徴**:
- デバッグシンボル付き
- 最適化レベル1（Cargo.tomlで設定）
- Rapierのデバッグレンダラーが有効

### 6.2 リリースビルド
配布用のビルド。

**特徴**:
- 完全な最適化
- デバッグシンボルなし
- 小さいバイナリサイズ

### 6.3 プロファイリング

#### パフォーマンス計測
Bevyの診断機能を使用：

```rust
#[cfg(debug_assertions)]
app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
   .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin);
```

## 7. アセット管理

### 7.1 アセットパス
Bevyは `assets/` ディレクトリをルートとしてアセットをロードします。

**例**:
```rust
// assets/sprites/cherry.png を読み込む
asset_server.load("sprites/cherry.png")

// assets/sounds/bgm/title_bgm.ogg を読み込む
asset_server.load("sounds/bgm/title_bgm.ogg")
```

### 7.2 アセット命名規則
- **小文字とアンダースコア**: `merge_small.wav`
- **拡張子明記**: `.png`, `.ogg`, `.wav`
- **説明的な名前**: `button_click.wav` (良い), `sound1.wav` (悪い)

## 8. コーディング規約

### 8.1 命名規則
- **モジュール**: snake_case (`collision.rs`)
- **型**: PascalCase (`FruitType`, `GameState`)
- **関数**: snake_case (`spawn_fruit`, `update_score`)
- **定数**: SCREAMING_SNAKE_CASE (`GRAVITY`, `CONTAINER_WIDTH`)
- **変数**: snake_case (`fruit_type`, `next_fruit`)

### 8.2 コメント
- **ドキュメントコメント**: `///` を使用
- **実装コメント**: `//` を使用
- 複雑なロジックには必ずコメントを付ける

### 8.3 モジュール構成
- 1ファイル = 1責務
- 大きくなりすぎたファイルは分割
- `mod.rs` でモジュールを統合

## 9. バージョン管理

### 9.1 Git戦略
- **main**: 安定版（動作するバージョンのみ）
- **develop**: 開発中（デフォルトブランチ）
- **feature/***:  機能ブランチ（オプション）

### 9.2 コミットメッセージ
```
Phase X: <フェーズ名>

<詳細説明>
```

**例**:
```
Phase 4: Implement fruit spawning system

- Add spawn.rs with input handling
- Implement placeholder sprite generation
- Add physics components to fruits
```

## 10. 拡張性

### 10.1 新機能の追加
新しい機能は以下の手順で追加：

1. `src/systems/` または `src/ui/` に新しいファイルを作成
2. `mod.rs` に追加
3. `main.rs` でシステムを登録
4. テストと検証

### 10.2 新しいフルーツの追加
1. `fruit.rs` の `FruitType` enumに追加
2. `constants.rs` にパラメータを定義
3. スプライトを `assets/sprites/` に追加

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 承認済み
