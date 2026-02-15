# Phase 4: フルーツシステムの実装

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 4-6時間
**完了日**: -
**依存関係**: Phase 3

### 目的
フルーツのスポーン、物理挙動、プレースホルダー表示を実装し、ゲームの基本的なインタラクションを実現する。

### スコープ
- フルーツスポーンシステムの実装
- プレイヤー入力処理（マウス、キーボード）
- フルーツの物理挙動（重力、衝突、jiggle effect）
- 次のフルーツプレビュー機能
- プレースホルダースプライトの表示

## 前提条件

- Phase 3が完了している
- 物理環境（Rapier2D、コンテナ）が整っている

## タスクリスト

### タスク 4.1: フルーツスポーンシステムの基本実装

**優先度**: P0
**推定工数**: 2時間
**ラベル**: task, phase-4, core

**説明**:
フルーツをワールドにスポーンする基本的なシステムを実装する。

**受け入れ基準**:
- [ ] `app/core/src/systems/mod.rs` が作成されている
- [ ] `app/core/src/systems/spawn.rs` が作成されている
- [ ] `spawn_fruit` 関数が実装されている
- [ ] フルーツにFruitコンポーネントが付与されている
- [ ] フルーツにRigidBody::Dynamicが設定されている
- [ ] フルーツにCollider::ballが設定されている
- [ ] フルーツに適切な物理パラメータが設定されている（質量、反発係数、摩擦係数）
- [ ] プレースホルダースプライト（単色の円）が表示される

**実装ガイド**:
```rust
// app/core/src/systems/mod.rs
pub mod spawn;

// app/core/src/systems/spawn.rs
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::*;

pub fn spawn_fruit(
    commands: &mut Commands,
    fruit_type: FruitType,
    position: Vec2,
) -> Entity {
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
        GravityScale(1.0),
    )).id()
}
```

**関連ドキュメント**:
- [03_physics_rendering.md - セクション2.2](../03_physics_rendering.md)

---

### タスク 4.2: プレイヤー入力処理システムの実装

**優先度**: P0
**推定工数**: 1.5時間
**ラベル**: task, phase-4, input

**説明**:
マウスまたはキーボード入力でフルーツをスポーンできるようにする。

**受け入れ基準**:
- [ ] `app/core/src/systems/input.rs` が作成されている
- [ ] `handle_fruit_spawn_input` システムが実装されている
- [ ] スペースキーでフルーツがスポーンできる
- [ ] マウス左クリックでフルーツがスポーンできる
- [ ] マウス位置（X座標）に応じてフルーツの落下位置が変わる
- [ ] 左右矢印キーでフルーツの落下位置を調整できる
- [ ] スポーン位置がコンテナの範囲内に制限されている
- [ ] NextFruitTypeリソースが使用されている
- [ ] スポーン後、次のフルーツがランダムに選ばれる

**実装ガイド**:
```rust
// app/core/src/systems/input.rs
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::seq::SliceRandom;
use crate::*;

#[derive(Resource)]
pub struct SpawnPosition {
    pub x: f32,
}

impl Default for SpawnPosition {
    fn default() -> Self {
        Self { x: 0.0 }
    }
}

pub fn handle_fruit_spawn_input(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_fruit: ResMut<NextFruitType>,
    spawn_pos: Res<SpawnPosition>,
) {
    if mouse_button.just_pressed(MouseButton::Left)
        || keyboard.just_pressed(KeyCode::Space) {

        // 現在の NextFruitType でフルーツをスポーン
        let spawn_y = constants::CONTAINER_HEIGHT / 2.0 - 50.0;
        spawn::spawn_fruit(
            &mut commands,
            next_fruit.0,
            Vec2::new(spawn_pos.x, spawn_y),
        );

        // 次のフルーツをランダムに選択
        let mut rng = rand::thread_rng();
        let spawnable = FruitType::spawnable_fruits();
        next_fruit.0 = *spawnable.choose(&mut rng).unwrap();
    }
}

pub fn update_spawn_position(
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut spawn_pos: ResMut<SpawnPosition>,
    time: Res<Time>,
) {
    // 矢印キーで移動
    const MOVE_SPEED: f32 = 300.0;
    if keyboard.pressed(KeyCode::ArrowLeft) {
        spawn_pos.x -= MOVE_SPEED * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        spawn_pos.x += MOVE_SPEED * time.delta_seconds();
    }

    // マウス位置を取得
    if let Ok(window) = windows.get_single() {
        if let Some(cursor_pos) = window.cursor_position() {
            let (camera, camera_transform) = camera_query.single();
            if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                spawn_pos.x = world_pos.x;
            }
        }
    }

    // コンテナの範囲内に制限
    let max_x = constants::CONTAINER_WIDTH / 2.0 - 40.0;
    spawn_pos.x = spawn_pos.x.clamp(-max_x, max_x);
}
```

