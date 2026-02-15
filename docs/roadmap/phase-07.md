# Phase 7: 基本UIの実装

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 4-6時間
**完了日**: -
**依存関係**: Phase 6

### 目的
タイトル画面、ゲーム中のHUD、ゲームオーバー画面、ポーズメニューを実装する。

### スコープ
- タイトル画面の実装（スタートボタン、ハイスコア表示）
- ゲーム中HUDの実装（スコア、タイマー、次のフルーツ、コンボ）
- ゲームオーバー画面の実装（最終スコア、新記録通知、リトライボタン）
- ポーズメニューの実装（ESCキーでトグル）
- ボタンインタラクションシステム
- UIアニメーション（基本的なもの）

## 前提条件

- Phase 6が完了している
- ゲームオーバー判定が正常に動作している
- AppState（Title, Playing, Paused, GameOver）が定義されている

## タスクリスト

### タスク 7.1: UIカラーパレットと定数の定義

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-7, ui

**説明**:
UI全体で使用するカラーパレット、フォントサイズ、スタイル定数を定義する。

**受け入れ基準**:
- [ ] `app/ui/src/styles.rs` が作成されている
- [ ] カラーパレットが定義されている（BG, PRIMARY, SECONDARY, TEXT, HIGHLIGHT）
- [ ] ボタン色が定義されている（NORMAL, HOVER, PRESSED）
- [ ] フォントサイズが定義されている（HUGE, LARGE, MEDIUM, SMALL）
- [ ] 各定数にドキュメントコメントがある

**実装ガイド**:
```rust
// app/ui/src/styles.rs
use bevy::prelude::*;

// 基本色
pub const BG_COLOR: Color = Color::srgb(0.95, 0.95, 0.90);        // 薄いベージュ
pub const PRIMARY_COLOR: Color = Color::srgb(0.3, 0.6, 0.3);     // 緑
pub const SECONDARY_COLOR: Color = Color::srgb(0.9, 0.5, 0.2);   // オレンジ
pub const TEXT_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);        // ダークグレー
pub const HIGHLIGHT_COLOR: Color = Color::srgb(1.0, 0.9, 0.0);   // 黄色

// ボタン色
pub const BUTTON_NORMAL: Color = Color::srgb(0.4, 0.7, 0.4);     // 明るい緑
pub const BUTTON_HOVER: Color = Color::srgb(0.5, 0.8, 0.5);      // より明るい緑
pub const BUTTON_PRESSED: Color = Color::srgb(0.3, 0.5, 0.3);    // 暗い緑

// フォントサイズ
pub const FONT_SIZE_HUGE: f32 = 72.0;      // タイトル
pub const FONT_SIZE_LARGE: f32 = 48.0;     // スコア
pub const FONT_SIZE_MEDIUM: f32 = 32.0;    // ボタン、ラベル
pub const FONT_SIZE_SMALL: f32 = 24.0;     // 補助情報

// ボタンサイズ
pub const BUTTON_LARGE_WIDTH: f32 = 240.0;
pub const BUTTON_LARGE_HEIGHT: f32 = 80.0;
pub const BUTTON_MEDIUM_WIDTH: f32 = 200.0;
pub const BUTTON_MEDIUM_HEIGHT: f32 = 60.0;
```

**関連ドキュメント**:
- [04_ui_ux.md - セクション1.2](../04_ui_ux.md)

---

### タスク 7.2: 共通UIコンポーネントの実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-7, ui

**説明**:
ボタン、テキストなどの再利用可能なUIコンポーネントを実装する。

**受け入れ基準**:
- [ ] `app/ui/src/components.rs` が作成されている
- [ ] `MenuButton` コンポーネントが定義されている
- [ ] `ButtonAction` enum が定義されている（StartGame, RetryGame, GoToTitle等）
- [ ] ボタン生成ヘルパー関数が実装されている
- [ ] テキスト生成ヘルパー関数が実装されている

