# Phase 8: リッチなビジュアルエフェクト

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 4-6時間
**完了日**: -
**依存関係**: Phase 7

### 目的
パーティクルエフェクト、画面シェイク、アニメーションを実装し、ゲームに視覚的な魅力を追加する。

### スコープ
- フルーツ合体時のパーティクルエフェクト
- 大きなフルーツ合体時の画面シェイク
- スイカ完成時の特別エフェクト
- フルーツスポーン時のアニメーション改善
- UI要素のアニメーション（スコア加算時のパルス等）
- 2.5D深度エフェクト（Z軸によるサイズ変化）

## 前提条件

- Phase 7が完了している
- 基本UIが実装されている
- 合体システムが正常に動作している

## タスクリスト

### タスク 8.1: パーティクルシステムの基盤実装

**優先度**: P0
**推定工数**: 1.5時間
**ラベル**: task, phase-8, effects

**説明**:
シンプルなパーティクルシステムを実装し、合体時にパーティクルを生成する。

**受け入れ基準**:
- [ ] `app/core/src/systems/particles.rs` が作成されている
- [ ] `Particle` コンポーネントが定義されている
- [ ] `spawn_particles` 関数が実装されている
- [ ] `update_particles` システムが実装されている
- [ ] パーティクルが時間経過で消滅する
- [ ] パーティクルに速度と重力が適用される

**実装ガイド**:
```rust
// app/core/src/systems/particles.rs
use bevy::prelude::*;

#[derive(Component)]
pub struct Particle {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub velocity: Vec2,
}

pub fn spawn_particles(
    commands: &mut Commands,
    position: Vec2,
    color: Color,
    count: u32,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..count {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed = rng.gen_range(50.0..200.0);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;

        commands.spawn((
            Particle {
                lifetime: 0.0,
                max_lifetime: rng.gen_range(0.3..0.8),
                velocity,
            },
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(rng.gen_range(3.0..8.0))),
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 10.0),
                ..default()
            },
        ));
    }
}

pub fn update_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Particle, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle, mut sprite) in query.iter_mut() {
        particle.lifetime += time.delta_seconds();

        // 寿命チェック
        if particle.lifetime >= particle.max_lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        // 速度を適用（重力効果）
        particle.velocity.y -= 300.0 * time.delta_seconds();
        transform.translation.x += particle.velocity.x * time.delta_seconds();
        transform.translation.y += particle.velocity.y * time.delta_seconds();

        // フェードアウト
        let alpha = 1.0 - (particle.lifetime / particle.max_lifetime);
        sprite.color.set_alpha(alpha);
    }
}
```

**関連ドキュメント**:
- [03_physics_rendering.md - セクション5.2](../03_physics_rendering.md)

---

### タスク 8.2: 合体時パーティクルエフェクトの統合

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-8, effects

**説明**:
FruitMergeEventを監視し、合体時にパーティクルを生成する。

**受け入れ基準**:
- [ ] `spawn_merge_particles` システムが実装されている
- [ ] FruitMergeEventを監視している
- [ ] フルーツの色に応じたパーティクルが生成される
- [ ] フルーツサイズに応じてパーティクル数が変化する
- [ ] パーティクルが合体位置から放射状に飛び散る

**実装ガイド**:
```rust
// app/core/src/systems/particles.rs (続き)
use crate::events::*;

pub fn spawn_merge_particles(
    mut commands: Commands,
    mut merge_events: EventReader<FruitMergeEvent>,
) {
    for event in merge_events.read() {
        let color = event.fruit_type.placeholder_color();

        // フルーツサイズに応じたパーティクル数
        let particle_count = match event.fruit_type {
            FruitType::Cherry | FruitType::Strawberry => 8,
            FruitType::Grape | FruitType::Dekopon | FruitType::Persimmon => 12,
            FruitType::Apple | FruitType::Pear | FruitType::Peach => 16,
            FruitType::Pineapple | FruitType::Melon => 20,
            FruitType::Watermelon => 30,
        };

        spawn_particles(
            &mut commands,
            event.position,
            color,
            particle_count,
        );
    }
}
```

**関連ドキュメント**:
- [03_physics_rendering.md - セクション5.2](../03_physics_rendering.md)

