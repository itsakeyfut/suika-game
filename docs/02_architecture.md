# スイカゲーム - 技術アーキテクチャ設計書

## 1. 技術スタック

### 1.1 コアテクノロジー
- **ゲームエンジン**: Bevy 0.18
  - ECS（Entity Component System）アーキテクチャ
  - 高性能な並列処理
  - Rustの安全性と速度を活用
  - 2D機能セット使用

### 1.2 主要依存クレート

```toml
[dependencies]
# ゲームエンジン
bevy = "0.17.3"

# 物理エンジン
bevy_rapier2d = "0.32.0"  # Bevy 0.18互換

# オーディオ
bevy_kira_audio = "0.24.0"

# ユーティリティ
rand = "0.8"  # ランダム生成

# データ永続化
serde = { version = "1.0", features = ["derive"] }
```

### 1.3 ビルドツール
- **ビルドシステム**: Cargo（Rust標準）
- **リリースビルド**: `cargo build --release`
- **実行**: `cargo run`

## 2. ECSアーキテクチャ設計

### 2.1 Entity（エンティティ）
Bevyでは、エンティティは単なるIDであり、コンポーネントの集合体です。

主要エンティティ：
- **Fruit**: フルーツオブジェクト
- **Container**: ゲームコンテナ（箱）の壁
- **Camera**: 2.5D斜め俯瞰カメラ
- **UI Elements**: スコア表示、ボタンなど

### 2.2 Component（コンポーネント）設計

#### 2.2.1 ゲームロジックコンポーネント

```rust
// フルーツタイプの定義
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FruitType {
    Cherry,      // サクランボ (段階1)
    Strawberry,  // イチゴ (段階2)
    Grape,       // ブドウ (段階3)
    Dekopon,     // デコポン (段階4)
    Persimmon,   // 柿 (段階5)
    Apple,       // リンゴ (段階6)
    Pear,        // 梨 (段階7)
    Peach,       // 桃 (段階8)
    Pineapple,   // パイナップル (段階9)
    Melon,       // メロン (段階10)
    Watermelon,  // スイカ (段階11)
}

// フルーツコンポーネント
#[derive(Component)]
pub struct Fruit {
    pub fruit_type: FruitType,
    pub points: u32,  // 合体時の得点
}

// 次のフルーツを示すマーカー
#[derive(Component)]
pub struct NextFruit;

// コンテナの壁を示すマーカー
#[derive(Component)]
pub struct Container;

// 境界チェック用マーカー
#[derive(Component)]
pub struct BoundaryLine;
```

#### 2.2.2 ビジュアルエフェクトコンポーネント

```rust
// パーティクルエフェクト
#[derive(Component)]
pub struct ParticleEffect {
    pub lifetime: f32,
    pub current_time: f32,
}

// スケールアニメーション
#[derive(Component)]
pub struct ScaleAnimation {
    pub start_scale: Vec3,
    pub end_scale: Vec3,
    pub duration: f32,
    pub elapsed: f32,
}

// カメラシェイク
#[derive(Component)]
pub struct CameraShake {
    pub intensity: f32,
    pub duration: f32,
    pub elapsed: f32,
}
```

#### 2.2.3 物理コンポーネント（Rapier提供）

Rapier2Dが提供するコンポーネント：
- `RigidBody`: 剛体タイプ（Dynamic, Fixed）
- `Collider`: 衝突形状（Ball, Cuboid）
- `Restitution`: 反発係数
- `Friction`: 摩擦係数
- `GravityScale`: 重力スケール
- `Velocity`: 速度
- `Mass`: 質量

### 2.3 Resource（リソース）設計

リソースはグローバルな状態を管理します。

```rust
// ゲーム状態
#[derive(Resource)]
pub struct GameState {
    pub score: u32,
    pub highscore: u32,
    pub combo: u32,
    pub elapsed_time: f32,
    pub is_game_over: bool,
}

// コンボタイマー
#[derive(Resource)]
pub struct ComboTimer {
    pub time_since_last_merge: f32,
    pub combo_window: f32,  // 2.0秒
}

// ゲームオーバー警告タイマー
#[derive(Resource)]
pub struct GameOverTimer {
    pub time_over_boundary: f32,
    pub warning_threshold: f32,  // 3.0秒
    pub is_warning: bool,
}

// 次のフルーツタイプ
#[derive(Resource)]
pub struct NextFruitType(pub FruitType);

// オーディオハンドル
#[derive(Resource)]
pub struct AudioHandles {
    pub bgm_title: Handle<AudioSource>,
    pub bgm_game: Handle<AudioSource>,
    pub bgm_gameover: Handle<AudioSource>,
    pub sfx_drop: Handle<AudioSource>,
    pub sfx_merge: Handle<AudioSource>,
    // ... その他の効果音
}

// フルーツアセットハンドル
#[derive(Resource)]
pub struct FruitAssets {
    pub sprites: HashMap<FruitType, Handle<Image>>,
    pub fallback_sprite: Handle<Image>,  // フォールバック用
}
```

