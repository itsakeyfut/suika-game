# スイカゲーム - 物理・レンダリング設計書

## 1. 2.5D視覚表現

### 1.1 2.5Dとは
2.5Dは2D物理演算を使用しながら、視覚的に3D的な奥行き感を表現する手法です。本プロジェクトでは斜め俯瞰視点（Oblique Overhead View）を採用し、ゲームに深みと立体感を与えます。

### 1.2 カメラ設計

#### 1.2.1 正投影カメラ
Bevyの `OrthographicProjection` を使用します。

```rust
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1.0,  // ズームレベル
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 1000.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
```

#### 1.2.2 カメラの配置
- **位置**: (0, 0, 1000) - Z軸方向に離れた位置
- **向き**: 下方向（-Z方向）を向く
- **上ベクトル**: Y軸正方向
- **スケール**: ゲーム全体が見えるように調整（1.0から開始）

### 1.3 奥行き表現の実装

#### 1.3.1 Z座標による描画順序
Y座標（高さ）に基づいてZ座標を設定し、手前のオブジェクトが上に描画されるようにします。

```rust
// 疑似コード
fn update_sprite_depth(
    mut query: Query<(&Transform, &mut Transform), With<Fruit>>,
) {
    for transform in query.iter_mut() {
        // Y座標が低いほど手前（Z値が大きい）
        transform.translation.z = -transform.translation.y * 0.01;
    }
}
```

#### 1.3.2 レイヤリング
- **背景**: Z = -100
- **ゲームコンテナ（箱）**: Z = -50
- **フルーツ**: Z = -Y * 0.01 （動的に変化）
- **UI**: Z = 100（常に最前面）

### 1.4 視覚効果

#### 1.4.1 影の追加（オプション）
フルーツの下に薄い影を配置し、立体感を強調：
- 影のスプライトをフルーツの下に配置
- Y座標に応じて影のサイズを変更（高いほど大きく薄く）

#### 1.4.2 パララックス効果（オプション）
背景を複数レイヤーに分けて異なる速度でスクロールさせることで奥行き感を強調（カメラ移動がある場合）。

## 2. 物理エンジン統合（Rapier2D）

### 2.1 Rapier2Dの概要
Rapier2Dは高性能な2D物理エンジンで、Bevyと公式に統合されています。

### 2.2 プラグイン設定

```rust
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())  // デバッグ表示
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -980.0),  // 重力加速度
            ..default()
        })
        .run();
}
```

### 2.3 重力設定

#### 2.3.1 重力パラメータ
- **基本重力**: `Vec2::new(0.0, -980.0)` ピクセル/秒²
- **調整可能**: ゲームプレイの快適性に応じて -500.0 〜 -1500.0 の範囲で調整
- **理由**: リアルすぎる重力（-9.8 m/s²）はゲームプレイには遅すぎる

#### 2.3.2 個別の重力スケール
特定のフルーツに異なる重力を適用する場合：

```rust
commands.spawn((
    // ... 他のコンポーネント
    GravityScale(1.5),  // 通常の1.5倍の重力
));
```

### 2.4 フルーツの物理設定

#### 2.4.1 フルーツのパラメータ表

| フルーツ | 半径(px) | 質量(kg) | 反発係数 | 摩擦係数 |
|----------|---------|---------|---------|---------|
| サクランボ | 20 | 0.5 | 0.3 | 0.3 |
| イチゴ | 25 | 0.7 | 0.3 | 0.3 |
| ブドウ | 30 | 1.0 | 0.3 | 0.3 |
| デコポン | 35 | 1.5 | 0.3 | 0.3 |
| 柿 | 40 | 2.0 | 0.3 | 0.3 |
| リンゴ | 50 | 3.0 | 0.3 | 0.3 |
| 梨 | 55 | 4.0 | 0.3 | 0.3 |
| 桃 | 60 | 5.0 | 0.3 | 0.3 |
| パイナップル | 70 | 7.0 | 0.3 | 0.3 |
| メロン | 80 | 10.0 | 0.3 | 0.3 |
| スイカ | 100 | 15.0 | 0.3 | 0.3 |

*注: 上記の値は初期値であり、プレイテスト後に調整する可能性があります。*

#### 2.4.2 フルーツコンポーネント構成

```rust
fn spawn_fruit(
    commands: &mut Commands,
    fruit_type: FruitType,
    position: Vec2,
    assets: &FruitAssets,
) {
    let params = fruit_type.parameters();  // constants.rsから取得

    commands.spawn((
        // ゲームロジックコンポーネント
        Fruit {
            fruit_type,
            points: params.points,
        },

        // 描画
        SpriteBundle {
            texture: assets.sprites[&fruit_type].clone(),
            transform: Transform::from_xyz(position.x, position.y, 0.0)
                .with_scale(Vec3::splat(params.radius / 50.0)),  // 正規化
            ..default()
        },

        // 物理 - 剛体
        RigidBody::Dynamic,

        // 物理 - コライダー（円形）
        Collider::ball(params.radius),

        // 物理 - 反発係数
        Restitution {
            coefficient: params.restitution,
            combine_rule: CoefficientCombineRule::Average,
        },

        // 物理 - 摩擦係数
        Friction {
            coefficient: params.friction,
            combine_rule: CoefficientCombineRule::Average,
        },

        // 物理 - 質量
        ColliderMassProperties::Mass(params.mass),

        // 減衰（空気抵抗のような効果）
        Damping {
            linear_damping: 0.5,   // 線形減衰
            angular_damping: 1.0,  // 角減衰
        },
    ));
}
```