**実装ガイド**:
```rust
// app/ui/src/components.rs
use bevy::prelude::*;
use crate::styles::*;

#[derive(Component)]
pub struct MenuButton {
    pub action: ButtonAction,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonAction {
    StartGame,
    RetryGame,
    GoToTitle,
    ResumeGame,
}

// ボタン生成ヘルパー
pub fn spawn_button(
    parent: &mut ChildBuilder,
    text: &str,
    action: ButtonAction,
    font_size: f32,
    width: f32,
    height: f32,
) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(width),
            height: Val::Px(height),
            margin: UiRect::all(Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(BUTTON_NORMAL),
        MenuButton { action },
    )).with_children(|parent| {
        parent.spawn((
            Text::new(text),
            TextFont {
                font_size,
                ..default()
            },
            TextColor(TEXT_COLOR),
        ));
    });
}

// テキスト生成ヘルパー
pub fn spawn_text(
    parent: &mut ChildBuilder,
    text: &str,
    font_size: f32,
    color: Color,
) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
    ));
}
```

**関連ドキュメント**:
- [04_ui_ux.md - セクション3.1](../04_ui_ux.md)

---

### タスク 7.3: タイトル画面の実装

**優先度**: P0
**推定工数**: 1.5時間
**ラベル**: task, phase-7, ui

**説明**:
ゲームタイトル、スタートボタン、ハイスコア表示を含むタイトル画面を実装する。

**受け入れ基準**:
- [ ] `app/ui/src/screens/title.rs` が作成されている
- [ ] `setup_title_screen` システムが実装されている
- [ ] ゲームタイトルが中央上部に表示される
- [ ] スタートボタンが中央に配置される
- [ ] ハイスコアが下部に表示される
- [ ] StateScoped(AppState::Title) が設定されている（自動削除）

**実装ガイド**:
```rust
// app/ui/src/screens/title.rs
use bevy::prelude::*;
use suika_game_core::*;
use crate::{components::*, styles::*};

pub fn setup_title_screen(
    mut commands: Commands,
    game_state: Res<GameState>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(BG_COLOR),
        StateScoped(AppState::Title),
    )).with_children(|parent| {
        // ゲームタイトル
        parent.spawn((
            Text::new("スイカゲーム"),
            TextFont {
                font_size: FONT_SIZE_HUGE,
                ..default()
            },
            TextColor(PRIMARY_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(100.0)),
                ..default()
            },
        ));

        // スタートボタン
        spawn_button(
            parent,
            "スタート",
            ButtonAction::StartGame,
            FONT_SIZE_LARGE,
            BUTTON_LARGE_WIDTH,
            BUTTON_LARGE_HEIGHT,
        );

        // ハイスコア表示
        parent.spawn((
            Text::new(format!("ハイスコア: {}", format_number(game_state.highscore))),
            TextFont {
                font_size: FONT_SIZE_SMALL,
                ..default()
            },
            TextColor(TEXT_COLOR),
            Node {
                margin: UiRect::top(Val::Px(150.0)),
                ..default()
            },
        ));
    });
}

fn format_number(n: u32) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
```

**関連ドキュメント**:
- [04_ui_ux.md - セクション2.1](../04_ui_ux.md)

---

### タスク 7.4: ゲーム中HUDの実装

**優先度**: P0
**推定工数**: 2時間
**ラベル**: task, phase-7, ui

**説明**:
スコア、タイマー、次のフルーツプレビュー、コンボカウンターを含むHUDを実装する。

**受け入れ基準**:
- [ ] `app/ui/src/screens/hud.rs` が作成されている
- [ ] `setup_hud` システムが実装されている
- [ ] スコア表示が中央上部に配置される
- [ ] タイマーとハイスコアが右上に配置される
- [ ] 次のフルーツプレビューが左上に配置される
- [ ] コンボカウンターが右下に配置される
- [ ] 各要素にマーカーコンポーネントが設定されている
- [ ] `update_*` システムがリソース変更時に表示を更新する

