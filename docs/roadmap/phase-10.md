# Phase 10: 調整とポリッシュ

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 6-8時間
**完了日**: -
**依存関係**: Phase 9

### 目的
ゲームプレイを最適化し、バランス調整、バグ修正、パフォーマンス改善を行う。

### スコープ
- 物理パラメータの微調整（重力、反発係数、摩擦等）
- ゲームバランス調整（フルーツサイズ、得点、コンボタイマー等）
- UI/UXの改善（レイアウト、フォント、色調整）
- パフォーマンス最適化
- バグ修正とエッジケース処理
- プレイテストとフィードバック反映
- 経過時間システムの実装

## 前提条件

- Phase 9が完了している
- ゲームのすべての基本機能が実装されている
- BGMと効果音が統合されている

## タスクリスト

### タスク 10.1: 物理パラメータの調整

**優先度**: P0
**推定工数**: 2時間
**ラベル**: task, phase-10, tuning

**説明**:
重力、反発係数、摩擦係数などの物理パラメータを調整し、最適なゲーム感を実現する。

**受け入れ基準**:
- [ ] 重力が適切に設定されている（フルーツの落下速度が快適）
- [ ] 各フルーツの反発係数が調整されている（jiggle effectが自然）
- [ ] 摩擦係数が適切に設定されている（転がり具合が適切）
- [ ] フルーツが安定して静止する（過度な揺れがない）
- [ ] 合体時の物理挙動が自然である
- [ ] パラメータがconstants.rsに整理されている

**実装ガイド**:
```rust
// app/core/src/constants.rs
// 重力調整（現在: -980.0）
pub const GRAVITY: f32 = -980.0;  // 調整候補: -800.0 ~ -1200.0

// 反発係数（フルーツごとに調整）
pub const RESTITUTION_SMALL: f32 = 0.3;   // 小さいフルーツ
pub const RESTITUTION_MEDIUM: f32 = 0.25;  // 中サイズ
pub const RESTITUTION_LARGE: f32 = 0.2;    // 大きいフルーツ

// 摩擦係数
pub const FRICTION_FRUIT: f32 = 0.5;
pub const FRICTION_WALL: f32 = 0.5;

// Dampingパラメータ（揺れの減衰）
pub const LINEAR_DAMPING: f32 = 0.5;
pub const ANGULAR_DAMPING: f32 = 1.0;

// フルーツサイズ調整（半径、ピクセル単位）
pub const FRUIT_RADII: [f32; 11] = [
    20.0,  // Cherry
    30.0,  // Strawberry
    40.0,  // Grape
    50.0,  // Dekopon
    60.0,  // Persimmon
    70.0,  // Apple
    80.0,  // Pear
    90.0,  // Peach
    100.0, // Pineapple
    110.0, // Melon
    120.0, // Watermelon
];

// 質量（半径に比例、または個別調整）
pub fn fruit_mass(radius: f32) -> f32 {
    // 質量 = 半径^2 * 密度係数
    radius * radius * 0.01
}
```

**調整方法**:
1. 実際にプレイして感覚を確認
2. 重力を調整（落下速度）
3. 反発係数を調整（跳ね返り具合）
4. 摩擦係数を調整（滑り具合）
5. Dampingを調整（揺れの減衰速度）
6. 繰り返しテスト

**関連ドキュメント**:
- [03_physics_rendering.md - セクション2](../03_physics_rendering.md)

---

### タスク 10.2: ゲームバランス調整

**優先度**: P0
**推定工数**: 1.5時間
**ラベル**: task, phase-10, tuning

**説明**:
得点、コンボタイマー、ゲームオーバータイマーなどのゲームバランスを調整する。

**受け入れ基準**:
- [ ] 各フルーツの得点が適切に設定されている
- [ ] コンボウィンドウ（2.0秒）が適切である
- [ ] ゲームオーバータイマー（3.0秒）が適切である
- [ ] コンボボーナスの倍率が適切である
- [ ] フルーツ出現確率が適切である（現在は均等）
- [ ] ゲーム難易度が適切である

