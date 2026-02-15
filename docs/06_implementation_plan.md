# スイカゲーム - 実装計画書

## 1. 実装フェーズ概要

本プロジェクトは11のフェーズに分けて段階的に実装します。各フェーズは独立してテスト可能で、前のフェーズの成果物を基に構築されます。

### 1.1 フェーズ一覧

| フェーズ | 名称 | 推定工数 | 依存関係 |
|---------|------|---------|---------|
| Phase 1 | プロジェクトセットアップ | 1-2時間 | なし |
| Phase 2 | ゲーム状態管理とリソース | 2-3時間 | Phase 1 |
| Phase 3 | 物理環境の構築 | 3-4時間 | Phase 2 |
| Phase 4 | フルーツシステムの実装 | 4-6時間 | Phase 3 |
| Phase 5 | 衝突検出と合体システム | 4-6時間 | Phase 4 |
| Phase 6 | ゲームオーバー判定 | 2-3時間 | Phase 5 |
| Phase 7 | 基本UIの実装 | 4-6時間 | Phase 6 |
| Phase 8 | リッチなビジュアルエフェクト | 4-6時間 | Phase 7 |
| Phase 9 | サウンド統合 | 3-4時間 | Phase 8 |
| Phase 10 | 調整とポリッシュ | 6-8時間 | Phase 9 |
| Phase 11 | ピクセルアート統合（後日） | 8-12時間 | Phase 10 |

**総推定工数**: 40-60時間（ピクセルアート作成を除く）

## 2. Phase 1: プロジェクトセットアップ

### 2.1 目標
プロジェクトの依存関係とディレクトリ構造を整備し、ビルドが通る状態にする。

### 2.2 作業内容

#### 2.2.1 Cargo.toml の更新

```toml
[package]
name = "suika-game"
version = "0.1.0"
edition = "2024"

[dependencies]
# ゲームエンジン
bevy = "0.17.3"

# 物理エンジン
bevy_rapier2d = "0.32.0"

# オーディオ
bevy_kira_audio = "0.24.0"

# ユーティリティ
rand = "0.8"

# データ永続化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.dev]
opt-level = 1  # 開発中のパフォーマンス向上

[profile.dev.package."*"]
opt-level = 3  # 依存クレートは最適化
```

#### 2.2.2 ディレクトリ構造の作成

```bash
mkdir -p assets/sprites/placeholder
mkdir -p assets/sounds/bgm
mkdir -p assets/sounds/sfx
mkdir -p assets/fonts
mkdir -p save
```

#### 2.2.3 .gitignore の更新

```.gitignore
/target
Cargo.lock
/save/*.json
```

#### 2.2.4 基本的な main.rs の作成

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .run();
}
```

### 2.3 検証
- `cargo build` が成功する
- `cargo run` でウィンドウが開く
- ディレクトリ構造が正しく作成されている

---

## 3. Phase 2: ゲーム状態管理とリソース

### 3.1 目標
コアのデータ構造とゲーム状態を定義し、ハイスコアの保存/読み込み機能を実装する。

### 3.2 作業内容

#### 3.2.1 ファイル作成
- `src/constants.rs`
- `src/fruit.rs`
- `src/components.rs`
- `src/resources.rs`
- `src/states.rs`

#### 3.2.2 constants.rs の実装

```rust
use bevy::prelude::*;

// 物理パラメータ
pub const GRAVITY: f32 = -980.0;
pub const CONTAINER_WIDTH: f32 = 600.0;
pub const CONTAINER_HEIGHT: f32 = 800.0;
pub const WALL_THICKNESS: f32 = 20.0;
pub const BOUNDARY_LINE_Y: f32 = 300.0;  // ゲームオーバーライン

// フルーツパラメータ
pub struct FruitParams {
    pub radius: f32,
    pub mass: f32,
    pub restitution: f32,
    pub friction: f32,
    pub points: u32,
}

// ... フルーツごとのパラメータ定義
```

#### 3.2.3 fruit.rs の実装

```rust
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FruitType {
    Cherry,
    Strawberry,
    Grape,
    Dekopon,
    Persimmon,
    Apple,
    Pear,
    Peach,
    Pineapple,
    Melon,
    Watermelon,
}

impl FruitType {
    pub fn next(&self) -> Option<FruitType> {
        // 次の段階のフルーツを返す
    }