**実装ガイド**:
```rust
// app/ui/src/screens/hud.rs
use bevy::prelude::*;
use suika_game_core::*;
use crate::styles::*;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct TimerText;

#[derive(Component)]
pub struct HighscoreText;

#[derive(Component)]
pub struct ComboText;

#[derive(Component)]
pub struct NextFruitPreview;

pub fn setup_hud(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        StateScoped(AppState::Playing),
    )).with_children(|parent| {
        // スコア表示（中央上部）
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Px(20.0),
                ..default()
            },
            ScoreText,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("スコア: 0"),
                TextFont {
                    font_size: FONT_SIZE_LARGE,
                    ..default()
                },
                TextColor(PRIMARY_COLOR),
            ));
        });

        // 右上コンテナ（タイマー、ハイスコア）
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        )).with_children(|parent| {
            // タイマー
            parent.spawn((
                Text::new("時間: 0:00"),
                TextFont {
                    font_size: FONT_SIZE_SMALL,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                TimerText,
            ));

            // ハイスコア
            parent.spawn((
                Text::new("ハイ: 0"),
                TextFont {
                    font_size: FONT_SIZE_SMALL,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                HighscoreText,
            ));
        });

        // 次のフルーツプレビュー（左上）
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(20.0),
                width: Val::Px(100.0),
                height: Val::Px(120.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("次"),
                TextFont {
                    font_size: FONT_SIZE_SMALL,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            // プレビュー用のスプライト（実装は次のフェーズで詳細化）
            parent.spawn((
                Node {
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(1.0, 0.0, 0.0)),  // プレースホルダー
                NextFruitPreview,
            ));
        });

        // コンボカウンター（右下、初期は非表示）
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                bottom: Val::Px(20.0),
                ..default()
            },
            Visibility::Hidden,
            ComboText,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("コンボ ×2!"),
                TextFont {
                    font_size: FONT_SIZE_MEDIUM,
                    ..default()
                },
                TextColor(HIGHLIGHT_COLOR),
            ));
        });
    });
}

pub fn update_score_text(
    game_state: Res<GameState>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    if game_state.is_changed() {
        for mut text in query.iter_mut() {
            **text = format!("スコア: {}", format_number(game_state.score));
        }
    }
}

pub fn update_timer_text(
    game_state: Res<GameState>,
    mut query: Query<&mut Text, With<TimerText>>,
) {
    if game_state.is_changed() {
        let minutes = (game_state.elapsed_time as u32) / 60;
        let seconds = (game_state.elapsed_time as u32) % 60;

        for mut text in query.iter_mut() {
            **text = format!("時間: {}:{:02}", minutes, seconds);
        }
    }
}

pub fn update_combo_text(
    combo_timer: Res<ComboTimer>,
    mut query: Query<(&mut Text, &mut Visibility), With<ComboText>>,
) {
    if combo_timer.is_changed() {
        for (mut text, mut visibility) in query.iter_mut() {
            if combo_timer.current_combo >= 2 {
                **text = format!("コンボ ×{}!", combo_timer.current_combo);
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn format_number(n: u32) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
```

**関連ドキュメント**:
- [04_ui_ux.md - セクション2.2](../04_ui_ux.md)

---

### タスク 7.5: ゲームオーバー画面の実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-7, ui

**説明**:
最終スコア、新記録通知、リトライボタン、タイトルに戻るボタンを含むゲームオーバー画面を実装する。

**受け入れ基準**:
- [ ] `app/ui/src/screens/game_over.rs` が作成されている
- [ ] `setup_game_over_screen` システムが実装されている
- [ ] "GAME OVER" テキストが表示される
- [ ] 最終スコアが表示される
- [ ] 新記録の場合、祝福メッセージが表示される
- [ ] もう一度ボタンが配置される
- [ ] タイトルへボタンが配置される

**実装ガイド**:
```rust
// app/ui/src/screens/game_over.rs
use bevy::prelude::*;
use suika_game_core::*;
use crate::{components::*, styles::*};

pub fn setup_game_over_screen(
    mut commands: Commands,
    game_state: Res<GameState>,
) {
    let is_new_record = game_state.score >= game_state.highscore && game_state.score > 0;

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(BG_COLOR),
        StateScoped(AppState::GameOver),
    )).with_children(|parent| {
        // "GAME OVER" テキスト
        parent.spawn((
            Text::new("GAME OVER"),
            TextFont {
                font_size: FONT_SIZE_HUGE,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.2, 0.2)),
            Node {
                margin: UiRect::bottom(Val::Px(50.0)),
                ..default()
            },
        ));

        // 最終スコア
        parent.spawn((
            Text::new(format!("あなたのスコア: {}", format_number(game_state.score))),
            TextFont {
                font_size: FONT_SIZE_LARGE,
                ..default()
            },
            TextColor(TEXT_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));

        // 新記録通知
        if is_new_record {
            parent.spawn((
                Text::new("🎉 新記録! 🎉"),
                TextFont {
                    font_size: FONT_SIZE_MEDIUM + 4.0,
                    ..default()
                },
                TextColor(HIGHLIGHT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));
        }

        // ハイスコア
        parent.spawn((
            Text::new(format!("ハイスコア: {}", format_number(game_state.highscore))),
            TextFont {
                font_size: FONT_SIZE_MEDIUM,
                ..default()
            },
            TextColor(TEXT_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));

        // もう一度ボタン
        spawn_button(
            parent,
            "もう一度",
            ButtonAction::RetryGame,
            FONT_SIZE_LARGE,
            BUTTON_LARGE_WIDTH,
            BUTTON_LARGE_HEIGHT,
        );

        // タイトルへボタン
        spawn_button(
            parent,
            "タイトルへ",
            ButtonAction::GoToTitle,
            FONT_SIZE_MEDIUM,
            BUTTON_MEDIUM_WIDTH,
            BUTTON_MEDIUM_HEIGHT,
        );
    });
}

fn format_number(n: u32) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
```

