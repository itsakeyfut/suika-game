# Phase 5: 衝突検出と合体システム

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 4-6時間
**完了日**: -
**依存関係**: Phase 4

### 目的
同じフルーツの衝突を検出し、合体処理とスコアシステムを実装する。

### スコープ
- フルーツ衝突検出システムの実装
- 合体イベントの定義と処理
- スコア計算とコンボシステムの実装
- 合体時の基本エフェクト（スケールアニメーション）
- スイカ同士の消滅処理

## 前提条件

- Phase 4が完了している
- フルーツが正常にスポーンされ、物理挙動が機能している

## タスクリスト

### タスク 5.1: 合体イベントの定義

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-5, core

**説明**:
フルーツ合体イベントを定義し、イベント駆動のアーキテクチャを構築する。

**受け入れ基準**:
- [ ] `app/core/src/events.rs` が作成されている
- [ ] `FruitMergeEvent` イベントが定義されている
- [ ] イベントに必要な情報が含まれている（entity1, entity2, fruit_type, position）
- [ ] イベントがAppに登録されている

**実装ガイド**:
```rust
// app/core/src/events.rs
use bevy::prelude::*;
use crate::FruitType;

#[derive(Event)]
pub struct FruitMergeEvent {
    pub entity1: Entity,
    pub entity2: Entity,
    pub fruit_type: FruitType,
    pub position: Vec2,
}

// main.rs または lib.rs に追加
.add_event::<FruitMergeEvent>()
```

**関連ドキュメント**:
- [02_architecture.md - セクション2.3](../02_architecture.md)

---

### タスク 5.2: 衝突検出システムの実装

**優先度**: P0
**推定工数**: 2時間
**ラベル**: task, phase-5, physics

**説明**:
Rapier2DのCollisionEventを監視し、同じフルーツの衝突を検出する。

**受け入れ基準**:
- [ ] `app/core/src/systems/collision.rs` が作成されている
- [ ] `detect_fruit_collision` システムが実装されている
- [ ] CollisionEvent::Startedを正しく処理している
- [ ] 同じFruitTypeの衝突を検出している
- [ ] FruitMergeEventを発火している
- [ ] 重複検出を防止している（同じペアで複数回発火しない）
- [ ] エラーハンドリングが適切に実装されている

**実装ガイド**:
```rust
// app/core/src/systems/collision.rs
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::*;
use std::collections::HashSet;

#[derive(Resource, Default)]
pub struct ProcessedCollisions {
    pub pairs: HashSet<(Entity, Entity)>,
}

pub fn detect_fruit_collision(
    mut collision_events: EventReader<CollisionEvent>,
    fruit_query: Query<(&Fruit, &Transform)>,
    mut merge_events: EventWriter<events::FruitMergeEvent>,
    mut processed: ResMut<ProcessedCollisions>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            // 既に処理済みかチェック
            let pair = if e1 < e2 { (*e1, *e2) } else { (*e2, *e1) };
            if processed.pairs.contains(&pair) {
                continue;
            }

            // 両エンティティがFruitコンポーネントを持つかチェック
            if let (Ok((fruit1, transform1)), Ok((fruit2, transform2))) = (
                fruit_query.get(*e1),
                fruit_query.get(*e2),
            ) {
                // 同じフルーツタイプかチェック
                if fruit1.fruit_type == fruit2.fruit_type {
                    // 接触点の計算（中点）
                    let position = (transform1.translation.truncate()
                        + transform2.translation.truncate()) / 2.0;

                    // 合体イベントを発火
                    merge_events.send(events::FruitMergeEvent {
                        entity1: *e1,
                        entity2: *e2,
                        fruit_type: fruit1.fruit_type,
                        position,
                    });

                    // 処理済みとしてマーク
                    processed.pairs.insert(pair);
                }
            }
        }
    }
}

// フレームごとに処理済みペアをクリア
pub fn clear_processed_collisions(
    mut processed: ResMut<ProcessedCollisions>,
) {
    processed.pairs.clear();
}
```

**関連ドキュメント**:
- [02_architecture.md - セクション2.6](../02_architecture.md)

---

### タスク 5.3: 合体処理システムの実装

**優先度**: P0
**推定工数**: 1.5時間
**ラベル**: task, phase-5, core