### 2.4 System（システム）設計

システムはゲームロジックを実装する関数です。

#### 2.4.1 システムカテゴリ

1. **セットアップシステム** (Startup)
   - `setup_camera`: カメラのセットアップ
   - `setup_container`: ゲームコンテナの作成
   - `load_assets`: アセットの読み込み

2. **入力処理システム** (Update)
   - `handle_mouse_input`: マウス入力の処理
   - `handle_keyboard_input`: キーボード入力の処理

3. **ゲームロジックシステム** (Update)
   - `spawn_fruit`: フルーツのスポーン
   - `detect_collision`: 衝突検出
   - `merge_fruits`: フルーツの合体処理
   - `check_boundary`: 境界チェック
   - `update_score`: スコア更新
   - `update_timer`: タイマー更新

4. **ビジュアルエフェクトシステム** (Update)
   - `update_particles`: パーティクルの更新
   - `update_scale_animation`: スケールアニメーション
   - `update_camera_shake`: カメラシェイク

5. **UIシステム** (Update)
   - `update_score_ui`: スコア表示の更新
   - `update_next_fruit_ui`: 次のフルーツ表示の更新
   - `handle_button_interaction`: ボタンのインタラクション

6. **オーディオシステム** (Update)
   - `play_bgm`: BGMの再生・管理
   - `play_sfx`: 効果音の再生

#### 2.4.2 システム実行順序

Bevyのシステムセット（SystemSet）を使用して実行順序を制御：

```rust
// 疑似コード
app.add_systems(Startup, (
    setup_camera,
    setup_container,
    load_assets,
));

app.add_systems(Update, (
    handle_input,
    spawn_fruit,
    detect_collision.after(spawn_fruit),
    merge_fruits.after(detect_collision),
    check_boundary,
    update_score,
    update_effects,
    update_ui,
));
```

### 2.5 State（ゲーム状態）管理

Bevyの状態管理機能を使用して画面遷移を実装：

```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,     // タイトル画面
    Playing,   // ゲームプレイ中
    Paused,    // ポーズ中
    GameOver,  // ゲームオーバー画面
}
```

各状態に応じて異なるシステムを実行：
- `Title`: タイトル画面のUI、スタートボタン処理
- `Playing`: メインゲームロジック、物理演算
- `Paused`: ポーズメニュー表示、物理演算を停止
- `GameOver`: ゲームオーバー画面のUI、リトライ処理

**状態遷移**:
- `Title` → `Playing`: スタートボタンクリック
- `Playing` ⇄ `Paused`: ESCキー押下
- `Playing` → `GameOver`: ゲームオーバー条件成立
- `GameOver` → `Title`: タイトルに戻るボタン
- `GameOver` → `Playing`: リトライボタン

### 2.6 エラーハンドリングと安全パターン

#### 2.6.1 アセットローディングのエラーハンドリング

```rust
// アセット読み込みエラーの検出とフォールバック
fn check_asset_load_status(
    asset_server: Res<AssetServer>,
    assets: Res<FruitAssets>,
) {
    for (fruit_type, handle) in &assets.sprites {
        match asset_server.get_load_state(handle) {
            Some(LoadState::Failed(_)) => {
                error!("Failed to load sprite for {:?}, using fallback", fruit_type);
                // フォールバックスプライトを使用
            }
            _ => {}
        }
    }
}

// プレースホルダーの自動生成
fn generate_fallback_sprites(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) -> Handle<Image> {
    // 単色の円形画像を生成
    let image = Image::new_fill(
        Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255, 0, 255, 255], // マゼンタでエラーを視認しやすく
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    images.add(image)
}
```

#### 2.6.2 二重削除防止パターン