**実装ガイド**:
```rust
// app/core/src/constants.rs (続き)

// 得点テーブル（調整候補）
pub const FRUIT_POINTS: [u32; 11] = [
    10,    // Cherry
    20,    // Strawberry
    40,    // Grape
    80,    // Dekopon
    160,   // Persimmon
    320,   // Apple
    640,   // Pear
    1280,  // Peach
    2560,  // Pineapple
    5120,  // Melon
    10240, // Watermelon
];

// コンボシステム
pub const COMBO_WINDOW: f32 = 2.0;  // 調整候補: 1.5 ~ 3.0秒
pub const COMBO_MAX: u32 = 10;

// コンボボーナス（調整候補）
pub fn combo_bonus_multiplier(combo: u32) -> f32 {
    match combo {
        2 => 1.10,      // +10%
        3 => 1.20,      // +20%
        4 => 1.30,      // +30%
        5..=u32::MAX => 1.50,  // +50%
        _ => 1.0,
    }
}

// ゲームオーバータイマー
pub const GAME_OVER_TIMER: f32 = 3.0;  // 調整候補: 2.0 ~ 4.0秒

// フルーツ出現確率（将来の拡張）
// 現在は均等だが、難易度調整で使用可能
pub fn fruit_spawn_weights() -> [f32; 5] {
    // Cherry, Strawberry, Grape, Dekopon, Persimmon
    [1.0, 1.0, 1.0, 1.0, 1.0]  // 均等
    // 難易度調整例: [2.0, 1.5, 1.0, 0.8, 0.5]  // 小さいフルーツが出やすい
}
```

**調整方法**:
1. 実際にプレイしてスコア推移を確認
2. 得点が低すぎる/高すぎる場合は調整
3. コンボが発生しやすすぎる/しにくすぎる場合はタイマー調整
4. ゲームオーバーまでの時間を確認
5. 繰り返しテスト

**関連ドキュメント**:
- [01_specification.md - セクション2.3](../01_specification.md)

---

### タスク 10.3: UI/UXの改善と微調整

**優先度**: P1
**推定工数**: 2時間
**ラベル**: task, phase-10, ui

**説明**:
UIレイアウト、フォント、色、アニメーションを微調整し、ユーザビリティを向上させる。

**受け入れ基準**:
- [ ] すべてのテキストが読みやすい
- [ ] ボタンのサイズと配置が適切である
- [ ] 色のコントラストが十分である
- [ ] HUD要素が邪魔にならない
- [ ] アニメーションがスムーズである
- [ ] レスポンシブに対応している（異なるウィンドウサイズ）

**実装ガイド**:
```rust
// app/ui/src/styles.rs (調整)

// カラーパレット微調整
pub const BG_COLOR: Color = Color::srgb(0.95, 0.95, 0.90);
pub const PRIMARY_COLOR: Color = Color::srgb(0.3, 0.6, 0.3);
pub const SECONDARY_COLOR: Color = Color::srgb(0.9, 0.5, 0.2);
pub const TEXT_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);
pub const HIGHLIGHT_COLOR: Color = Color::srgb(1.0, 0.9, 0.0);

// ボタン色（コントラスト改善）
pub const BUTTON_NORMAL: Color = Color::srgb(0.4, 0.7, 0.4);
pub const BUTTON_HOVER: Color = Color::srgb(0.5, 0.8, 0.5);
pub const BUTTON_PRESSED: Color = Color::srgb(0.3, 0.5, 0.3);

// フォントサイズ微調整
pub const FONT_SIZE_HUGE: f32 = 72.0;
pub const FONT_SIZE_LARGE: f32 = 48.0;
pub const FONT_SIZE_MEDIUM: f32 = 32.0;
pub const FONT_SIZE_SMALL: f32 = 24.0;

// マージン・パディング
pub const SPACING_SMALL: f32 = 10.0;
pub const SPACING_MEDIUM: f32 = 20.0;
pub const SPACING_LARGE: f32 = 50.0;
```

**調整項目**:
- スコア表示の位置とサイズ
- 次のフルーツプレビューのサイズ
- コンボカウンターの位置と表示時間
- ボタンのパディングとマージン
- 境界線の太さと色
- アニメーションの速度とイージング

**関連ドキュメント**:
- [04_ui_ux.md](../04_ui_ux.md)

---

### タスク 10.4: パフォーマンス最適化

**優先度**: P1
**推定工数**: 2時間
**ラベル**: task, phase-10, optimization

**説明**:
プロファイリングを行い、ボトルネックを特定して最適化する。

**受け入れ基準**:
- [ ] 60fps以上を安定して維持できる
- [ ] フルーツ数が多い場合でもフレームレートが安定している
- [ ] パーティクル数が適切に制限されている
- [ ] メモリリークがない
- [ ] 無駄なシステム実行がない（Changed<T>フィルタの活用）

