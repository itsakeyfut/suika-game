# Phase 9: サウンド統合

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 3-4時間
**完了日**: -
**依存関係**: Phase 8

### 目的
BGMと効果音を統合し、ゲームに音響体験を追加する。

### スコープ
- bevy_kira_audioの統合
- BGMシステムの実装（タイトル、ゲーム中、ゲームオーバー）
- 効果音システムの実装（合体、スイカ完成、UI操作等）
- 音量管理システム
- プレースホルダーオーディオアセットの準備
- 状態に応じたBGM切り替え

## 前提条件

- Phase 8が完了している
- ビジュアルエフェクトが実装されている
- bevy_kira_audio依存関係がCargo.tomlに追加されている

## タスクリスト

### タスク 9.1: bevy_kira_audioの統合とセットアップ

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-9, audio

**説明**:
bevy_kira_audioプラグインをBevyアプリに統合し、オーディオ再生の基盤を構築する。

**受け入れ基準**:
- [ ] bevy_kira_audio = "0.24.0" がCargo.tomlに追加されている
- [ ] AudioPluginがAppに追加されている
- [ ] `cargo run` でオーディオシステムが初期化される
- [ ] ビルドエラーがない

**実装ガイド**:
```rust
// app/suika-game/Cargo.toml
[dependencies]
bevy = "0.17.3"
bevy_rapier2d = "0.32.0"
bevy_kira_audio = "0.24.0"
# ... 他の依存関係

// main.rs
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioPlugin)
        // ... 他のプラグイン
        .run();
}
```

**関連ドキュメント**:
- [05_audio.md - セクション1.1](../05_audio.md)

---

### タスク 9.2: オーディオアセットハンドルの定義

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-9, audio

**説明**:
BGMと効果音のハンドルを管理するリソースを定義する。

**受け入れ基準**:
- [ ] `app/audio/src/handles.rs` が作成されている
- [ ] `BgmHandles` リソースが定義されている
- [ ] `SfxHandles` リソースが定義されている
- [ ] すべての必要なオーディオファイルがハンドルとして定義されている

**実装ガイド**:
```rust
// app/audio/src/handles.rs
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;

#[derive(Resource)]
pub struct BgmHandles {
    pub title: Handle<AudioSource>,
    pub game: Handle<AudioSource>,
    pub gameover: Handle<AudioSource>,
}

#[derive(Resource)]
pub struct SfxHandles {
    pub drop: Handle<AudioSource>,
    pub merge_small: Handle<AudioSource>,
    pub merge_medium: Handle<AudioSource>,
    pub merge_large: Handle<AudioSource>,
    pub watermelon: Handle<AudioSource>,
    pub combo: Handle<AudioSource>,
    pub gameover: Handle<AudioSource>,
    pub button_click: Handle<AudioSource>,
    pub button_hover: Handle<AudioSource>,
}

pub fn load_audio_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(BgmHandles {
        title: asset_server.load("sounds/bgm/title_bgm.ogg"),
        game: asset_server.load("sounds/bgm/game_bgm.ogg"),
        gameover: asset_server.load("sounds/bgm/gameover_bgm.ogg"),
    });

    commands.insert_resource(SfxHandles {
        drop: asset_server.load("sounds/sfx/drop.wav"),
        merge_small: asset_server.load("sounds/sfx/merge_small.wav"),
        merge_medium: asset_server.load("sounds/sfx/merge_medium.wav"),
        merge_large: asset_server.load("sounds/sfx/merge_large.wav"),
        watermelon: asset_server.load("sounds/sfx/watermelon.wav"),
        combo: asset_server.load("sounds/sfx/combo.wav"),
        gameover: asset_server.load("sounds/sfx/gameover.wav"),
        button_click: asset_server.load("sounds/sfx/button_click.wav"),
        button_hover: asset_server.load("sounds/sfx/button_hover.wav"),
    });
}
```

**関連ドキュメント**:
- [05_audio.md - セクション2.3](../05_audio.md)

---

### タスク 9.3: BGM管理システムの実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-9, audio

**説明**:
AppStateに応じてBGMを切り替えるシステムを実装する。