### 2.5 ゲームコンテナ（箱）の設定

#### 2.5.1 コンテナのサイズ
- **幅**: 600 ピクセル
- **高さ**: 800 ピクセル
- **壁の厚さ**: 20 ピクセル

#### 2.5.2 壁の物理設定

```rust
fn setup_container(mut commands: Commands) {
    let wall_thickness = 20.0;
    let container_width = 600.0;
    let container_height = 800.0;

    // 左壁
    commands.spawn((
        Container,
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.6, 0.4, 0.2),  // 茶色
                custom_size: Some(Vec2::new(wall_thickness, container_height)),
                ..default()
            },
            transform: Transform::from_xyz(
                -container_width / 2.0,
                0.0,
                -50.0,
            ),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(wall_thickness / 2.0, container_height / 2.0),
        Friction::coefficient(0.5),
        Restitution::coefficient(0.3),
    ));

    // 右壁（同様）
    // 底面（同様）
}
```

### 2.6 Jiggle Effect（揺れエフェクト）の実装

#### 2.6.1 原理
フルーツが着地した後に自然に揺れるエフェクトは、以下のパラメータの組み合わせで実現：
1. **反発係数（Restitution）**: 0.3 - 適度に跳ね返る
2. **減衰（Damping）**: 線形0.5、角1.0 - 徐々に静止
3. **摩擦（Friction）**: 0.3 - 滑りすぎない

#### 2.6.2 調整のポイント
- 反発係数を上げすぎると永遠に跳ね続ける
- 減衰を強くしすぎると不自然に急停止
- 摩擦が強すぎると転がらない

### 2.7 衝突検出

#### 2.7.1 衝突イベントのリスニング

```rust
fn detect_collision(
    mut collision_events: EventReader<CollisionEvent>,
    fruit_query: Query<&Fruit>,
    mut merge_events: EventWriter<FruitMergeEvent>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _flags) = event {
            // 両方のエンティティがフルーツか確認
            if let (Ok(fruit1), Ok(fruit2)) = (
                fruit_query.get(*e1),
                fruit_query.get(*e2)
            ) {
                // 同じタイプか確認
                if fruit1.fruit_type == fruit2.fruit_type {
                    // 合体イベントを発火
                    merge_events.send(FruitMergeEvent {
                        entity1: *e1,
                        entity2: *e2,
                        fruit_type: fruit1.fruit_type,
                    });
                }
            }
        }
    }
}
```

#### 2.7.2 接触点の取得（オプション）

より正確な合体位置を得るために：

```rust
fn get_contact_point(
    collision_event: &CollisionEvent,
    context: &RapierContext,
) -> Option<Vec2> {
    // Rapier の contact pair から接触点を取得
    // 詳細は bevy_rapier2d のドキュメントを参照
}
```

## 3. レンダリング設計

### 3.1 スプライトシステム

#### 3.1.1 スプライトの構成
- **フォーマット**: PNG（透過あり）
- **サイズ**: 各フルーツ 200x200 ピクセル（高解像度）
- **スタイル**: ピクセルアート
- **背景**: 透明

#### 3.1.2 スプライトのロード

```rust
#[derive(Resource)]
pub struct FruitAssets {
    pub sprites: HashMap<FruitType, Handle<Image>>,
}

fn load_fruit_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut sprites = HashMap::new();

    sprites.insert(
        FruitType::Cherry,
        asset_server.load("sprites/cherry.png"),
    );
    // ... 他のフルーツも同様

    commands.insert_resource(FruitAssets { sprites });
}
```

#### 3.1.3 プレースホルダースプライト
アート作成前は、単色の円でプレースホルダーを使用：

```rust
fn create_placeholder_sprite(
    commands: &mut Commands,
    fruit_type: FruitType,
) {
    let color = match fruit_type {
        FruitType::Cherry => Color::srgb(1.0, 0.0, 0.0),      // 赤
        FruitType::Strawberry => Color::srgb(1.0, 0.3, 0.3),  // ピンク
        FruitType::Grape => Color::srgb(0.5, 0.0, 0.8),       // 紫
        // ...
    };

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color,
            custom_size: Some(Vec2::splat(radius * 2.0)),
            ..default()
        },
        // ...
    });
}
```

### 3.2 パーティクルシステム

#### 3.2.1 合体時のパーティクル
フルーツが合体した時に発生するエフェクト：