**実装ガイド**:
```rust
// パーティクル数の制限
const MAX_PARTICLES: usize = 200;

pub fn spawn_particles_limited(
    commands: &mut Commands,
    position: Vec2,
    color: Color,
    count: u32,
    particle_count: &mut ResMut<ParticleCount>,
) {
    let available = (MAX_PARTICLES - particle_count.0).min(count as usize);

    for _ in 0..available {
        // パーティクル生成
        particle_count.0 += 1;
    }
}

pub fn cleanup_dead_particles(
    mut particle_count: ResMut<ParticleCount>,
) {
    // Updateシステムでdespawnされたパーティクルの数を減算
    // （実装はUpdateシステムで管理）
}

// Changed<T>フィルタの活用例
pub fn update_score_text_optimized(
    game_state: Res<GameState>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    // game_state.is_changed()でのみ実行
    if !game_state.is_changed() {
        return;
    }

    for mut text in query.iter_mut() {
        **text = format!("スコア: {}", format_number(game_state.score));
    }
}

// Bevyの診断ツールを使用
// FrameTimeDiagnosticsPluginを追加してFPS監視
.add_plugins(FrameTimeDiagnosticsPlugin)
.add_plugins(LogDiagnosticsPlugin::default())
```

**最適化チェックリスト**:
- [ ] Changed<T>フィルタを使用してシステム実行を制限
- [ ] パーティクル数を制限（最大200個程度）
- [ ] 不要なクエリを削減
- [ ] イベントのread()を必ず使用（複数回のiterは避ける）
- [ ] 重い処理を分散（毎フレーム実行しない）

**関連ドキュメント**:
- [10_advanced_topics.md - セクション2](../10_advanced_topics.md)

---

### タスク 10.5: 経過時間システムの実装

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-10, features

**説明**:
ゲーム中の経過時間を計測し、GameStateに記録する。

**受け入れ基準**:
- [ ] `update_elapsed_time` システムが実装されている
- [ ] Playing状態でのみ時間が進む
- [ ] Paused状態では時間が止まる
- [ ] GameStateのelapsed_timeが更新される
- [ ] UI（タイマー）が正しく表示される

**実装ガイド**:
```rust
// app/core/src/systems/game_state.rs
use bevy::prelude::*;
use crate::*;

pub fn update_elapsed_time(
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
) {
    game_state.elapsed_time += time.delta_seconds();
}

// main.rsに追加
.add_systems(Update, (
    game_state::update_elapsed_time,
).run_if(in_state(AppState::Playing)))
```

**関連ドキュメント**:
- [02_architecture.md - セクション3](../02_architecture.md)

---

### タスク 10.6: バグ修正とエッジケース処理

**優先度**: P0
**推定工数**: 2時間
**ラベル**: task, phase-10, bugfix

**説明**:
既知のバグを修正し、エッジケースを適切に処理する。

**受け入れ基準**:
- [ ] 重複despawn問題が完全に解決されている
- [ ] 境界線外にフルーツが飛び出す問題が解決されている
- [ ] ゲームオーバー後の状態遷移が正しい
- [ ] 高速連続合体時のクラッシュがない
- [ ] 音が重複再生されすぎない
- [ ] UIの重なり問題がない

**実装ガイド**:
```rust
// 境界線外のフルーツを削除
pub fn cleanup_out_of_bounds_fruits(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Fruit>>,
) {
    const MAX_Y: f32 = 1000.0;  // 箱の上部より十分上
    const MIN_Y: f32 = -1000.0;  // 箱の下部より十分下

    for (entity, transform) in query.iter() {
        if transform.translation.y > MAX_Y || transform.translation.y < MIN_Y {
            warn!("Fruit out of bounds, despawning: {:?}", entity);
            commands.entity(entity).despawn();
        }
    }
}

// 音の重複再生防止
#[derive(Resource, Default)]
pub struct SfxCooldowns {
    pub merge: f32,
    pub combo: f32,
}

pub fn update_sfx_cooldowns(
    mut cooldowns: ResMut<SfxCooldowns>,
    time: Res<Time>,
) {
    cooldowns.merge = (cooldowns.merge - time.delta_seconds()).max(0.0);
    cooldowns.combo = (cooldowns.combo - time.delta_seconds()).max(0.0);
}

pub fn play_merge_sfx_with_cooldown(
    mut merge_events: EventReader<FruitMergeEvent>,
    audio: Res<Audio>,
    sfx_handles: Res<SfxHandles>,
    mut cooldowns: ResMut<SfxCooldowns>,
) {
    const COOLDOWN: f32 = 0.05;  // 50ms

    for event in merge_events.read() {
        if cooldowns.merge > 0.0 {
            continue;  // クールダウン中
        }

        // 効果音再生
        // ...

        cooldowns.merge = COOLDOWN;
    }
}
```

**バグチェックリスト**:
- [ ] すべての状態遷移が正しい
- [ ] リソースが適切に初期化される
- [ ] エンティティが適切にdespawnされる
- [ ] イベントが重複処理されない
- [ ] エラーハンドリングが適切
- [ ] エッジケースが考慮されている

