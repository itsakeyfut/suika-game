# Phase 3: 物理環境の構築

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 3-4時間
**完了日**: -
**依存関係**: Phase 2

### 目的
Rapier2D物理エンジンを統合し、2.5Dカメラとゲームコンテナ（箱）を実装する。

### スコープ
- Rapier2Dプラグインのセットアップ
- 2.5D斜め俯瞰視点のカメラ構築
- ゲームコンテナ（左右の壁、底面）の作成
- 境界線の表示
- デバッグレンダラーの設定

## 前提条件

- Phase 2が完了している
- bevy_rapier2d依存関係がCargo.tomlに追加されている

## タスクリスト

### タスク 3.1: Rapier2Dプラグインのセットアップ

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-3, physics

**説明**:
bevy_rapier2dプラグインをBevyアプリに統合し、物理シミュレーションの基盤を構築する。

**受け入れ基準**:
- [ ] bevy_rapier2d = "0.32.0" がCargo.tomlに追加されている
- [ ] RapierPhysicsPluginがAppに追加されている
- [ ] RapierDebugRenderPluginがAppに追加されている（デバッグ用）
- [ ] 重力が設定されている（0, -980.0）
- [ ] pixels_per_meter が適切に設定されている
- [ ] `cargo run` で物理エンジンが動作する

**実装ガイド**:
```rust
// app/suika-game/Cargo.toml
[dependencies]
bevy = "0.17.3"
bevy_rapier2d = "0.32.0"
suika-game-core = { path = "../core" }

// app/suika-game/src/main.rs
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use suika_game_core::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, constants::GRAVITY),
            ..default()
        })
        .init_state::<AppState>()
        // ... リソース追加
        .run();
}
```

**関連ドキュメント**:
- [03_physics_rendering.md - セクション2](../03_physics_rendering.md)
- [Rapier2D公式ドキュメント](https://rapier.rs/docs/user_guides/bevy_plugin/getting_started_bevy/)

---

### タスク 3.2: 2.5Dカメラのセットアップ

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-3, rendering

**説明**:
斜め俯瞰視点の2.5Dカメラを実装し、奥行き感のある表現を実現する。

**受け入れ基準**:
- [ ] `app/suika-game/src/camera.rs` が作成されている
- [ ] `setup_camera` システムが実装されている
- [ ] OrthographicProjectionが設定されている
- [ ] カメラの位置と角度が適切に設定されている
- [ ] Z軸の表示順序が正しく機能する
- [ ] ゲーム画面が正しく表示される

**実装ガイド**:
```rust
// app/suika-game/src/camera.rs
use bevy::prelude::*;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 1000.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

// main.rsに追加
mod camera;
use camera::setup_camera;

// Startupシステムに追加
.add_systems(Startup, setup_camera)
```

**関連ドキュメント**:
- [03_physics_rendering.md - セクション4](../03_physics_rendering.md)

---

### タスク 3.3: ゲームコンテナ（箱）の実装

**優先度**: P0
**推定工数**: 1.5時間
**ラベル**: task, phase-3, physics

**説明**:
ゲームコンテナ（左右の壁と底面）を物理ボディとして実装する。

**受け入れ基準**:
- [ ] `app/suika-game/src/container.rs` が作成されている
- [ ] `setup_container` システムが実装されている
- [ ] 左壁、右壁、底面の3つの壁が作成されている
- [ ] 各壁にRigidBody::Fixedが設定されている
- [ ] 各壁にColliderが設定されている
- [ ] 摩擦係数と反発係数が適切に設定されている
- [ ] 壁が視覚的に表示されている
- [ ] Containerマーカーコンポーネントが付与されている

**実装ガイド**:
```rust
// app/suika-game/src/container.rs
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use suika_game_core::*;

pub fn setup_container(mut commands: Commands) {
    // 左壁
    spawn_wall(
        &mut commands,
        Vec2::new(-constants::CONTAINER_WIDTH / 2.0, 0.0),
        Vec2::new(constants::WALL_THICKNESS, constants::CONTAINER_HEIGHT),
    );

    // 右壁
    spawn_wall(
        &mut commands,
        Vec2::new(constants::CONTAINER_WIDTH / 2.0, 0.0),
        Vec2::new(constants::WALL_THICKNESS, constants::CONTAINER_HEIGHT),
    );

    // 底面
    spawn_wall(
        &mut commands,
        Vec2::new(0.0, -constants::CONTAINER_HEIGHT / 2.0),
        Vec2::new(constants::CONTAINER_WIDTH, constants::WALL_THICKNESS),
    );
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

**関連ドキュメント**:
- [03_physics_rendering.md - セクション2](../03_physics_rendering.md)

---

### タスク 3.4: 境界線（ゲームオーバーライン）の表示

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-3, rendering

**説明**:
ゲームオーバー判定に使用する境界線を視覚的に表示する。

**受け入れ基準**:
- [ ] `setup_container` に境界線スポーン処理が追加されている
- [ ] 境界線が赤い半透明の線として表示されている
- [ ] 境界線のY座標がconstants::BOUNDARY_LINE_Yに一致している
- [ ] BoundaryLineマーカーコンポーネントが付与されている
- [ ] 境界線に物理コライダーは設定されていない（視覚のみ）

**実装ガイド**:
```rust
// container.rsのsetup_container関数に追加
pub fn setup_container(mut commands: Commands) {
    // ... 壁のスポーン

    // 境界線（視覚のみ、コライダーなし）
    commands.spawn((
        BoundaryLine,
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(1.0, 0.0, 0.0, 0.5),
                custom_size: Some(Vec2::new(constants::CONTAINER_WIDTH, 2.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, constants::BOUNDARY_LINE_Y, 1.0),
            ..default()
        },
    ));
}
```

**関連ドキュメント**:
- [01_specification.md - セクション1.3](../01_specification.md)

---

### タスク 3.5: デバッグレンダラーの設定

**優先度**: P2
**推定工数**: 0.5時間
**ラベル**: task, phase-3, debug

**説明**:
物理コライダーを可視化するデバッグレンダラーを設定し、開発時の確認を容易にする。

**受け入れ基準**:
- [ ] RapierDebugRenderPluginが設定されている
- [ ] デバッグレンダラーのON/OFFを切り替えられる
- [ ] コライダーの形状が緑色の線で表示される
- [ ] Dキーでデバッグ表示を切り替えられる

**実装ガイド**:
```rust
// app/suika-game/src/debug.rs
use bevy::prelude::*;
use bevy_rapier2d::render::RapierDebugRenderPlugin;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(debug_assertions)]
        {
            app.add_plugins(RapierDebugRenderPlugin::default());
            app.add_systems(Update, toggle_debug_render);
        }
    }
}