```rust
fn spawn_merge_particles(
    commands: &mut Commands,
    position: Vec2,
    fruit_type: FruitType,
) {
    for i in 0..20 {  // 20個のパーティクルを生成
        let angle = (i as f32 / 20.0) * std::f32::consts::TAU;
        let velocity = Vec2::from_angle(angle) * 200.0;

        commands.spawn((
            ParticleEffect {
                lifetime: 0.5,
                current_time: 0.0,
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgba(1.0, 1.0, 0.0, 1.0),  // 黄色
                    custom_size: Some(Vec2::splat(5.0)),
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 10.0),
                ..default()
            },
            RigidBody::Dynamic,
            Velocity::linear(velocity),
            GravityScale(0.5),  // 軽い重力
        ));
    }
}
```

#### 3.2.2 パーティクルの更新と削除

```rust
fn update_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ParticleEffect, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut particle, mut sprite) in query.iter_mut() {
        particle.current_time += time.delta_seconds();

        // 透明度を徐々に減少
        let alpha = 1.0 - (particle.current_time / particle.lifetime);
        sprite.color.set_alpha(alpha);

        // 寿命が尽きたら削除
        if particle.current_time >= particle.lifetime {
            commands.entity(entity).despawn();
        }
    }
}
```

### 3.3 スケールアニメーション

#### 3.3.1 合体時のポップエフェクト
新しいフルーツが出現する際に、大きくなってから元のサイズに戻る：

```rust
#[derive(Component)]
pub struct ScaleAnimation {
    pub start_scale: Vec3,
    pub end_scale: Vec3,
    pub duration: f32,
    pub elapsed: f32,
}

fn update_scale_animation(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut ScaleAnimation)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut anim) in query.iter_mut() {
        anim.elapsed += time.delta_seconds();
        let t = (anim.elapsed / anim.duration).min(1.0);

        // イージング関数（ease-out）
        let t = 1.0 - (1.0 - t).powi(3);

        transform.scale = anim.start_scale.lerp(anim.end_scale, t);

        // アニメーション終了時にコンポーネントを削除
        if anim.elapsed >= anim.duration {
            commands.entity(entity).remove::<ScaleAnimation>();
        }
    }
}
```

### 3.4 カメラシェイク

#### 3.4.1 シェイク実装

```rust
#[derive(Component)]
pub struct CameraShake {
    pub intensity: f32,
    pub duration: f32,
    pub elapsed: f32,
}

fn update_camera_shake(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut CameraShake), With<Camera>>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut shake) in query.iter_mut() {
        shake.elapsed += time.delta_seconds();

        if shake.elapsed < shake.duration {
            // ランダムなオフセット
            let offset_x = (rand::random::<f32>() - 0.5) * shake.intensity;
            let offset_y = (rand::random::<f32>() - 0.5) * shake.intensity;

            // 強度を徐々に減衰
            let decay = 1.0 - (shake.elapsed / shake.duration);
            transform.translation.x += offset_x * decay;
            transform.translation.y += offset_y * decay;
        } else {
            // シェイク終了時にコンポーネントを削除
            commands.entity(entity).remove::<CameraShake>();
        }
    }
}
```

#### 3.4.2 シェイクのトリガー

```rust
fn trigger_camera_shake(
    mut camera_query: Query<Entity, With<Camera>>,
    mut commands: Commands,
    fruit_type: FruitType,
) {
    if let Ok(camera_entity) = camera_query.get_single_mut() {
        // フルーツのサイズに応じてシェイク強度を調整
        let intensity = match fruit_type {
            FruitType::Cherry | FruitType::Strawberry => 2.0,
            FruitType::Grape | FruitType::Dekopon => 5.0,
            FruitType::Persimmon | FruitType::Apple => 8.0,
            FruitType::Pear | FruitType::Peach => 12.0,
            FruitType::Pineapple | FruitType::Melon => 15.0,
            FruitType::Watermelon => 20.0,
        };

        commands.entity(camera_entity).insert(CameraShake {
            intensity,
            duration: 0.3,
            elapsed: 0.0,
        });
    }
}
```

## 4. 描画最適化

### 4.1 スプライトバッチング
Bevyは自動的に同じテクスチャを使用するスプライトをバッチ描画します。

### 4.2 Z-ordering最適化
- フルーツのZ値を動的に更新するシステムを最小限に
- フルーツが移動した時のみ更新

### 4.3 カリング
画面外のオブジェクトは自動的にカリングされます（Bevyのデフォルト動作）。

## 5. デバッグツール

### 5.1 Rapierデバッグレンダラー
開発中は物理コライダーを可視化：

```rust
.add_plugins(RapierDebugRenderPlugin::default())
```

### 5.2 カスタムギズモ
- 境界線の表示
- フルーツの速度ベクトル表示（オプション）
- グリッド表示（オプション）

## 6. パフォーマンス目標

### 6.1 目標フレームレート
- **最低**: 60 FPS
- **目標**: 144 FPS（高リフレッシュレートディスプレイ対応）

### 6.2 物理演算のステップ
- Rapierのデフォルト設定（60 Hz）を使用
- 必要に応じて調整可能

### 6.3 最大エンティティ数
- 同時存在フルーツ: 最大50個程度を想定
- パーティクル: 最大200個程度を想定

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 承認済み