    pub fn parameters(&self) -> FruitParams {
        // constants.rsからパラメータを取得
    }

    pub fn spawnable_fruits() -> [FruitType; 5] {
        // 出現可能な5種類
    }
}
```

#### 3.2.4 components.rs の実装

```rust
use bevy::prelude::*;
use crate::fruit::FruitType;

#[derive(Component)]
pub struct Fruit {
    pub fruit_type: FruitType,
    pub points: u32,
}

#[derive(Component)]
pub struct NextFruit;

#[derive(Component)]
pub struct Container;

// ... その他のコンポーネント
```

#### 3.2.5 resources.rs の実装

```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Default)]
pub struct GameState {
    pub score: u32,
    pub highscore: u32,
    pub combo: u32,
    pub elapsed_time: f32,
    pub is_game_over: bool,
}

#[derive(Resource, Serialize, Deserialize)]
pub struct HighscoreData {
    pub highscore: u32,
}

// ハイスコアの保存/読み込み
impl HighscoreData {
    pub fn load() -> Self {
        // save/highscore.json から読み込み
    }

    pub fn save(&self) {
        // save/highscore.json に保存
    }
}
```

#### 3.2.6 states.rs の実装

```rust
use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,
    Playing,
    GameOver,
}
```

#### 3.2.7 main.rs の更新

```rust
mod components;
mod constants;
mod fruit;
mod resources;
mod states;

use bevy::prelude::*;
use states::AppState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .insert_resource(resources::GameState::default())
        .run();
}
```

### 3.3 検証
- 全てのファイルがコンパイルエラーなくビルドできる
- ハイスコアの保存/読み込みが動作する（単体テスト）

---

## 4. Phase 3: 物理環境の構築

### 4.1 目標
Rapier2Dを統合し、2.5Dカメラとゲームコンテナを作成する。

### 4.2 作業内容

#### 4.2.1 ファイル作成
- `src/camera.rs`

#### 4.2.2 main.rs へのプラグイン追加

```rust
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())  // デバッグ用
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, GRAVITY),
            ..default()
        })
        // ...
        .add_systems(Startup, (setup_camera, setup_container))
        .run();
}
```

#### 4.2.3 camera.rs の実装

```rust
use bevy::prelude::*;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 1000.0),
        ..default()
    });
}
```

#### 4.2.4 コンテナのセットアップ

```rust
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub fn setup_container(mut commands: Commands) {
    // 左壁
    spawn_wall(&mut commands,
        Vec2::new(-CONTAINER_WIDTH / 2.0, 0.0),
        Vec2::new(WALL_THICKNESS, CONTAINER_HEIGHT)
    );

    // 右壁
    spawn_wall(&mut commands,
        Vec2::new(CONTAINER_WIDTH / 2.0, 0.0),
        Vec2::new(WALL_THICKNESS, CONTAINER_HEIGHT)
    );

    // 底面
    spawn_wall(&mut commands,
        Vec2::new(0.0, -CONTAINER_HEIGHT / 2.0),
        Vec2::new(CONTAINER_WIDTH, WALL_THICKNESS)
    );

    // 境界線（視覚のみ）
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(1.0, 0.0, 0.0, 0.5),
                custom_size: Some(Vec2::new(CONTAINER_WIDTH, 2.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, BOUNDARY_LINE_Y, 1.0),
            ..default()
        },
        BoundaryLine,
    ));
}

fn spawn_wall(commands: &mut Commands, position: Vec2, size: Vec2) {
    commands.spawn((
        Container,
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.6, 0.4, 0.2),
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_xyz(position.x, position.y, -50.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(size.x / 2.0, size.y / 2.0),
        Friction::coefficient(0.5),
        Restitution::coefficient(0.3),
    ));
}
```

### 4.3 検証
- ウィンドウにカメラビューが表示される
- ゲームコンテナ（箱）が表示される
- デバッグレンダラーでコライダーが可視化される
- 重力が設定されている

---

## 5. Phase 4: フルーツシステムの実装

### 5.1 目標
フルーツのスポーン、物理挙動、プレースホルダー表示を実装する。

### 5.2 作業内容

#### 5.2.1 ファイル作成
- `src/systems/mod.rs`
- `src/systems/spawn.rs`
- `src/systems/physics.rs`

#### 5.2.2 spawn.rs の実装

```rust
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