**説明**:
FruitMergeEventを処理し、古いフルーツを削除して新しいフルーツを生成する。

**受け入れ基準**:
- [ ] `handle_fruit_merge` システムが実装されている
- [ ] 2つのフルーツエンティティが正しく削除される
- [ ] 次の段階のフルーツが接触点に生成される
- [ ] スイカ同士の場合、消滅のみで新フルーツは生成されない
- [ ] 合体後のフルーツに正しいパラメータが設定されている
- [ ] 重複despawnを防止している

**実装ガイド**:
```rust
// app/core/src/systems/collision.rs (続き)
pub fn handle_fruit_merge(
    mut commands: Commands,
    mut merge_events: EventReader<events::FruitMergeEvent>,
    fruit_query: Query<Entity, With<Fruit>>,
) {
    for event in merge_events.read() {
        // エンティティが存在するかチェック（重複despawn防止）
        if fruit_query.get(event.entity1).is_err()
            || fruit_query.get(event.entity2).is_err() {
            continue;
        }

        // 2つのフルーツを削除
        commands.entity(event.entity1).despawn();
        commands.entity(event.entity2).despawn();

        // 次の段階のフルーツをスポーン（スイカ以外）
        if let Some(next_fruit) = event.fruit_type.next() {
            spawn::spawn_fruit(
                &mut commands,
                next_fruit,
                event.position,
            );
        }
        // スイカ同士の場合は消滅のみ（新フルーツなし）
    }
}
```

**関連ドキュメント**:
- [01_specification.md - セクション2.2](../01_specification.md)

---

### タスク 5.4: スコアシステムの実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-5, score

**説明**:
フルーツ合体時のスコア計算とGameStateの更新を実装する。

**受け入れ基準**:
- [ ] `app/core/src/systems/score.rs` が作成されている
- [ ] `update_score_on_merge` システムが実装されている
- [ ] 合体時に正しい得点が加算される
- [ ] GameStateのscoreが更新される
- [ ] コンボシステムが実装されている
- [ ] ComboTimerが正しく管理されている
- [ ] コンボボーナスが正しく計算される（+10%, +20%, +30%, +50%）

**実装ガイド**:
```rust
// app/core/src/systems/score.rs
use bevy::prelude::*;
use crate::*;

pub fn update_score_on_merge(
    mut merge_events: EventReader<events::FruitMergeEvent>,
    mut game_state: ResMut<GameState>,
    mut combo_timer: ResMut<ComboTimer>,
    time: Res<Time>,
) {
    for event in merge_events.read() {
        let base_points = event.fruit_type.parameters().points;

        // コンボチェック
        if combo_timer.time_since_last_merge < combo_timer.combo_window {
            combo_timer.current_combo = (combo_timer.current_combo + 1).min(constants::COMBO_MAX);
        } else {
            combo_timer.current_combo = 1;
        }

        // コンボボーナス計算
        let bonus_multiplier = match combo_timer.current_combo {
            2 => 1.10,
            3 => 1.20,
            4 => 1.30,
            5..=u32::MAX => 1.50,
            _ => 1.0,
        };

        let final_points = (base_points as f32 * bonus_multiplier) as u32;
        game_state.score += final_points;

        // タイマーリセット
        combo_timer.time_since_last_merge = 0.0;
    }
}

pub fn update_combo_timer(
    mut combo_timer: ResMut<ComboTimer>,
    time: Res<Time>,
) {
    combo_timer.time_since_last_merge += time.delta_seconds();
}
```

**関連ドキュメント**:
- [01_specification.md - セクション2.3](../01_specification.md)

---

### タスク 5.5: 合体時の基本エフェクト（スケールアニメーション）

**優先度**: P1
**推定工数**: 1時間
**ラベル**: task, phase-5, effects

**説明**:
合体時に新しいフルーツがポップアップするスケールアニメーションを実装する。

**受け入れ基準**:
- [ ] `app/core/src/systems/effects.rs` が作成されている
- [ ] `MergeAnimation` コンポーネントが定義されている
- [ ] `animate_merge_scale` システムが実装されている
- [ ] 新しいフルーツが小さく出現し、徐々に元のサイズに拡大する
- [ ] アニメーション時間が適切（0.2〜0.3秒程度）
- [ ] アニメーション後、コンポーネントが削除される

