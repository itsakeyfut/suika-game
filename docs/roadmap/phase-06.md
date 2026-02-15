# Phase 6: ゲームオーバー検出

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 2-3時間
**完了日**: -
**依存関係**: Phase 5

### 目的
境界線を超えたフルーツを検出し、タイマーベースのゲームオーバー判定を実装する。

### スコープ
- 境界線超過検出システムの実装
- 3.0秒タイマーベースのゲームオーバー判定
- ゲームオーバー警告表示（境界線の点滅）
- ハイスコア更新と保存
- 状態遷移処理（Playing → GameOver）

## 前提条件

- Phase 5が完了している
- フルーツ合体システムが正常に動作している
- GameOverTimerリソースが定義されている

## タスクリスト

### タスク 6.1: 境界線超過検出システムの実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-6, game-over

**説明**:
フルーツが境界線を超えているかを毎フレームチェックし、GameOverTimerを更新する。

**受け入れ基準**:
- [ ] `app/core/src/systems/boundary.rs` が作成されている
- [ ] `check_boundary_overflow` システムが実装されている
- [ ] フルーツのY座標がBOUNDARY_LINE_Yを超えているかチェックしている
- [ ] GameOverTimerが正しく更新される（超過時: 加算、非超過時: リセット）
- [ ] is_warning フラグが適切に設定される
- [ ] 複数フルーツが境界線を超えた場合も正しく処理される

**実装ガイド**:
```rust
// app/core/src/systems/boundary.rs
use bevy::prelude::*;
use crate::*;

pub fn check_boundary_overflow(
    fruit_query: Query<&Transform, With<Fruit>>,
    mut game_over_timer: ResMut<GameOverTimer>,
    time: Res<Time>,
) {
    let mut has_overflow = false;

    // すべてのフルーツをチェック
    for transform in fruit_query.iter() {
        if transform.translation.y > constants::BOUNDARY_LINE_Y {
            has_overflow = true;
            break;
        }
    }

    if has_overflow {
        // タイマーを進める
        game_over_timer.time_over_boundary += time.delta_seconds();

        // 警告フラグを設定
        if game_over_timer.time_over_boundary > 0.0 {
            game_over_timer.is_warning = true;
        }
    } else {
        // タイマーをリセット
        game_over_timer.time_over_boundary = 0.0;
        game_over_timer.is_warning = false;
    }
}
```

**関連ドキュメント**:
- [01_specification.md - セクション2.2](../01_specification.md)

---

### タスク 6.2: ゲームオーバー判定システムの実装

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-6, game-over

**説明**:
GameOverTimerが3.0秒を超えた場合にゲームオーバー状態に遷移する。

**受け入れ基準**:
- [ ] `trigger_game_over` システムが実装されている
- [ ] タイマーが3.0秒を超えたらGameOverイベントを発火する
- [ ] AppStateがPlaying → GameOverに遷移する
- [ ] ゲームオーバー判定後、タイマーがリセットされる
- [ ] 重複トリガーを防止している

**実装ガイド**:
```rust
// app/core/src/systems/boundary.rs (続き)
pub fn trigger_game_over(
    game_over_timer: Res<GameOverTimer>,
    mut next_state: ResMut<NextState<AppState>>,
    current_state: Res<State<AppState>>,
) {
    // Playing状態でのみチェック
    if *current_state.get() != AppState::Playing {
        return;
    }

    // 3.0秒を超えたらゲームオーバー
    if game_over_timer.time_over_boundary >= game_over_timer.warning_threshold {
        info!("Game Over! Time over boundary: {:.2}s", game_over_timer.time_over_boundary);
        next_state.set(AppState::GameOver);
    }
}
```

**関連ドキュメント**:
- [01_specification.md - セクション2.2](../01_specification.md)

---

### タスク 6.3: 境界線警告エフェクト（点滅）の実装

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-6, effects

**説明**:
境界線を超過している間、境界線を赤く点滅させて警告する。

**受け入れ基準**:
- [ ] `animate_boundary_warning` システムが実装されている
- [ ] is_warning フラグがtrueの時に境界線が点滅する
- [ ] 点滅周期が1秒間隔である
- [ ] 警告解除時に境界線が通常の色に戻る
- [ ] 点滅が滑らかである（急激な変化でない）

**実装ガイド**:
```rust
// app/core/src/systems/effects.rs に追加
pub fn animate_boundary_warning(
    game_over_timer: Res<GameOverTimer>,
    mut boundary_query: Query<&mut Sprite, With<BoundaryLine>>,
    time: Res<Time>,
) {
    for mut sprite in boundary_query.iter_mut() {
        if game_over_timer.is_warning {
            // 1秒周期で点滅（0.5秒ごとに明滅）
            let blink = (time.elapsed_seconds() * 2.0).sin() > 0.0;

            if blink {
                // 明るい赤
                sprite.color = Color::srgba(1.0, 0.0, 0.0, 0.8);
            } else {
                // 暗い赤
                sprite.color = Color::srgba(0.6, 0.0, 0.0, 0.4);
            }
        } else {
            // 通常の色（半透明の赤）
            sprite.color = Color::srgba(1.0, 0.0, 0.0, 0.5);
        }
    }
}
```

**関連ドキュメント**:
- [03_physics_rendering.md - セクション5](../03_physics_rendering.md)

---

### タスク 6.4: ハイスコア更新と保存

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-6, persistence

**説明**:
ゲームオーバー時にハイスコアを更新し、ファイルに保存する。