pub fn handle_fruit_spawn_input(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    next_fruit_type: Res<NextFruitType>,
) {
    // マウスクリックまたはスペースキーでフルーツをスポーン
    if mouse_button.just_pressed(MouseButton::Left) || keyboard.just_pressed(KeyCode::Space) {
        // カーソル位置を取得してワールド座標に変換
        // フルーツをスポーン
        spawn_fruit(&mut commands, next_fruit_type.0, cursor_world_pos);

        // 次のフルーツをランダムに決定
    }
}

pub fn spawn_fruit(
    commands: &mut Commands,
    fruit_type: FruitType,
    position: Vec2,
) {
    let params = fruit_type.parameters();

    commands.spawn((
        Fruit {
            fruit_type,
            points: params.points,
        },
        SpriteBundle {
            sprite: Sprite {
                color: fruit_type.placeholder_color(),
                custom_size: Some(Vec2::splat(params.radius * 2.0)),
                ..default()
            },
            transform: Transform::from_xyz(position.x, position.y, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::ball(params.radius),
        Restitution::coefficient(params.restitution),
        Friction::coefficient(params.friction),
        ColliderMassProperties::Mass(params.mass),
        Damping {
            linear_damping: 0.5,
            angular_damping: 1.0,
        },
    ));
}
```

#### 5.2.3 プレースホルダー色の定義

```rust
impl FruitType {
    pub fn placeholder_color(&self) -> Color {
        match self {
            FruitType::Cherry => Color::srgb(1.0, 0.0, 0.0),
            FruitType::Strawberry => Color::srgb(1.0, 0.3, 0.3),
            FruitType::Grape => Color::srgb(0.5, 0.0, 0.8),
            FruitType::Dekopon => Color::srgb(1.0, 0.6, 0.0),
            FruitType::Persimmon => Color::srgb(1.0, 0.5, 0.0),
            FruitType::Apple => Color::srgb(0.8, 0.0, 0.0),
            FruitType::Pear => Color::srgb(0.8, 0.9, 0.3),
            FruitType::Peach => Color::srgb(1.0, 0.7, 0.7),
            FruitType::Pineapple => Color::srgb(0.9, 0.8, 0.0),
            FruitType::Melon => Color::srgb(0.5, 0.9, 0.3),
            FruitType::Watermelon => Color::srgb(0.0, 0.7, 0.2),
        }
    }
}
```

#### 5.2.4 システムの登録

```rust
.add_systems(Update, (
    handle_fruit_spawn_input,
).run_if(in_state(AppState::Playing)))
```

### 5.3 検証
- スペースキーまたはマウスクリックでフルーツがスポーンする
- フルーツが重力で落下する
- フルーツが壁や他のフルーツと衝突する
- フルーツが自然に揺れて静止する（jiggle effect）
- 次のフルーツがランダムに選ばれる
- **ホットリロードテスト**: プレースホルダー色を変更して即座に反映されるか確認（Bevyの標準ホットリロード機能が動作することを確認）

---

## 6. Phase 5: 衝突検出と合体システム

### 6.1 目標
同じフルーツの衝突を検出し、合体処理を実装する。

### 6.2 作業内容

#### 6.2.1 ファイル作成
- `src/systems/collision.rs`
- `src/systems/score.rs`

#### 6.2.2 イベントの定義

```rust
#[derive(Event)]
pub struct FruitMergeEvent {
    pub entity1: Entity,
    pub entity2: Entity,
    pub fruit_type: FruitType,
    pub position: Vec2,
}
```

#### 6.2.3 collision.rs の実装

```rust
pub fn detect_fruit_collision(
    mut collision_events: EventReader<CollisionEvent>,
    fruit_query: Query<(&Fruit, &Transform)>,
    mut merge_events: EventWriter<FruitMergeEvent>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            if let (Ok((fruit1, transform1)), Ok((fruit2, transform2))) = (
                fruit_query.get(*e1),
                fruit_query.get(*e2),
            ) {
                if fruit1.fruit_type == fruit2.fruit_type {
                    // 合体イベントを発火
                    let position = (transform1.translation.truncate()
                        + transform2.translation.truncate()) / 2.0;

                    merge_events.send(FruitMergeEvent {
                        entity1: *e1,
                        entity2: *e2,
                        fruit_type: fruit1.fruit_type,
                        position,
                    });
                }
            }
        }
    }
}