**関連ドキュメント**:
- [10_advanced_topics.md - セクション8](../10_advanced_topics.md)

---

### タスク 10.7: プレイテストとフィードバック反映

**優先度**: P0
**推定工数**: 2時間
**ラベル**: task, phase-10, testing

**説明**:
実際にゲームをプレイし、問題点を洗い出してフィードバックを反映する。

**受け入れ基準**:
- [ ] 最低10回以上のプレイテストを実施
- [ ] 問題点がリスト化されている
- [ ] 主要な問題が修正されている
- [ ] ゲームバランスが適切である
- [ ] 致命的なバグがない

**プレイテストチェックリスト**:
- [ ] ゲームが起動する
- [ ] タイトル画面が表示される
- [ ] スタートボタンが機能する
- [ ] フルーツを落とせる
- [ ] フルーツが合体する
- [ ] スコアが加算される
- [ ] コンボが機能する
- [ ] ゲームオーバーになる
- [ ] ハイスコアが保存される
- [ ] リトライできる
- [ ] タイトルに戻れる
- [ ] ESCでポーズできる
- [ ] BGMと効果音が再生される
- [ ] 長時間プレイしても安定している

**フィードバック項目例**:
- 操作性（フルーツを落とす操作が快適か）
- ゲームバランス（難しすぎる/簡単すぎる）
- 視覚的な魅力（エフェクトが十分か）
- 音響体験（BGMと効果音のバランス）
- UI/UX（わかりやすいか、見やすいか）
- バグ（クラッシュ、意図しない挙動）

**関連ドキュメント**:
- [06_implementation_plan.md - セクション11](../06_implementation_plan.md)

---

## フェーズ検証

### 検証項目

- [ ] すべてのタスクが完了している
- [ ] `cargo build --workspace -- -D warnings` が成功する
- [ ] `cargo test --workspace` が成功する
- [ ] `cargo clippy --workspace -- -D warnings` が成功する
- [ ] 物理パラメータが適切に調整されている
- [ ] ゲームバランスが適切である
- [ ] UIが読みやすく使いやすい
- [ ] 60fps以上を安定して維持できる
- [ ] 致命的なバグがない
- [ ] プレイテストで問題が発見されていない
- [ ] オリジナルのスイカゲームと同等の遊び心地である

### 検証手順

```bash
# ビルドチェック（警告をエラーとして扱う）
cargo build --workspace --release -- -D warnings

# テスト実行
cargo test --workspace

# Clippy チェック（警告をエラーとして扱う）
cargo clippy --workspace -- -D warnings

# フォーマットチェック
cargo fmt --all -- --check

# リリースビルドで実行
cargo run --release

# プレイテスト（複数回）
# - 10回以上プレイして問題を確認
# - スコアが適切に推移するか
# - ゲームバランスが適切か
# - バグがないか
# - パフォーマンスが安定しているか

# パフォーマンス計測
# - FPS表示を有効にして確認
# - フルーツが大量にある状態でもテスト
```

## 完了条件

- [ ] すべてのタスクが完了している
- [ ] すべての検証項目が合格している
- [ ] プレイテストで重大な問題が発見されていない
- [ ] ドキュメントが最新の状態に更新されている
- [ ] コードがクリーンで保守可能な状態である
- [ ] パフォーマンスが目標（60fps以上）を達成している
- [ ] ゲームが楽しくプレイできる

## 次のフェーズ

Phase 10完了 → 次は **Phase 11: ピクセルアート統合（後日）** に進む

Phase 10完了時点で、ゲームは完成状態となる。Phase 11はオプションであり、プレースホルダーグラフィックを自作のピクセルアートに置き換えるフェーズである。

## 備考

- Phase 10は最も重要なフェーズの一つ（ゲームの完成度を決める）
- 調整は主観的な部分が多いため、複数人でテストすることが望ましい
- パラメータは定数として定義し、簡単に調整できるようにする
- プレイテストは時間をかけて丁寧に行う
- 発見した問題は必ず記録し、優先順位をつけて修正する
- パフォーマンス問題はプロファイリングツールを活用して特定
- Phase 10完了後、一度コードレビューを実施することを推奨

**調整のヒント**:
- 重力は落下速度に直結（-800.0で遅め、-1200.0で速め）
- 反発係数は0.2〜0.4が自然（高いと跳ねすぎる）
- コンボウィンドウは1.5〜3.0秒が適切（短すぎるとコンボが発生しにくい）
- 得点は2倍ずつ増えるのが一般的（10, 20, 40, 80...）
- UI要素は画面の20%以内に抑える（ゲームエリアを圧迫しない）

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 未着手