---

### タスク 8.3: 画面シェイクシステムの実装

**優先度**: P1
**推定工数**: 1.5時間
**ラベル**: task, phase-8, effects

**説明**:
大きなフルーツの合体時にカメラを揺らして衝撃を演出する。

**受け入れ基準**:
- [ ] `app/core/src/systems/camera_shake.rs` が作成されている
- [ ] `CameraShake` コンポーネントが定義されている
- [ ] `trigger_shake_on_merge` システムが実装されている
- [ ] `apply_camera_shake` システムが実装されている
- [ ] 大きいフルーツほど強く揺れる
- [ ] シェイクが滑らかに減衰する

**実装ガイド**:
```rust
// app/core/src/systems/camera_shake.rs
use bevy::prelude::*;
use crate::events::*;

#[derive(Component)]
pub struct CameraShake {
    pub intensity: f32,
    pub duration: f32,
    pub elapsed: f32,
}

pub fn trigger_shake_on_merge(
    mut commands: Commands,
    mut merge_events: EventReader<FruitMergeEvent>,
    camera_query: Query<Entity, With<Camera2d>>,
) {
    for event in merge_events.read() {
        // 大きいフルーツのみシェイク
        let intensity = match event.fruit_type {
            FruitType::Peach => 3.0,
            FruitType::Pineapple => 5.0,
            FruitType::Melon => 8.0,
            FruitType::Watermelon => 12.0,
            _ => continue,  // 小さいフルーツはシェイクなし
        };

        if let Ok(camera_entity) = camera_query.get_single() {
            commands.entity(camera_entity).insert(CameraShake {
                intensity,
                duration: 0.3,
                elapsed: 0.0,
            });
        }
    }
}

pub fn apply_camera_shake(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut CameraShake), With<Camera2d>>,
    time: Res<Time>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for (entity, mut transform, mut shake) in query.iter_mut() {
        shake.elapsed += time.delta_seconds();

        if shake.elapsed >= shake.duration {
            // シェイク終了、カメラをリセット
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
            commands.entity(entity).remove::<CameraShake>();
        } else {
            // 減衰シェイク
            let progress = shake.elapsed / shake.duration;
            let current_intensity = shake.intensity * (1.0 - progress);

            let offset_x = rng.gen_range(-current_intensity..current_intensity);
            let offset_y = rng.gen_range(-current_intensity..current_intensity);

            transform.translation.x = offset_x;
            transform.translation.y = offset_y;
        }
    }
}
```

**関連ドキュメント**:
- [03_physics_rendering.md - セクション5.3](../03_physics_rendering.md)

---

### タスク 8.4: スイカ完成時の特別エフェクト

**優先度**: P1
**推定工数**: 1時間
**ラベル**: task, phase-8, effects

**説明**:
スイカが完成した際の特別な演出を実装する。

**受け入れ基準**:
- [ ] `spawn_watermelon_effect` システムが実装されている
- [ ] スイカ完成時に大量のパーティクルが生成される
- [ ] 複数色のパーティクル（虹色）が使用される
- [ ] 強い画面シェイクが発生する
- [ ] スイカに一瞬光るエフェクトが追加される（オプション）

**実装ガイド**:
```rust
// app/core/src/systems/particles.rs (続き)
pub fn spawn_watermelon_effect(
    mut commands: Commands,
    mut merge_events: EventReader<FruitMergeEvent>,
) {
    for event in merge_events.read() {
        // メロン同士の合体（= スイカ誕生）
        if event.fruit_type == FruitType::Melon {
            // 虹色パーティクル
            let colors = [
                Color::srgb(1.0, 0.0, 0.0),  // 赤
                Color::srgb(1.0, 0.5, 0.0),  // オレンジ
                Color::srgb(1.0, 1.0, 0.0),  // 黄
                Color::srgb(0.0, 1.0, 0.0),  // 緑
                Color::srgb(0.0, 0.5, 1.0),  // 青
                Color::srgb(0.5, 0.0, 1.0),  // 紫
            ];

            for color in colors.iter() {
                spawn_particles(
                    &mut commands,
                    event.position,
                    *color,
                    15,
                );
            }

            info!("🍉 Watermelon created!");
        }
    }
}
```