```rust
// フルーツ合体時の安全な削除
fn handle_fruit_merge(
    mut commands: Commands,
    mut merge_events: EventReader<FruitMergeEvent>,
    fruit_query: Query<Entity, With<Fruit>>,
) {
    for event in merge_events.read() {
        // 既に削除されていないか確認してから削除
        if fruit_query.get(event.entity1).is_ok() {
            commands.entity(event.entity1).despawn();
        }
        if fruit_query.get(event.entity2).is_ok() {
            commands.entity(event.entity2).despawn();
        }

        // 新しいフルーツをスポーン
        commands.spawn((
            Fruit {
                fruit_type: event.next_fruit_type,
                points: event.points,
            },
            Transform::from_translation(event.position.extend(0.0)),
            // ... その他のコンポーネント
        ));
    }
}

// または、削除済みフラグを使用する方法
#[derive(Component)]
pub struct PendingDespawn;

fn mark_for_despawn(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    fruit_query: Query<&Fruit, Without<PendingDespawn>>,
) {
    for event in collision_events.read() {
        // 両方のフルーツがまだ存在し、削除マークがないか確認
        if let (Ok(fruit1), Ok(fruit2)) = (
            fruit_query.get(entity1),
            fruit_query.get(entity2),
        ) {
            if fruit1.fruit_type == fruit2.fruit_type {
                commands.entity(entity1).insert(PendingDespawn);
                commands.entity(entity2).insert(PendingDespawn);
                // 合体イベントを発火
            }
        }
    }
}

fn despawn_marked(
    mut commands: Commands,
    marked_query: Query<Entity, With<PendingDespawn>>,
) {
    for entity in marked_query.iter() {
        commands.entity(entity).despawn();
    }
}
```

#### 2.6.3 タイマーのリセット処理

```rust
// コンボタイマーの更新と自動リセット
fn update_combo_timer(
    time: Res<Time>,
    mut combo_timer: ResMut<ComboTimer>,
    mut game_state: ResMut<GameState>,
) {
    combo_timer.time_since_last_merge += time.delta_secs();

    // 2.0秒経過でコンボリセット
    if combo_timer.time_since_last_merge > combo_timer.combo_window {
        if game_state.combo > 0 {
            info!("Combo reset: {} → 0", game_state.combo);
            game_state.combo = 0;
        }
    }
}

// ゲームオーバー警告タイマー
fn update_game_over_timer(
    time: Res<Time>,
    mut timer: ResMut<GameOverTimer>,
    mut next_state: ResMut<NextState<AppState>>,
    fruit_query: Query<&Transform, With<Fruit>>,
) {
    let mut any_over_boundary = false;

    for transform in fruit_query.iter() {
        if transform.translation.y > BOUNDARY_LINE_Y {
            any_over_boundary = true;
            break;
        }
    }

    if any_over_boundary {
        timer.time_over_boundary += time.delta_secs();
        timer.is_warning = true;

        if timer.time_over_boundary >= timer.warning_threshold {
            next_state.set(AppState::GameOver);
        }
    } else {
        // 境界線以下に戻ったらリセット
        timer.time_over_boundary = 0.0;
        timer.is_warning = false;
    }
}
```

## 3. モジュール構成

### 3.1 ディレクトリ構造

```
src/
├── main.rs              // メインエントリポイント
├── components.rs        // 全コンポーネント定義
├── resources.rs         // 全リソース定義
├── states.rs            // ゲーム状態定義
├── constants.rs         // 定数定義
├── fruit.rs             // フルーツタイプと関連ロジック
├── camera.rs            // カメラセットアップ
├── systems/
│   ├── mod.rs
│   ├── spawn.rs         // スポーンシステム
│   ├── collision.rs     // 衝突検出・合体システム
│   ├── boundary.rs      // 境界チェックシステム
│   ├── physics.rs       // 物理パラメータ調整
│   ├── score.rs         // スコアリングシステム
│   ├── effects.rs       // ビジュアルエフェクト
│   ├── camera_shake.rs  // カメラシェイク
│   └── ui.rs            // UI更新システム
├── ui/
│   ├── mod.rs
│   ├── hud.rs           // ゲーム中HUD
│   ├── title.rs         // タイトル画面
│   └── game_over.rs     // ゲームオーバー画面
└── audio/
    ├── mod.rs
    ├── bgm.rs           // BGM管理
    └── sfx.rs           // 効果音管理
```

### 3.2 モジュール責務

#### `main.rs`
- Bevyアプリの初期化
- プラグインの追加（Rapier, KiraAudio）
- システムの登録
- 状態管理の設定

#### `components.rs`
- 全てのカスタムコンポーネントの定義
- コンポーネント関連のヘルパー関数

#### `resources.rs`
- グローバルリソースの定義
- リソース初期化関数

#### `constants.rs`
- ゲーム定数（重力、箱のサイズ、スコアテーブル）
- 物理パラメータ（反発係数、摩擦係数）
- UI定数（色、フォントサイズ）

#### `fruit.rs`
- `FruitType` enum
- フルーツのパラメータ（サイズ、質量、得点）
- フルーツ関連のユーティリティ関数

## 4. データフロー

### 4.1 フルーツスポーンフロー

```
1. プレイヤー入力検出 (handle_input)
   ↓
2. ランダムフルーツタイプ選択 (spawn_fruit)
   ↓
3. エンティティ生成 (Fruit, Transform, RigidBody, Collider, Sprite)
   ↓
4. 次のフルーツタイプ更新 (NextFruitType リソース)
   ↓
5. UI更新 (update_next_fruit_ui)
```