#[cfg(debug_assertions)]
fn toggle_debug_render(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_render: ResMut<bevy_rapier2d::render::DebugRenderContext>,
) {
    if keyboard.just_pressed(KeyCode::KeyD) {
        debug_render.enabled = !debug_render.enabled;
    }
}

// main.rsに追加
.add_plugins(DebugPlugin)
```

**関連ドキュメント**:
- [10_advanced_topics.md - セクション7](../10_advanced_topics.md)

---

### タスク 3.6: システムの統合とテスト

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-3, integration

**説明**:
Phase 3で実装したすべてのシステムをmain.rsに統合し、動作確認を行う。

**受け入れ基準**:
- [ ] main.rsに全システムが追加されている
- [ ] `cargo run` でゲーム画面が表示される
- [ ] カメラが正しく配置されている
- [ ] ゲームコンテナ（箱）が表示されている
- [ ] 境界線が表示されている
- [ ] デバッグレンダラーでコライダーが確認できる
- [ ] 警告なしでビルドできる

**実装ガイド**:
```rust
// app/suika-game/src/main.rs
mod camera;
mod container;
mod debug;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use camera::setup_camera;
use container::setup_container;
use debug::DebugPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(DebugPlugin)
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, suika_game_core::constants::GRAVITY),
            ..default()
        })
        .init_state::<suika_game_core::AppState>()
        // ... リソース
        .add_systems(Startup, (setup_camera, setup_container))
        .run();
}
```

**関連ドキュメント**:
- [07_project_structure.md](../07_project_structure.md)

---

## フェーズ検証

### 検証項目

- [ ] すべてのタスクが完了している
- [ ] `cargo build --workspace` が成功する
- [ ] `cargo run` でゲーム画面が表示される
- [ ] ゲームコンテナ（箱）が正しく表示されている
- [ ] 境界線が正しい位置に表示されている
- [ ] デバッグレンダラーでコライダーが確認できる
- [ ] 重力が正しく設定されている（次フェーズでフルーツを落として確認）
- [ ] 警告なしでビルドできる

### 検証手順

```bash
# ビルドチェック
cargo build --workspace

# ゲーム実行
cargo run

# 確認項目:
# 1. ウィンドウが開く
# 2. 箱（左右の壁、底面）が表示される
# 3. 境界線（赤い線）が表示される
# 4. Dキーでデバッグレンダラーをトグル
# 5. コライダーが緑色の線で表示される
```

## 完了条件

- [ ] すべてのタスクが完了している
- [ ] すべての検証項目が合格している
- [ ] ドキュメントが更新されている（必要な場合）
- [ ] コードがフォーマットされている（`just fmt`）
- [ ] Clippyチェックが通っている（`just clippy`）

## 次のフェーズ

Phase 3完了 → 次は **Phase 4: フルーツシステムの実装** に進む

## 備考

- Phase 3完了時点で、ゲーム画面に箱が表示される
- まだフルーツは表示されないが、物理環境は整っている
- デバッグレンダラーは開発時のみ有効（debug_assertions）
- 2.5D表現は後のフェーズで深さ（Z軸）を使って強化される
- pixels_per_meter = 100.0 は調整可能（Phase 10で微調整）

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 未着手