**受け入れ基準**:
- [ ] `save_highscore_on_game_over` システムが実装されている
- [ ] OnEnter(AppState::GameOver) で実行される
- [ ] 現在のスコアがハイスコアを超えている場合のみ更新する
- [ ] HighscoreDataが正しく保存される
- [ ] ファイル保存エラーが適切にハンドリングされる
- [ ] ログに保存結果が出力される

**実装ガイド**:
```rust
// app/core/src/systems/game_over.rs
use bevy::prelude::*;
use crate::*;

pub fn save_highscore_on_game_over(
    mut game_state: ResMut<GameState>,
) {
    // ハイスコア更新チェック
    if game_state.score > game_state.highscore {
        info!("New highscore! {} -> {}", game_state.highscore, game_state.score);
        game_state.highscore = game_state.score;

        // ファイルに保存
        let highscore_data = persistence::HighscoreData {
            highscore: game_state.highscore,
        };

        match persistence::save_highscore(&highscore_data) {
            Ok(_) => info!("Highscore saved successfully"),
            Err(e) => error!("Failed to save highscore: {}", e),
        }
    } else {
        info!("Final score: {} (Highscore: {})", game_state.score, game_state.highscore);
    }
}
```

**関連ドキュメント**:
- [02_architecture.md - セクション4](../02_architecture.md)

---

### タスク 6.5: ゲーム状態のリセット機能

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-6, state-management

**説明**:
新しいゲームを開始する際に、ゲーム状態を初期状態にリセットする。

**受け入れ基準**:
- [ ] `reset_game_state` システムが実装されている
- [ ] OnEnter(AppState::Playing) で実行される
- [ ] GameState（score, elapsed_time）がリセットされる
- [ ] ComboTimerがリセットされる
- [ ] GameOverTimerがリセットされる
- [ ] 既存のフルーツがすべて削除される
- [ ] ハイスコアは保持される

**実装ガイド**:
```rust
// app/core/src/systems/game_over.rs (続き)
pub fn reset_game_state(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut combo_timer: ResMut<ComboTimer>,
    mut game_over_timer: ResMut<GameOverTimer>,
    fruit_query: Query<Entity, With<Fruit>>,
) {
    // スコアと経過時間をリセット（ハイスコアは保持）
    let highscore = game_state.highscore;
    *game_state = GameState {
        score: 0,
        highscore,
        elapsed_time: 0.0,
    };

    // コンボタイマーをリセット
    *combo_timer = ComboTimer::default();

    // ゲームオーバータイマーをリセット
    *game_over_timer = GameOverTimer::default();

    // 既存のフルーツをすべて削除
    for entity in fruit_query.iter() {
        commands.entity(entity).despawn();
    }

    info!("Game state reset. Highscore: {}", highscore);
}
```

**関連ドキュメント**:
- [02_architecture.md - セクション3](../02_architecture.md)

---

### タスク 6.6: システムの統合とテスト

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-6, integration

**説明**:
Phase 6で実装したすべてのシステムをメインアプリに統合する。

**受け入れ基準**:
- [ ] すべてのシステムがmain.rsに追加されている
- [ ] システムの実行順序が適切に設定されている
- [ ] OnEnterとOnExitのスケジュールが正しく設定されている
- [ ] `cargo run` でゲームオーバー判定が動作する
- [ ] 警告なしでビルドできる

**実装ガイド**:
```rust
// main.rs
use suika_game_core::systems::*;

fn main() {
    App::new()
        // ... プラグイン
        .add_systems(Update, (
            boundary::check_boundary_overflow,
            boundary::trigger_game_over,
            effects::animate_boundary_warning,
        ).run_if(in_state(AppState::Playing)))
        .add_systems(OnEnter(AppState::Playing), (
            game_over::reset_game_state,
        ))
        .add_systems(OnEnter(AppState::GameOver), (
            game_over::save_highscore_on_game_over,
        ))
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
- [ ] フルーツが境界線を超えると警告が表示される（境界線が点滅）
- [ ] 3.0秒以上超過状態が続くとゲームオーバーになる
- [ ] 境界線を下回るとタイマーがリセットされる
- [ ] ゲームオーバー時にハイスコアが更新される
- [ ] ハイスコアがファイルに保存される
- [ ] 新しいゲームを開始すると状態がリセットされる
- [ ] ハイスコアは保持される

### 検証手順

```bash
# ゲーム実行
cargo run

# 確認項目:
# 1. フルーツを大量に落として境界線を超えさせる
# 2. 境界線が点滅する（警告状態）
# 3. 3秒以上超過状態を維持するとゲームオーバーになる
# 4. 境界線を下回ると点滅が止まり、タイマーがリセットされる
# 5. ゲームオーバー時にハイスコアが更新される
# 6. save/highscore.json ファイルが作成/更新される
# 7. 新しいゲームを開始するとスコアが0にリセットされる
# 8. ハイスコアは保持される
```

## 完了条件

- [ ] すべてのタスクが完了している
- [ ] すべての検証項目が合格している
- [ ] ドキュメントが更新されている（必要な場合）
- [ ] コードがフォーマットされている（`just fmt`）
- [ ] Clippyチェックが通っている（`just clippy`）

## 次のフェーズ

Phase 6完了 → 次は **Phase 7: 基本UIの実装** に進む

## 備考

- Phase 6完了時点で、ゲームの基本的なゲームループが完成する
- UIはまだ実装されていないため、ログでスコアやゲームオーバーを確認する
- 境界線の点滅はPhase 7でUI実装後により洗練される可能性がある
- ゲームオーバー時の演出（効果音、画面遷移）はPhase 9で追加
- タイマーの3.0秒は調整可能（Phase 10で微調整）

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 未着手