**関連ドキュメント**:
- [04_ui_ux.md - セクション3](../04_ui_ux.md)

---

### タスク 4.3: 次のフルーツプレビュー表示

**優先度**: P1
**推定工数**: 1時間
**ラベル**: task, phase-4, ui

**説明**:
画面上部に次に落とすフルーツのプレビューを表示する。

**受け入れ基準**:
- [ ] `app/core/src/systems/preview.rs` が作成されている
- [ ] `setup_fruit_preview` システムが実装されている
- [ ] `update_fruit_preview` システムが実装されている
- [ ] 次のフルーツがプレビューとして小さく表示される
- [ ] NextFruitTypeが変更されるとプレビューも更新される
- [ ] プレビューはスポーン位置の上に表示される
- [ ] NextFruitPreviewマーカーコンポーネントが使用されている

**実装ガイド**:
```rust
// app/core/src/systems/preview.rs
use bevy::prelude::*;
use crate::*;

pub fn setup_fruit_preview(
    mut commands: Commands,
    next_fruit: Res<NextFruitType>,
) {
    let params = next_fruit.0.parameters();

    commands.spawn((
        NextFruitPreview,
        SpriteBundle {
            sprite: Sprite {
                color: next_fruit.0.placeholder_color(),
                custom_size: Some(Vec2::splat(params.radius * 1.5)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, constants::CONTAINER_HEIGHT / 2.0 - 100.0, 10.0),
            ..default()
        },
    ));
}

pub fn update_fruit_preview(
    mut preview_query: Query<(&mut Sprite, &mut Transform), With<NextFruitPreview>>,
    next_fruit: Res<NextFruitType>,
    spawn_pos: Res<input::SpawnPosition>,
) {
    if next_fruit.is_changed() {
        for (mut sprite, mut transform) in preview_query.iter_mut() {
            let params = next_fruit.0.parameters();
            sprite.color = next_fruit.0.placeholder_color();
            sprite.custom_size = Some(Vec2::splat(params.radius * 1.5));
        }
    }

    // プレビュー位置をスポーン位置に合わせる
    for (_, mut transform) in preview_query.iter_mut() {
        transform.translation.x = spawn_pos.x;
    }
}
```

**関連ドキュメント**:
- [04_ui_ux.md - セクション2](../04_ui_ux.md)

---

### タスク 4.4: 物理パラメータの調整

**優先度**: P1
**推定工数**: 1時間
**ラベル**: task, phase-4, physics

**説明**:
フルーツの物理挙動（jiggle effect等）を自然にするためのパラメータ調整を行う。

**受け入れ基準**:
- [ ] フルーツが自然に落下する
- [ ] フルーツ同士が衝突すると自然に転がる
- [ ] jiggle effect（揺れて静止）が実現されている
- [ ] フルーツが箱の外に飛び出さない
- [ ] 各フルーツのサイズが適切（半径の差が適切）
- [ ] 質量、反発係数、摩擦係数が適切に調整されている

**実装ガイド**:
```rust
// app/core/src/constants.rs または fruit.rs
impl FruitType {
    pub fn parameters(&self) -> FruitParams {
        match self {
            FruitType::Cherry => FruitParams {
                radius: 20.0,
                mass: 1.0,
                restitution: 0.4,  // 反発係数
                friction: 0.5,     // 摩擦係数
                points: 10,
            },
            FruitType::Strawberry => FruitParams {
                radius: 30.0,
                mass: 2.0,
                restitution: 0.35,
                friction: 0.5,
                points: 20,
            },
            // ... 他のフルーツ
            FruitType::Watermelon => FruitParams {
                radius: 120.0,
                mass: 50.0,
                restitution: 0.2,
                friction: 0.6,
                points: 10240,
            },
        }
    }
}

// Dampingパラメータの調整
Damping {
    linear_damping: 0.5,   // 直線運動の減衰
    angular_damping: 1.0,  // 回転運動の減衰（jiggle effectに影響）
}
```