**実装ガイド**:
```rust
// app/core/src/systems/effects.rs
use bevy::prelude::*;

#[derive(Component)]
pub struct MergeAnimation {
    pub elapsed: f32,
    pub duration: f32,
    pub start_scale: f32,
    pub target_scale: f32,
}

pub fn animate_merge_scale(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut MergeAnimation)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut anim) in query.iter_mut() {
        anim.elapsed += time.delta_seconds();

        if anim.elapsed >= anim.duration {
            // アニメーション完了
            transform.scale = Vec3::splat(anim.target_scale);
            commands.entity(entity).remove::<MergeAnimation>();
        } else {
            // イージング（ease-out）
            let t = anim.elapsed / anim.duration;
            let t = 1.0 - (1.0 - t).powi(3);  // cubic ease-out
            let scale = anim.start_scale + (anim.target_scale - anim.start_scale) * t;
            transform.scale = Vec3::splat(scale);
        }
    }
}

// spawn_fruit関数に追加
pub fn spawn_fruit_with_animation(
    commands: &mut Commands,
    fruit_type: FruitType,
    position: Vec2,
) -> Entity {
    let entity = spawn_fruit(commands, fruit_type, position);

    commands.entity(entity).insert(MergeAnimation {
        elapsed: 0.0,
        duration: 0.25,
        start_scale: 0.3,
        target_scale: 1.0,
    });

    entity
}
```

**関連ドキュメント**:
- [03_physics_rendering.md - セクション5](../03_physics_rendering.md)

---

### タスク 5.6: システムの統合とテスト

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-5, integration

**説明**:
Phase 5で実装したすべてのシステムをメインアプリに統合する。

**受け入れ基準**:
- [ ] すべてのシステムがmain.rsに追加されている
- [ ] システムの実行順序が適切に設定されている
- [ ] ProcessedCollisionsリソースが初期化されている
- [ ] イベントが登録されている
- [ ] `cargo run` で合体システムが動作する

**実装ガイド**:
```rust
// main.rs
use suika_game_core::{events::*, systems::*};

fn main() {
    App::new()
        // ... プラグイン
        .add_event::<FruitMergeEvent>()
        .insert_resource(collision::ProcessedCollisions::default())
        .add_systems(Update, (
            collision::detect_fruit_collision,
            collision::handle_fruit_merge,
            score::update_score_on_merge,
            score::update_combo_timer,
            effects::animate_merge_scale,
        ).chain().run_if(in_state(AppState::Playing)))
        .add_systems(Last, collision::clear_processed_collisions)
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
- [ ] 同じフルーツ同士が接触すると合体する
- [ ] 合体後、正しい次の段階のフルーツが生成される
- [ ] スイカ同士が接触すると両方消滅する
- [ ] スコアが正しく加算される
- [ ] コンボシステムが機能する（2秒以内の連続合体でボーナス）
- [ ] 合体時のスケールアニメーションが表示される
- [ ] 重複despawnエラーが発生しない

### 検証手順

```bash
# ゲーム実行
cargo run

# 確認項目:
# 1. 同じ色のフルーツを2つ落として接触させる
# 2. 合体して次の段階のフルーツに進化する
# 3. スコアが加算される（画面には表示されないが、ログで確認可能）
# 4. 新しいフルーツが小さく出現し、徐々に拡大する
# 5. 2秒以内に連続で合体するとコンボが増える
# 6. スイカを2つ作って接触させると消滅する
# 7. エラーが発生しない
```

## 完了条件

- [ ] すべてのタスクが完了している
- [ ] すべての検証項目が合格している
- [ ] ドキュメントが更新されている（必要な場合）
- [ ] コードがフォーマットされている（`just fmt`）
- [ ] Clippyチェックが通っている（`just clippy`）

## 次のフェーズ

Phase 5完了 → 次は **Phase 6: ゲームオーバー判定** に進む

## 備考

- Phase 5完了時点で、ゲームの核となる合体メカニクスが機能する
- コンボシステムは後のPhaseでUI表示を追加する（現在は内部処理のみ）
- スケールアニメーションは基本的なもので、Phase 8でより豊かなエフェクトを追加
- 重複despawn問題はProcessedCollisionsリソースで解決
- スイカ合体時の消滅は満足感のある演出を後で追加する予定

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 未着手