**受け入れ基準**:
- [ ] `app/audio/src/bgm.rs` が作成されている
- [ ] `CurrentBgm` リソースが定義されている
- [ ] `switch_bgm_on_state_change` システムが実装されている
- [ ] 状態遷移時にBGMがスムーズに切り替わる
- [ ] フェードイン/フェードアウトが実装されている
- [ ] ループ再生が正しく設定されている

**実装ガイド**:
```rust
// app/audio/src/bgm.rs
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use suika_game_core::AppState;
use crate::handles::BgmHandles;

#[derive(Resource)]
pub struct CurrentBgm {
    pub track: BgmTrack,
}

impl Default for CurrentBgm {
    fn default() -> Self {
        Self {
            track: BgmTrack::None,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum BgmTrack {
    None,
    Title,
    Game,
    GameOver,
}

pub fn switch_bgm_on_state_change(
    current_state: Res<State<AppState>>,
    mut current_bgm: ResMut<CurrentBgm>,
    audio: Res<Audio>,
    bgm_handles: Res<BgmHandles>,
) {
    let desired_track = match current_state.get() {
        AppState::Title => BgmTrack::Title,
        AppState::Playing | AppState::Paused => BgmTrack::Game,
        AppState::GameOver => BgmTrack::GameOver,
    };

    if current_bgm.track != desired_track {
        // 現在のBGMを停止
        audio.stop();

        // 新しいBGMを再生
        match desired_track {
            BgmTrack::Title => {
                audio.play(bgm_handles.title.clone())
                    .looped()
                    .with_volume(0.6)
                    .fade_in(AudioTween::linear(std::time::Duration::from_secs_f32(1.0)));
            }
            BgmTrack::Game => {
                audio.play(bgm_handles.game.clone())
                    .looped()
                    .with_volume(0.4)
                    .fade_in(AudioTween::linear(std::time::Duration::from_secs_f32(1.5)));
            }
            BgmTrack::GameOver => {
                audio.play(bgm_handles.gameover.clone())
                    .with_volume(0.5);  // ループなし（ワンショット）
            }
            BgmTrack::None => {}
        }

        current_bgm.track = desired_track;
        info!("BGM switched to: {:?}", desired_track);
    }
}
```

**関連ドキュメント**:
- [05_audio.md - セクション2.3](../05_audio.md)

---

### タスク 9.4: 合体効果音システムの実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-9, audio

**説明**:
フルーツ合体時にサイズに応じた効果音を再生する。

**受け入れ基準**:
- [ ] `app/audio/src/sfx.rs` が作成されている
- [ ] `play_merge_sfx` システムが実装されている
- [ ] フルーツサイズに応じて異なる効果音が再生される
- [ ] ピッチ調整が実装されている（フルーツサイズに応じて）
- [ ] スイカ完成時に特別な効果音が再生される

**実装ガイド**:
```rust
// app/audio/src/sfx.rs
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use suika_game_core::{events::*, FruitType};
use crate::handles::SfxHandles;

pub fn play_merge_sfx(
    mut merge_events: EventReader<FruitMergeEvent>,
    audio: Res<Audio>,
    sfx_handles: Res<SfxHandles>,
) {
    for event in merge_events.read() {
        // フルーツサイズに応じた効果音を選択
        let (sfx, base_pitch) = match event.fruit_type {
            FruitType::Cherry | FruitType::Strawberry | FruitType::Grape => {
                (sfx_handles.merge_small.clone(), 1.2)
            }
            FruitType::Dekopon | FruitType::Persimmon | FruitType::Apple | FruitType::Pear => {
                (sfx_handles.merge_medium.clone(), 1.0)
            }
            FruitType::Peach | FruitType::Pineapple => {
                (sfx_handles.merge_large.clone(), 0.8)
            }
            FruitType::Melon => {
                // メロン合体 = スイカ誕生
                audio.play(sfx_handles.watermelon.clone())
                    .with_volume(0.8);
                return;
            }
            FruitType::Watermelon => {
                // スイカ同士の消滅
                audio.play(sfx_handles.merge_large.clone())
                    .with_volume(0.6)
                    .with_playback_rate(0.6);
                return;
            }
        };

        // ピッチを調整して再生
        audio.play(sfx.clone())
            .with_volume(0.7)
            .with_playback_rate(base_pitch);
    }
}
```

**関連ドキュメント**:
- [05_audio.md - セクション3.2](../05_audio.md)

---

### タスク 9.5: コンボ効果音とその他ゲームSFXの実装

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-9, audio