**関連ドキュメント**:
- [03_physics_rendering.md - セクション2.3](../03_physics_rendering.md)

---

### タスク 4.5: システムの統合とテスト

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-4, integration

**説明**:
Phase 4で実装したすべてのシステムをメインアプリに統合する。

**受け入れ基準**:
- [ ] すべてのシステムがmain.rsに追加されている
- [ ] システムの実行順序が適切に設定されている
- [ ] SpawnPositionリソースが初期化されている
- [ ] システムがAppState::Playingでのみ実行される
- [ ] `cargo run` でフルーツをスポーンできる

**実装ガイド**:
```rust
// app/suika-game/src/main.rs (または app/core/src/lib.rs でプラグイン化)
use suika_game_core::systems::*;

fn main() {
    App::new()
        // ... プラグイン
        .insert_resource(input::SpawnPosition::default())
        .add_systems(Startup, (
            setup_camera,
            setup_container,
            preview::setup_fruit_preview,
        ))
        .add_systems(Update, (
            input::update_spawn_position,
            input::handle_fruit_spawn_input,
            preview::update_fruit_preview,
        ).run_if(in_state(AppState::Playing)))
        .run();
}
```

**関連ドキュメント**:
- [02_architecture.md - セクション2.4](../02_architecture.md)

---

### タスク 4.6: ホットリロードテスト

**優先度**: P2
**推定工数**: 0.5時間
**ラベル**: task, phase-4, testing

**説明**:
Bevyのホットリロード機能が正しく動作することを確認する。

**受け入れ基準**:
- [ ] ゲーム実行中にプレースホルダー色を変更できる
- [ ] 変更が即座に反映される（ゲーム再起動不要）
- [ ] ホットリロードの動作確認手順がドキュメント化されている

**実装ガイド**:
```bash
# 1. ゲームを実行
cargo run

# 2. ゲーム実行中に app/core/src/fruit.rs を編集
# 例: Cherry の色を赤から青に変更
# Color::srgb(1.0, 0.0, 0.0) → Color::srgb(0.0, 0.0, 1.0)

# 3. ファイルを保存

# 4. ゲーム内で自動的にリロードされることを確認
# （フルーツの色が即座に変わる）
```

**関連ドキュメント**:
- [10_advanced_topics.md - セクション1](../10_advanced_topics.md)

---

## フェーズ検証

### 検証項目

- [ ] すべてのタスクが完了している
- [ ] `cargo build --workspace` が成功する
- [ ] `cargo run` でゲームが起動する
- [ ] スペースキーでフルーツがスポーンする
- [ ] マウスクリックでフルーツがスポーンする
- [ ] 矢印キーで落下位置を調整できる
- [ ] マウス移動で落下位置が変わる
- [ ] 次のフルーツプレビューが表示される
- [ ] フルーツが重力で落下する
- [ ] フルーツ同士が衝突する
- [ ] フルーツが自然に揺れて静止する（jiggle effect）
- [ ] フルーツが箱の外に飛び出さない
- [ ] ホットリロードが機能する

### 検証手順

```bash
# ゲーム実行
cargo run

# 確認項目:
# 1. 画面上部に次のフルーツプレビューが表示される
# 2. マウスを動かすとプレビューも移動する
# 3. スペースキーまたはマウスクリックでフルーツがスポーンする
# 4. フルーツが落下し、箱の底や壁に衝突する
# 5. フルーツが少し揺れてから静止する
# 6. 複数のフルーツを落として積み上げられる
# 7. 次のフルーツがランダムに選ばれる（柿以下の5種類）
```

## 完了条件

- [ ] すべてのタスクが完了している
- [ ] すべての検証項目が合格している
- [ ] ドキュメントが更新されている（必要な場合）
- [ ] コードがフォーマットされている（`just fmt`）
- [ ] Clippyチェックが通っている（`just clippy`）

## 次のフェーズ

Phase 4完了 → 次は **Phase 5: 衝突検出と合体システム** に進む

## 備考

- Phase 4完了時点で、基本的なゲームプレイ（フルーツを落とす）が可能になる
- まだ合体機能はないが、フルーツを積み上げることはできる
- プレースホルダースプライト（単色の円）を使用しているが、Phase 11でピクセルアートに置き換える
- jiggle effectの調整はプレイフィールに大きく影響するため、丁寧に調整する
- ホットリロード機能を活用して、色やサイズを素早く調整できる

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 未着手