pub fn handle_fruit_merge(
    mut commands: Commands,
    mut merge_events: EventReader<FruitMergeEvent>,
) {
    for event in merge_events.read() {
        // 2つのフルーツを削除
        commands.entity(event.entity1).despawn();
        commands.entity(event.entity2).despawn();

        // 次の段階のフルーツをスポーン（スイカ以外）
        if let Some(next_fruit) = event.fruit_type.next() {
            spawn_fruit(&mut commands, next_fruit, event.position);
        }
    }
}
```

#### 6.2.4 スコアシステムの実装

```rust
pub fn update_score_on_merge(
    mut merge_events: EventReader<FruitMergeEvent>,
    mut game_state: ResMut<GameState>,
) {
    for event in merge_events.read() {
        let points = event.fruit_type.parameters().points;
        game_state.score += points;

        // コンボ処理（オプション）
    }
}
```

### 6.3 検証
- 同じフルーツが接触すると合体する
- 合体後、正しい次の段階のフルーツが生成される
- スイカ同士が接触すると両方消滅する
- スコアが正しく加算される

---

## 7. Phase 6: ゲームオーバー判定

### 7.1 目標
境界線を超えたフルーツを検出し、ゲームオーバー処理を実装する。

### 7.2 作業内容

#### 7.2.1 ファイル作成
- `src/systems/boundary.rs`

#### 7.2.2 boundary.rs の実装

```rust
pub fn check_game_over_boundary(
    fruit_query: Query<&Transform, With<Fruit>>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
) {
    let mut fruits_over_line = 0;

    for transform in fruit_query.iter() {
        if transform.translation.y > BOUNDARY_LINE_Y {
            fruits_over_line += 1;
        }
    }

    if fruits_over_line > 0 {
        // 警告状態（一定時間経過でゲームオーバー）
        // タイマー処理
    } else {
        // タイマーリセット
    }

    // ゲームオーバー判定
    if /* 条件 */ {
        game_state.is_game_over = true;

        // ハイスコア更新
        if game_state.score > game_state.highscore {
            game_state.highscore = game_state.score;
            // 保存
        }

        next_state.set(AppState::GameOver);
    }
}
```

### 7.3 検証
- フルーツが境界線を超えると警告が表示される
- 一定時間超えた状態が続くとゲームオーバーになる
- ハイスコアが正しく更新・保存される

---

## 8. Phase 7: 基本UIの実装

### 8.1 目標
タイトル画面、ゲーム中のHUD、ゲームオーバー画面を実装する。

### 8.2 作業内容

#### 8.2.1 ファイル作成
- `src/ui/mod.rs`
- `src/ui/title.rs`
- `src/ui/hud.rs`
- `src/ui/game_over.rs`
- `src/systems/ui.rs`

#### 8.2.2 各画面のUI実装
（詳細はUI/UX設計書を参照）

### 8.3 検証
- タイトル画面が表示され、スタートボタンが機能する
- ゲーム中にスコア、次のフルーツ、タイマーが表示される
- ゲームオーバー画面が表示され、リトライボタンが機能する

---

## 9. Phase 8: リッチなビジュアルエフェクト

### 9.1 目標
パーティクル、画面シェイク、スケールアニメーションを実装する。

### 9.2 作業内容

#### 9.2.1 ファイル作成
- `src/systems/effects.rs`
- `src/systems/camera_shake.rs`

#### 9.2.2 エフェクトの実装
（詳細は物理・レンダリング設計書を参照）

### 9.3 検証
- 合体時にパーティクルエフェクトが発生する
- 大きなフルーツの合体時に画面が揺れる
- 新しいフルーツがポップアニメーションで出現する

---

## 10. Phase 9: サウンド統合

### 10.1 目標
BGMと効果音を統合し、適切なタイミングで再生する。

### 10.2 作業内容

#### 10.2.1 ファイル作成
- `src/audio/mod.rs`
- `src/audio/bgm.rs`
- `src/audio/sfx.rs`

#### 10.2.2 オーディオシステムの実装
（詳細はオーディオ設計書を参照）

### 10.3 検証
- 各画面で適切なBGMが再生される
- 効果音が適切なタイミングで再生される
- 音量バランスが適切

---

## 11. Phase 10: 調整とポリッシュ

### 11.1 目標
ゲームプレイを快適にするための微調整とバグ修正。

### 11.2 作業内容

#### 11.2.1 物理パラメータの調整
- 重力、反発係数、摩擦係数の微調整
- フルーツサイズのバランス調整
- jiggle effectの最適化

#### 11.2.2 UI/UXの改善
- フォントとレイアウトの調整
- カラーパレットの微調整
- アニメーションのタイミング調整

#### 11.2.3 パフォーマンス最適化
- プロファイリング
- ボトルネックの特定と改善
- メモリリークのチェック

#### 11.2.4 プレイテスト
- 実際にプレイしてバグを発見
- ゲームバランスの調整
- ユーザビリティの改善

### 11.3 検証
- 安定して60fps以上を維持
- オリジナルのスイカゲームと同様の遊び心地
- 目立ったバグがない

---

## 12. Phase 11: ピクセルアート統合（後日）

### 12.1 目標
プレースホルダーを自作のピクセルアートスプライトに置き換える。

### 12.2 作業内容

#### 12.2.1 アートアセットの作成
- 各フルーツのピクセルアートスプライト（200x200px）
- 背景画像（オプション）
- UIエレメント（ボタン、アイコン）

#### 12.2.2 スプライトの統合
- スプライトシートの作成
- アセットローダーの更新
- プレースホルダーからの置き換え

#### 12.2.3 視覚調整
- スプライトのサイズ調整
- アニメーションフレームの追加（オプション）

### 12.3 検証
- 全てのフルーツが正しいスプライトで表示される
- 視覚的な統一感がある
- パフォーマンスに影響がない
- **ホットリロード活用**: スプライトファイルを編集して保存すると、ゲーム再起動なしで即座に反映されることを確認（開発効率の向上）

---

## 13. 開発ワークフロー

### 13.1 日次ワークフロー

1. **朝**: 今日の目標フェーズを確認
2. **実装**: 集中して実装
3. **テスト**: 実装した機能をテスト
4. **コミット**: 動作する状態でコミット
5. **レビュー**: 1日の成果を確認

### 13.2 各フェーズのワークフロー

1. **計画**: フェーズの詳細を再確認
2. **実装**: コードを書く
3. **テスト**: 検証項目をチェック
4. **デバッグ**: 問題を修正
5. **コミット**: gitコミット
6. **次へ**: 次のフェーズに進む

### 13.3 Git コミット戦略

- **コミット粒度**: 各フェーズごとにコミット
- **コミットメッセージ**: `Phase X: <フェーズ名>`
- **ブランチ戦略**:
  - `main`: 安定版
  - `develop`: 開発中
  - `feature/phaseX`: 各フェーズ用ブランチ（オプション）

---

## 14. リスク管理

### 14.1 想定されるリスク

| リスク | 影響度 | 発生確率 | 対策 |
|--------|--------|---------|------|
| Bevy/Rapier のバグ | 高 | 低 | 公式ドキュメントとコミュニティを参照 |
| 物理挙動が不自然 | 中 | 中 | パラメータを段階的に調整 |
| パフォーマンス問題 | 中 | 低 | プロファイリングと最適化 |
| スコープクリープ | 高 | 中 | 計画に忠実に、拡張は後回し |

### 14.2 対応方針
- 問題が発生したら、まず公式ドキュメントを確認
- 解決しない場合は、Discordやフォーラムで質問
- 最悪の場合、機能を簡略化して実装

---

## 15. 成功基準

### 15.1 最小成功基準（MVP）
- [ ] フルーツをスポーンして落とせる
- [ ] 同じフルーツが合体する
- [ ] スイカを作成できる
- [ ] ゲームオーバー判定が機能する
- [ ] スコアが表示される

### 15.2 目標成功基準
- [ ] 全11種類のフルーツが実装されている
- [ ] リッチなビジュアルエフェクトがある
- [ ] BGMと効果音が統合されている
- [ ] 60fps以上で動作する
- [ ] オリジナルと遜色ない遊び心地

### 15.3 理想成功基準
- [ ] 自作のピクセルアートが統合されている
- [ ] ハイスコアランキング機能
- [ ] タイムアタックモード
- [ ] 複数のゲームモード

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 承認済み