**説明**:
コンボ発生時、ゲームオーバー時の効果音を実装する。

**受け入れ基準**:
- [ ] `play_combo_sfx` システムが実装されている
- [ ] コンボ数に応じてピッチが変化する
- [ ] `play_gameover_sfx` システムが実装されている
- [ ] ゲームオーバー時に効果音が再生される

**実装ガイド**:
```rust
// app/audio/src/sfx.rs (続き)
pub fn play_combo_sfx(
    combo_timer: Res<ComboTimer>,
    audio: Res<Audio>,
    sfx_handles: Res<SfxHandles>,
) {
    if combo_timer.is_changed() && combo_timer.current_combo >= 2 {
        // コンボ数に応じてピッチを上げる（最大+50%）
        let pitch = 1.0 + (combo_timer.current_combo as f32 * 0.1).min(0.5);

        audio.play(sfx_handles.combo.clone())
            .with_volume(0.6)
            .with_playback_rate(pitch);
    }
}

pub fn play_gameover_sfx(
    audio: Res<Audio>,
    sfx_handles: Res<SfxHandles>,
) {
    audio.play(sfx_handles.gameover.clone())
        .with_volume(0.7);
}
```

**関連ドキュメント**:
- [05_audio.md - セクション3](../05_audio.md)

---

### タスク 9.6: UI効果音の実装

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-9, audio

**説明**:
ボタンのホバー、クリック時の効果音を実装する。

**受け入れ基準**:
- [ ] `play_ui_sfx` システムが実装されている
- [ ] ボタンホバー時に効果音が再生される
- [ ] ボタンクリック時に効果音が再生される
- [ ] 音量が適切（小さめ）である

**実装ガイド**:
```rust
// app/audio/src/sfx.rs (続き)
use suika_game_ui::MenuButton;

pub fn play_ui_sfx(
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<MenuButton>),
    >,
    audio: Res<Audio>,
    sfx_handles: Res<SfxHandles>,
) {
    for interaction in interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                audio.play(sfx_handles.button_click.clone())
                    .with_volume(0.5);
            }
            Interaction::Hovered => {
                audio.play(sfx_handles.button_hover.clone())
                    .with_volume(0.3);
            }
            _ => {}
        }
    }
}
```

**関連ドキュメント**:
- [05_audio.md - セクション3.3](../05_audio.md)

---

### タスク 9.7: プレースホルダーオーディオアセットの準備

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-9, assets

**説明**:
テスト用のプレースホルダーオーディオファイルを準備または生成する。

**受け入れ基準**:
- [ ] `assets/sounds/bgm/` ディレクトリが作成されている
- [ ] `assets/sounds/sfx/` ディレクトリが作成されている
- [ ] タイトルBGMのプレースホルダーがある（title_bgm.ogg）
- [ ] ゲームBGMのプレースホルダーがある（game_bgm.ogg）
- [ ] ゲームオーバーBGMのプレースホルダーがある（gameover_bgm.ogg）
- [ ] すべての効果音のプレースホルダーがある（.wav形式）
- [ ] 各ファイルが正しく読み込める

**実装ガイド**:
```bash
# ディレクトリ作成
mkdir -p assets/sounds/bgm
mkdir -p assets/sounds/sfx

# プレースホルダーファイルの準備
# オプション1: フリー素材サイトからダウンロード
# - FreePD (https://freepd.com/) - CC0ライセンス
# - Freesound (https://freesound.org/) - 検索してCC0を選択

# オプション2: 効果音ジェネレーターで生成
# - SFXR (http://www.drpetter.se/project_sfxr.html)
# - ChipTone (https://sfbgames.itch.io/chiptone)

# 必要なファイル（例）:
# BGM:
# - title_bgm.ogg (軽快な曲、1-2分ループ)
# - game_bgm.ogg (穏やかな曲、2-3分ループ)
# - gameover_bgm.ogg (悲しい曲、10-15秒ワンショット)

# SFX:
# - drop.wav (短い「ポトッ」)
# - merge_small.wav (高音の「ポン」)
# - merge_medium.wav (中音の「ポン」)
# - merge_large.wav (低音の「ボン」)
# - watermelon.wav (ファンファーレ、1秒)
# - combo.wav (軽快な「ピロリン」)
# - gameover.wav (下降音、1.5秒)
# - button_click.wav (「カチッ」)
# - button_hover.wav (「ピッ」)
```