**関連ドキュメント**:
- [04_ui_ux.md - セクション2.4](../04_ui_ux.md)

---

### タスク 7.6: ポーズメニューの実装

**優先度**: P1
**推定工数**: 1時間
**ラベル**: task, phase-7, ui

**説明**:
ESCキーでトグルできるポーズメニューを実装する。

**受け入れ基準**:
- [ ] `app/ui/src/screens/pause.rs` が作成されている
- [ ] `setup_pause_menu` システムが実装されている
- [ ] `handle_pause_input` システムが実装されている
- [ ] ESCキーでPlaying ⇔ Pausedを切り替えられる
- [ ] ゲームに戻るボタンが機能する
- [ ] タイトルに戻るボタンが機能する
- [ ] 半透明の黒オーバーレイが表示される

**実装ガイド**:
```rust
// app/ui/src/screens/pause.rs
use bevy::prelude::*;
use suika_game_core::*;
use crate::{components::*, styles::*};

pub fn setup_pause_menu(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        StateScoped(AppState::Paused),
    )).with_children(|parent| {
        // "ポーズ中" テキスト
        parent.spawn((
            Text::new("ポーズ中"),
            TextFont {
                font_size: FONT_SIZE_LARGE,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(50.0)),
                ..default()
            },
        ));

        // ゲームに戻るボタン
        spawn_button(
            parent,
            "ゲームに戻る",
            ButtonAction::ResumeGame,
            FONT_SIZE_MEDIUM,
            300.0,
            60.0,
        );

        // タイトルに戻るボタン
        spawn_button(
            parent,
            "タイトルに戻る",
            ButtonAction::GoToTitle,
            FONT_SIZE_MEDIUM,
            300.0,
            60.0,
        );
    });
}

pub fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            AppState::Playing => {
                next_state.set(AppState::Paused);
            }
            AppState::Paused => {
                next_state.set(AppState::Playing);
            }
            _ => {}
        }
    }
}
```

**関連ドキュメント**:
- [04_ui_ux.md - セクション2.3](../04_ui_ux.md)

---

### タスク 7.7: ボタンインタラクションシステムの実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-7, ui

**説明**:
ボタンのホバー、クリック処理とアクション実行を実装する。

**受け入れ基準**:
- [ ] `app/ui/src/systems/button.rs` が作成されている
- [ ] `button_interaction_system` が実装されている
- [ ] ホバー時にボタンの色が変わる
- [ ] クリック時にボタンが押された状態になる
- [ ] 各ButtonActionが適切に処理される（状態遷移）
- [ ] ホバー時に軽くスケールアップする（オプション）

**実装ガイド**:
```rust
// app/ui/src/systems/button.rs
use bevy::prelude::*;
use suika_game_core::*;
use crate::{components::*, styles::*};

pub fn button_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &MenuButton),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut color, button) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRESSED.into();

                // アクション実行
                match button.action {
                    ButtonAction::StartGame => {
                        next_state.set(AppState::Playing);
                    }
                    ButtonAction::RetryGame => {
                        next_state.set(AppState::Playing);
                    }
                    ButtonAction::GoToTitle => {
                        next_state.set(AppState::Title);
                    }
                    ButtonAction::ResumeGame => {
                        next_state.set(AppState::Playing);
                    }
                }
            }
            Interaction::Hovered => {
                *color = BUTTON_HOVER.into();
            }
            Interaction::None => {
                *color = BUTTON_NORMAL.into();
            }
        }
    }
}

// ボタンホバーアニメーション（オプション）
pub fn button_hover_animation(
    mut query: Query<(&Interaction, &mut Transform), With<MenuButton>>,
) {
    for (interaction, mut transform) in query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                transform.scale = Vec3::splat(1.05);
            }
            _ => {
                transform.scale = Vec3::splat(1.0);
            }
        }
    }
}
```