**関連ドキュメント**:
- [01_specification.md - セクション3.2](../01_specification.md)

---

### タスク 8.5: スコア加算時のパルスアニメーション

**優先度**: P2
**推定工数**: 1時間
**ラベル**: task, phase-8, ui-effects

**説明**:
スコアが加算された際にスコア表示を一瞬大きくするパルスアニメーションを実装する。

**受け入れ基準**:
- [ ] `app/ui/src/systems/animations.rs` が作成されている
- [ ] `ScorePulseAnimation` コンポーネントが定義されている
- [ ] スコア更新時にアニメーションがトリガーされる
- [ ] 0.2秒程度でスケールが1.0 → 1.2 → 1.0に変化する
- [ ] アニメーションが滑らかである

**実装ガイド**:
```rust
// app/ui/src/systems/animations.rs
use bevy::prelude::*;
use suika_game_core::*;
use crate::screens::hud::ScoreText;

#[derive(Component)]
pub struct ScorePulseAnimation {
    pub elapsed: f32,
    pub duration: f32,
}

pub fn trigger_score_pulse(
    mut commands: Commands,
    game_state: Res<GameState>,
    score_query: Query<Entity, With<ScoreText>>,
) {
    if game_state.is_changed() && game_state.score > 0 {
        for entity in score_query.iter() {
            commands.entity(entity).insert(ScorePulseAnimation {
                elapsed: 0.0,
                duration: 0.2,
            });
        }
    }
}

pub fn animate_score_pulse(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut ScorePulseAnimation)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut anim) in query.iter_mut() {
        anim.elapsed += time.delta_seconds();

        if anim.elapsed >= anim.duration {
            transform.scale = Vec3::splat(1.0);
            commands.entity(entity).remove::<ScorePulseAnimation>();
        } else {
            // Sin波でパルス
            let t = anim.elapsed / anim.duration;
            let scale = 1.0 + (t * std::f32::consts::PI).sin() * 0.2;
            transform.scale = Vec3::splat(scale);
        }
    }
}
```

**関連ドキュメント**:
- [04_ui_ux.md - セクション4.1](../04_ui_ux.md)

---

### タスク 8.6: 2.5D深度エフェクトの実装

**優先度**: P2
**推定工数**: 1時間
**ラベル**: task, phase-8, rendering

**説明**:
フルーツのZ座標に応じてサイズを変化させ、奥行き感を演出する。

**受け入れ基準**:
- [ ] `apply_depth_scale` システムが実装されている
- [ ] フルーツのZ座標が-100〜100の範囲で変動する
- [ ] Z座標が大きい（手前）ほどスプライトが大きく表示される
- [ ] スケール変化が自然である（0.9〜1.1倍程度）
- [ ] パフォーマンスに影響がない

**実装ガイド**:
```rust
// app/core/src/systems/effects.rs (追加)
pub fn apply_depth_scale(
    mut query: Query<(&mut Transform, &Sprite), With<Fruit>>,
) {
    for (mut transform, _sprite) in query.iter_mut() {
        // Z座標に基づくスケール（-100 ~ 100の範囲を想定）
        let depth_factor = 1.0 + (transform.translation.z / 1000.0);
        let depth_scale = depth_factor.clamp(0.9, 1.1);

        // XYスケールのみ変更（Zは保持）
        let current_z_scale = transform.scale.z;
        transform.scale = Vec3::new(depth_scale, depth_scale, current_z_scale);
    }
}
```

**関連ドキュメント**:
- [03_physics_rendering.md - セクション4](../03_physics_rendering.md)

---

### タスク 8.7: フルーツスポーン時のアニメーション改善

**優先度**: P2
**推定工数**: 0.5時間
**ラベル**: task, phase-8, effects

**説明**:
既存のスケールアニメーションに加えて、回転やバウンスエフェクトを追加する。

**受け入れ基準**:
- [ ] `MergeAnimation` が拡張されている
- [ ] 出現時に軽く回転する
- [ ] バウンス効果が追加されている（ease-out-back）
- [ ] アニメーションが自然である