**注意**:
- プレースホルダーは後で自作音源に置き換え可能
- ファイル名を同じにすればコード変更不要
- CC0またはパブリックドメインの音源を使用すること

**関連ドキュメント**:
- [05_audio.md - セクション7](../05_audio.md)

---

### タスク 9.8: AudioPluginの実装と統合

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-9, integration

**説明**:
すべてのオーディオシステムをAudioPluginとして統合し、main.rsに追加する。

**受け入れ基準**:
- [ ] `app/audio/src/lib.rs` が作成されている
- [ ] `AudioPlugin` が実装されている
- [ ] すべてのシステムが適切なスケジュールに登録されている
- [ ] main.rsでプラグインが追加されている
- [ ] `cargo run` でBGMと効果音が再生される

**実装ガイド**:
```rust
// app/audio/src/lib.rs
mod bgm;
mod handles;
mod sfx;

pub use bgm::*;
pub use handles::*;
pub use sfx::*;

use bevy::prelude::*;
use bevy_kira_audio::AudioPlugin as KiraAudioPlugin;
use suika_game_core::AppState;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(KiraAudioPlugin)
            .init_resource::<CurrentBgm>()
            .add_systems(Startup, load_audio_assets)

            // BGM管理
            .add_systems(Update, bgm::switch_bgm_on_state_change)

            // 効果音
            .add_systems(Update, (
                sfx::play_merge_sfx,
                sfx::play_combo_sfx,
                sfx::play_ui_sfx,
            ))
            .add_systems(OnEnter(AppState::GameOver), sfx::play_gameover_sfx);
    }
}

// main.rsに追加
.add_plugins(suika_game_audio::AudioPlugin)
```

**関連ドキュメント**:
- [08_crate_architecture.md](../08_crate_architecture.md)

---

## フェーズ検証

### 検証項目

- [ ] すべてのタスクが完了している
- [ ] `cargo build --workspace` が成功する
- [ ] `cargo run` でゲームが起動する
- [ ] タイトル画面でBGMが再生される
- [ ] ゲーム開始時にBGMがスムーズに切り替わる
- [ ] フルーツ合体時に効果音が再生される
- [ ] フルーツサイズに応じて効果音のピッチが変化する
- [ ] スイカ完成時に特別な効果音が再生される
- [ ] コンボ発生時にコンボ音が再生され、ピッチが上がる
- [ ] ボタンホバー/クリック時に効果音が再生される
- [ ] ゲームオーバー時に効果音とBGMが再生される
- [ ] ESCでポーズしてもBGMは再生され続ける

### 検証手順

```bash
# ゲーム実行
cargo run

# 確認項目:
# 1. タイトル画面でBGMが流れる
# 2. スタートボタンをクリックすると効果音が鳴る
# 3. ゲーム画面に切り替わるとBGMが変わる
# 4. フルーツを合体させると効果音が鳴る
# 5. 小さいフルーツは高音、大きいフルーツは低音
# 6. 連続合体でコンボ音が鳴り、ピッチが上がる
# 7. スイカを作るとファンファーレが鳴る
# 8. ゲームオーバーになるとゲームオーバー音とBGMが流れる
# 9. 音量バランスが適切（BGMが効果音を邪魔しない）
# 10. 音が途切れたりノイズが入ったりしない
```

## 完了条件

- [ ] すべてのタスクが完了している
- [ ] すべての検証項目が合格している
- [ ] ドキュメントが更新されている（必要な場合）
- [ ] コードがフォーマットされている（`just fmt`）
- [ ] Clippyチェックが通っている（`just clippy`）

## 次のフェーズ

Phase 9完了 → 次は **Phase 10: 調整とポリッシュ** に進む

## 備考

- Phase 9完了時点で、ゲームが完全な音響体験を持つ
- プレースホルダーオーディオは後で自作音源に置き換え可能
- bevy_kira_audioのバージョンはBevy 0.17.3互換のものを使用
- 音量バランスはPhase 10で微調整
- ポーズ中のBGM停止機能は将来の拡張として検討
- 音量設定UIは将来の拡張として検討（Phase 11以降）
- BGMのフェード時間は調整可能

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 未着手