### 4.2 フルーツ合体フロー

```
1. Rapier CollisionEvent発火
   ↓
2. 衝突検出システムがイベントをキャッチ (detect_collision)
   ↓
3. 両フルーツのタイプを比較
   ↓
4. 同じタイプなら合体処理開始 (merge_fruits)
   ↓
5. 2つのフルーツエンティティを削除
   ↓
6. 接触点に新しいフルーツをスポーン
   ↓
7. スコア加算 (update_score)
   ↓
8. エフェクト発生 (spawn_merge_effect)
   ↓
9. 効果音再生 (play_merge_sfx)
```

### 4.3 ゲームオーバーフロー

```
1. フルーツ位置チェック (check_boundary)
   ↓
2. 境界線超過検出
   ↓
3. 警告タイマー開始
   ↓
4. タイマー経過後にゲームオーバー判定
   ↓
5. GameState.is_game_over = true
   ↓
6. ハイスコア更新・保存
   ↓
7. 状態遷移 (Playing → GameOver)
   ↓
8. ゲームオーバー画面表示
```

## 5. イベント駆動アーキテクチャ

### 5.1 Bevyイベント

カスタムイベントを定義してシステム間の疎結合を実現：

```rust
// フルーツ合体イベント
#[derive(Event)]
pub struct FruitMergeEvent {
    pub fruit_type: FruitType,
    pub position: Vec2,
    pub points: u32,
}

// ゲームオーバーイベント
#[derive(Event)]
pub struct GameOverEvent {
    pub final_score: u32,
}

// コンボイベント
#[derive(Event)]
pub struct ComboEvent {
    pub combo_count: u32,
}
```

### 5.2 イベントの使用例

```rust
// イベント送信側（合体システム）
fn merge_fruits(
    mut merge_events: EventWriter<FruitMergeEvent>,
    // ...
) {
    // 合体処理
    merge_events.send(FruitMergeEvent {
        fruit_type: FruitType::Apple,
        position: Vec2::new(0.0, 0.0),
        points: 100,
    });
}

// イベント受信側（エフェクトシステム）
fn spawn_merge_effect(
    mut commands: Commands,
    mut merge_events: EventReader<FruitMergeEvent>,
) {
    for event in merge_events.read() {
        // パーティクルエフェクトを生成
    }
}
```

## 6. プラグイン構成

### 6.1 外部プラグイン

```rust
app.add_plugins(DefaultPlugins)
   .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
   .add_plugins(RapierDebugRenderPlugin::default())  // デバッグ用
   .add_plugins(AudioPlugin);
```

### 6.2 カスタムプラグイン（オプション）

コードの組織化のため、機能ごとにプラグインを作成：

```rust
pub struct FruitPlugin;
impl Plugin for FruitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_fruit, merge_fruits));
    }
}

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_score_ui, update_next_fruit_ui));
    }
}
```

## 7. パフォーマンス最適化戦略

### 7.1 ECSベストプラクティス
- クエリフィルタの活用（`With`, `Without`）
- システム並列化（依存関係のないシステムは自動並列実行）
- 不要なエンティティは即座に削除

### 7.2 物理演算最適化
- 静的オブジェクト（壁）は `RigidBody::Fixed` を使用
- スリープ機能の活用（静止したフルーツの計算を省略）
- コライダーの簡略化（円形コライダーのみ使用）

### 7.3 レンダリング最適化
- スプライトバッチング（同じテクスチャのスプライトをまとめて描画）
- Z座標による描画順序制御
- 画面外オブジェクトのカリング

## 8. テスト戦略

### 8.8 単体テスト
- フルーツタイプの進化ロジック
- スコア計算ロジック
- ハイスコアの保存・読み込み

### 8.2 統合テスト
- システム間の相互作用
- イベントフロー

### 8.3 手動テスト
- 物理挙動の自然さ
- UI/UXの快適性
- パフォーマンス（60fps維持）

## 9. エラーハンドリング

### 9.1 Asset読み込みエラー
- 存在しないアセットファイルへの対処
- プレースホルダーの使用

### 9.2 ファイルI/Oエラー
- ハイスコアファイルの読み書きエラー
- デフォルト値へのフォールバック

### 9.3 物理演算エラー
- 異常な速度検出と補正
- NaN/Inf値のチェック

## 10. 拡張性設計

### 10.1 新機能追加の容易さ
- プラグインシステムによる機能分離
- イベント駆動による疎結合
- 定数ファイルによる調整の容易さ

### 10.2 将来的な拡張
- 新しいフルーツタイプの追加
- 新しいゲームモードの追加
- マルチプレイヤー対応（将来）

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 承認済み