**実装ガイド**:
```rust
// app/core/src/systems/effects.rs (MergeAnimation拡張)
pub fn animate_merge_scale(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut MergeAnimation)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut anim) in query.iter_mut() {
        anim.elapsed += time.delta_seconds();

        if anim.elapsed >= anim.duration {
            transform.scale = Vec3::splat(anim.target_scale);
            transform.rotation = Quat::IDENTITY;
            commands.entity(entity).remove::<MergeAnimation>();
        } else {
            let t = anim.elapsed / anim.duration;

            // Ease-out-back（バウンス効果）
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            let ease_t = 1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2);

            let scale = anim.start_scale + (anim.target_scale - anim.start_scale) * ease_t;
            transform.scale = Vec3::splat(scale);

            // 軽い回転
            let rotation_angle = (1.0 - t) * std::f32::consts::PI * 0.5;
            transform.rotation = Quat::from_rotation_z(rotation_angle);
        }
    }
}
```

**関連ドキュメント**:
- [03_physics_rendering.md - セクション5.1](../03_physics_rendering.md)

---

### タスク 8.8: システムの統合とテスト

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-8, integration

**説明**:
Phase 8で実装したすべてのエフェクトシステムをメインアプリに統合する。

**受け入れ基準**:
- [ ] すべてのシステムがmain.rsに追加されている
- [ ] システムの実行順序が適切に設定されている
- [ ] `cargo run` でエフェクトが動作する
- [ ] パフォーマンスに問題がない（60fps維持）

**実装ガイド**:
```rust
// main.rs
use suika_game_core::systems::*;

fn main() {
    App::new()
        // ... プラグイン
        .add_systems(Update, (
            // パーティクル
            particles::spawn_merge_particles,
            particles::spawn_watermelon_effect,
            particles::update_particles,

            // カメラシェイク
            camera_shake::trigger_shake_on_merge,
            camera_shake::apply_camera_shake,

            // その他エフェクト
            effects::apply_depth_scale,
        ).run_if(in_state(AppState::Playing)))
        .run();
}
```

**関連ドキュメント**:
- [02_architecture.md - セクション2.4](../02_architecture.md)

---

## フェーズ検証

### 検証項目

- [ ] すべてのタスクが完了している
- [ ] `cargo build --workspace` が成功する
- [ ] `cargo run` でゲームが起動する
- [ ] フルーツ合体時にパーティクルが飛び散る
- [ ] 大きなフルーツの合体時に画面が揺れる
- [ ] スイカ完成時に虹色のパーティクルと強いシェイクが発生する
- [ ] スコア加算時にスコア表示がパルスする
- [ ] フルーツ出現時のアニメーションが改善されている
- [ ] 2.5D深度エフェクトが機能している
- [ ] 60fps以上を維持している

### 検証手順

```bash
# ゲーム実行
cargo run

# 確認項目:
# 1. フルーツを合体させるとパーティクルが飛び散る
# 2. 大きいフルーツ（桃以上）の合体で画面が揺れる
# 3. スイカを作成すると虹色のパーティクルと強いシェイクが発生
# 4. スコアが加算されるとスコア表示が一瞬大きくなる
# 5. フルーツが出現時に回転しながら拡大する
# 6. パーティクルが自然に落下して消える
# 7. エフェクトが重なってもフレームレートが安定している
```

## 完了条件

- [ ] すべてのタスクが完了している
- [ ] すべての検証項目が合格している
- [ ] ドキュメントが更新されている（必要な場合）
- [ ] コードがフォーマットされている（`just fmt`）
- [ ] Clippyチェックが通っている（`just clippy`）

## 次のフェーズ

Phase 8完了 → 次は **Phase 9: サウンド統合** に進む

## 備考

- Phase 8完了時点で、ゲームが視覚的に魅力的になる
- パーティクルシステムは簡易的な実装（本格的なパーティクルライブラリは使用しない）
- 画面シェイクの強度は後で調整可能（Phase 10で微調整）
- 2.5D深度エフェクトはオプション機能（パフォーマンス問題があれば無効化）
- より高度なエフェクト（ブラー、グロー等）は将来の拡張として検討
- パーティクル数が多い場合、パフォーマンスに注意

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 未着手