**関連ドキュメント**:
- [04_ui_ux.md - セクション3.1](../04_ui_ux.md)

---

### タスク 7.8: システムの統合とプラグイン化

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-7, integration

**説明**:
Phase 7で実装したすべてのUIシステムをUIプラグインとして統合する。

**受け入れ基準**:
- [ ] `app/ui/src/lib.rs` が更新されている
- [ ] `UiPlugin` が実装されている
- [ ] すべてのシステムが適切なスケジュールに登録されている
- [ ] main.rsでプラグインが追加されている
- [ ] `cargo run` でUIが正しく表示される

**実装ガイド**:
```rust
// app/ui/src/lib.rs
mod components;
mod screens;
mod styles;
mod systems;

pub use components::*;
pub use styles::*;

use bevy::prelude::*;
use suika_game_core::AppState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // タイトル画面
            .add_systems(OnEnter(AppState::Title), screens::title::setup_title_screen)

            // ゲーム中HUD
            .add_systems(OnEnter(AppState::Playing), screens::hud::setup_hud)
            .add_systems(Update, (
                screens::hud::update_score_text,
                screens::hud::update_timer_text,
                screens::hud::update_combo_text,
            ).run_if(in_state(AppState::Playing)))

            // ゲームオーバー画面
            .add_systems(OnEnter(AppState::GameOver), screens::game_over::setup_game_over_screen)

            // ポーズメニュー
            .add_systems(OnEnter(AppState::Paused), screens::pause::setup_pause_menu)
            .add_systems(Update, screens::pause::handle_pause_input)

            // ボタンインタラクション
            .add_systems(Update, (
                systems::button::button_interaction_system,
                systems::button::button_hover_animation,
            ));
    }
}

// main.rsに追加
.add_plugins(suika_game_ui::UiPlugin)
```

**関連ドキュメント**:
- [08_crate_architecture.md](../08_crate_architecture.md)

---

## フェーズ検証

### 検証項目

- [ ] すべてのタスクが完了している
- [ ] `cargo build --workspace` が成功する
- [ ] `cargo run` でゲームが起動する
- [ ] タイトル画面が表示される
- [ ] スタートボタンをクリックするとゲームが開始される
- [ ] ゲーム中にスコア、タイマー、次のフルーツが表示される
- [ ] コンボが発生すると右下にコンボカウンターが表示される
- [ ] ESCキーでポーズメニューが開く
- [ ] ゲームオーバー時にゲームオーバー画面が表示される
- [ ] 新記録の場合、祝福メッセージが表示される
- [ ] リトライボタンでゲームが再開できる
- [ ] タイトルに戻るボタンでタイトル画面に戻れる
- [ ] ボタンホバー時に色が変わる

### 検証手順

```bash
# ゲーム実行
cargo run

# 確認項目:
# 1. タイトル画面が表示される
# 2. スタートボタンをクリックしてゲーム開始
# 3. HUDが正しく表示される（スコア、タイマー等）
# 4. フルーツを合体させるとスコアが更新される
# 5. 連続合体でコンボカウンターが表示される
# 6. ESCキーでポーズメニューが開く
# 7. ゲームに戻るボタンで復帰できる
# 8. ゲームオーバーになるとゲームオーバー画面が表示される
# 9. もう一度ボタンでゲームが再開できる
# 10. ボタンのホバー/クリックエフェクトが動作する
```

## 完了条件

- [ ] すべてのタスクが完了している
- [ ] すべての検証項目が合格している
- [ ] ドキュメントが更新されている（必要な場合）
- [ ] コードがフォーマットされている（`just fmt`）
- [ ] Clippyチェックが通っている（`just clippy`）

## 次のフェーズ

Phase 7完了 → 次は **Phase 8: リッチなビジュアルエフェクト** に進む

## 備考

- Phase 7完了時点で、ゲームが完全にプレイ可能になる
- 次のフルーツプレビューは現在プレースホルダー（単色の四角）
- Phase 8でパーティクルや画面シェイクを追加してより魅力的にする
- フォントは現在デフォルトフォントを使用（後でピクセルフォントに置き換え可能）
- UIレイアウトは後で微調整可能（Phase 10で最適化）
- 経過時間の更新システムはまだ実装されていない場合、追加が必要

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 未着手
