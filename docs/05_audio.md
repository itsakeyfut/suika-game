# スイカゲーム - オーディオ設計書

## 1. オーディオシステム概要

### 1.1 使用ライブラリ
- **bevy_kira_audio**: Bevyと統合されたオーディオプレイバック
  - バージョン: 0.22（Bevy 0.18互換）
  - 特徴:
    - 高品質なオーディオ再生
    - ループ再生対応
    - 音量・ピッチ調整
    - フェードイン/フェードアウト
    - 複数チャンネル対応

### 1.2 オーディオファイル形式
- **BGM**: OGG形式（ループに最適、ファイルサイズ小）
- **効果音**: WAV形式（低遅延、短い音に最適）
- **サンプルレート**: 44.1kHz
- **ビットレート**:
  - BGM: 128-192 kbps
  - 効果音: 16-bit

## 2. BGM（背景音楽）システム

### 2.1 BGMトラック一覧

| トラック名 | 用途 | 長さ | テンポ | 雰囲気 |
|-----------|------|------|-------|-------|
| title_bgm.ogg | タイトル画面 | 1-2分（ループ） | 120 BPM | 軽快、楽しい |
| game_bgm.ogg | ゲームプレイ中 | 2-3分（ループ） | 100 BPM | 集中できる、穏やか |
| gameover_bgm.ogg | ゲームオーバー | 10-15秒（ワンショット） | 80 BPM | 悲しい、諦め |

### 2.2 BGMの要件

#### タイトルBGM
- **雰囲気**: 明るく、プレイ意欲をかき立てる
- **楽器**: シンセ、ピコピコ音、軽いパーカッション
- **ループ**: シームレスにループ
- **音量**: 中程度（効果音を邪魔しない）

#### ゲームBGM
- **雰囲気**: 集中を維持できるアンビエント、適度なリズム
- **楽器**: シンセパッド、柔らかいピアノ、軽いドラム
- **ループ**: シームレスにループ
- **音量**: 低〜中程度（ゲームプレイの邪魔にならない）
- **変化**: 後半に向けてわずかに盛り上がる（テンション維持）

#### ゲームオーバーBGM
- **雰囲気**: 軽い失望感、しかし前向きさも残す
- **楽器**: ピアノ、ストリングス
- **長さ**: 10-15秒（ワンショット再生）
- **音量**: 中程度

### 2.3 BGM実装

#### 2.3.1 アセットの読み込み

```rust
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

#[derive(Resource)]
pub struct BgmHandles {
    pub title: Handle<AudioSource>,
    pub game: Handle<AudioSource>,
    pub gameover: Handle<AudioSource>,
}

fn load_bgm(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(BgmHandles {
        title: asset_server.load("sounds/bgm/title_bgm.ogg"),
        game: asset_server.load("sounds/bgm/game_bgm.ogg"),
        gameover: asset_server.load("sounds/bgm/gameover_bgm.ogg"),
    });
}
```

#### 2.3.2 BGMの再生・停止

```rust
fn play_title_bgm(
    audio: Res<Audio>,
    bgm_handles: Res<BgmHandles>,
) {
    audio.play(bgm_handles.title.clone())
        .looped()
        .with_volume(0.6);
}

fn stop_bgm(
    audio: Res<Audio>,
) {
    audio.stop();
}
```

#### 2.3.3 BGM切り替えシステム

```rust
#[derive(Resource)]
pub struct CurrentBgm {
    pub track: BgmTrack,
}

#[derive(PartialEq, Eq)]
pub enum BgmTrack {
    None,
    Title,
    Game,
    GameOver,
}

fn switch_bgm_on_state_change(
    mut commands: Commands,
    current_state: Res<State<AppState>>,
    mut current_bgm: ResMut<CurrentBgm>,
    audio: Res<Audio>,
    bgm_handles: Res<BgmHandles>,
) {
    let desired_track = match current_state.get() {
        AppState::Title => BgmTrack::Title,
        AppState::Playing => BgmTrack::Game,
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
                    .fade_in(AudioTween::new(
                        Duration::from_secs_f32(1.0),
                        AudioEasing::Linear,
                    ));
            }
            BgmTrack::Game => {
                audio.play(bgm_handles.game.clone())
                    .looped()
                    .with_volume(0.4)
                    .fade_in(AudioTween::new(
                        Duration::from_secs_f32(1.5),
                        AudioEasing::Linear,
                    ));
            }
            BgmTrack::GameOver => {
                audio.play(bgm_handles.gameover.clone())
                    .with_volume(0.5);  // ループなし
            }
            BgmTrack::None => {}
        }

        current_bgm.track = desired_track;
    }
}
```

## 3. 効果音（SFX）システム

### 3.1 効果音一覧

| ファイル名 | トリガー | 長さ | 音量 | ピッチ調整 |
|-----------|---------|------|------|----------|
| drop.wav | フルーツを落とした時 | 0.1秒 | 中 | なし |
| merge_small.wav | 小さいフルーツの合体 | 0.2秒 | 中 | あり（サイズに応じて） |
| merge_medium.wav | 中サイズフルーツの合体 | 0.3秒 | 中〜大 | あり |
| merge_large.wav | 大きいフルーツの合体 | 0.4秒 | 大 | あり |
| watermelon.wav | スイカ完成 | 1.0秒 | 大 | なし（ファンファーレ） |
| combo.wav | コンボ発生 | 0.3秒 | 中 | あり（コンボ数に応じて） |
| gameover.wav | ゲームオーバー | 1.5秒 | 大 | なし |
| button_click.wav | ボタンクリック | 0.1秒 | 小 | なし |
| button_hover.wav | ボタンホバー | 0.05秒 | 小 | なし |
| warning.wav | 境界線超過警告 | 0.3秒 | 中 | なし（ループ可能） |

### 3.2 効果音の要件

#### フルーツ落下音（drop.wav）
- **音の特徴**: 軽い「ポトッ」という音
- **ピッチ**: 固定
- **タイミング**: フルーツがスポーンし、プレイヤーが落下ボタンを押した瞬間

#### 合体音（merge_*.wav）
- **音の特徴**: ポップな「ポンッ」という音、フルーツが大きいほど低音
- **ピッチ調整**:
  - サクランボ〜ブドウ: 高め（1.2〜1.0）
  - デコポン〜梨: 中程度（1.0〜0.8）
  - 桃〜メロン: 低め（0.8〜0.6）
- **音量**: フルーツが大きいほど大きい
- **タイミング**: 2つのフルーツが接触して合体が始まる瞬間

#### スイカ完成音（watermelon.wav）
- **音の特徴**: 華やかなファンファーレ、達成感のある音
- **長さ**: 1秒程度
- **ピッチ**: 固定
- **タイミング**: 2つのメロンが合体してスイカになった瞬間

#### コンボ音（combo.wav）
- **音の特徴**: 軽快な「ピロリン♪」という音
- **ピッチ調整**: コンボ数が増えるほど高くなる（1.0 + combo * 0.1）
- **タイミング**: 短時間に連続で合体が発生した時

#### ゲームオーバー音（gameover.wav）
- **音の特徴**: 下降音階、「アウー」という感じ
- **長さ**: 1.5秒
- **タイミング**: ゲームオーバー判定時

#### UI効果音
- **button_click.wav**: 短い「カチッ」
- **button_hover.wav**: 非常に短い「ピッ」
- **タイミング**: ボタンのインタラクション時

### 3.3 効果音実装

#### 3.3.1 アセットの読み込み

```rust
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
    pub warning: Handle<AudioSource>,
}

fn load_sfx(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
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
        warning: asset_server.load("sounds/sfx/warning.wav"),
    });
}
```

#### 3.3.2 フルーツ合体音の再生

```rust
fn play_merge_sfx(
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
            FruitType::Peach | FruitType::Pineapple | FruitType::Melon => {
                (sfx_handles.merge_large.clone(), 0.8)
            }
            FruitType::Watermelon => {
                // スイカは特別な音
                audio.play(sfx_handles.watermelon.clone())
                    .with_volume(0.8);
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

#### 3.3.3 コンボ音の再生

```rust
fn play_combo_sfx(
    mut combo_events: EventReader<ComboEvent>,
    audio: Res<Audio>,
    sfx_handles: Res<SfxHandles>,
) {
    for event in combo_events.read() {
        // コンボ数に応じてピッチを上げる
        let pitch = 1.0 + (event.combo_count as f32 * 0.1).min(0.5);

        audio.play(sfx_handles.combo.clone())
            .with_volume(0.6)
            .with_playback_rate(pitch);
    }
}
```

#### 3.3.4 UI効果音の再生

```rust
fn play_ui_sfx(
    mut interaction_query: Query<
        (&Interaction, &MenuButton),
        Changed<Interaction>,
    >,
    audio: Res<Audio>,
    sfx_handles: Res<SfxHandles>,
) {
    for (interaction, _button) in interaction_query.iter() {
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

## 4. 音量管理

### 4.1 音量バランス

| 音源カテゴリ | 基本音量 | 調整範囲 |
|------------|---------|---------|
| BGM | 40-60% | 0-100% |
| 効果音（重要） | 70-80% | 0-100% |
| 効果音（UI） | 30-50% | 0-100% |

### 4.2 音量設定システム（オプション）

```rust
#[derive(Resource)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub bgm_volume: f32,
    pub sfx_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            bgm_volume: 0.6,
            sfx_volume: 0.7,
        }
    }
}

fn apply_volume_settings(
    audio: Res<Audio>,
    settings: Res<AudioSettings>,
) {
    // bevy_kira_audioのグローバル音量設定
    // （実装方法はバージョンに依存）
}
```

## 5. オーディオチャンネル管理

### 5.1 チャンネル分離

```rust
// 異なるチャンネルでBGMと効果音を管理
#[derive(Resource)]
pub struct AudioChannels {
    pub bgm_channel: AudioChannel,
    pub sfx_channel: AudioChannel,
}

fn setup_audio_channels(mut commands: Commands) {
    commands.insert_resource(AudioChannels {
        bgm_channel: AudioChannel::new("bgm".to_owned()),
        sfx_channel: AudioChannel::new("sfx".to_owned()),
    });
}
```

### 5.2 チャンネル別音量制御

```rust
fn play_bgm_on_channel(
    audio: Res<Audio>,
    channels: Res<AudioChannels>,
    bgm_handles: Res<BgmHandles>,
) {
    audio.play_in_channel(
        bgm_handles.game.clone(),
        &channels.bgm_channel,
    ).looped().with_volume(0.4);
}
```

## 6. パフォーマンス最適化

### 6.1 同時再生数の制限
- 効果音の最大同時再生数: 8-10
- 同じ効果音の連続再生間隔: 最低50ms

### 6.2 プリロード
- ゲーム開始時に全てのオーディオアセットをロード
- 再生遅延を最小限に

### 6.3 メモリ管理
- 長いBGMはストリーミング再生
- 短い効果音はメモリに常駐

## 7. プレースホルダーオーディオ

### 7.1 開発初期のプレースホルダー
実際のオーディオアセット作成前は以下を使用：
- **BGM**: フリー素材サイトからCC0ライセンスの音楽
  - [FreePD](https://freepd.com/)
  - [CC0 Music](https://www.youtube.com/c/CC0Music)
- **効果音**: 簡易的な合成音または効果音ジェネレーター
  - [SFXR](http://www.drpetter.se/project_sfxr.html)
  - [ChipTone](https://sfbgames.itch.io/chiptone)

### 7.2 最終アセットへの置き換え
- ファイル名を同じにすることでコード変更なしに置き換え可能
- 音量・ピッチは最終アセットに合わせて微調整

## 8. デバッグとテスト

### 8.1 オーディオデバッグUI

```rust
#[cfg(debug_assertions)]
fn debug_audio_ui(
    mut commands: Commands,
    audio_settings: Res<AudioSettings>,
) {
    // 画面に音量スライダーを表示
    // BGM/SFX個別の音量調整
    // ミュートボタン
}
```

### 8.2 テストチェックリスト
- [ ] 全てのBGMが正しくループする
- [ ] 画面遷移時にBGMがスムーズに切り替わる
- [ ] 効果音が適切なタイミングで再生される
- [ ] 音量バランスが適切（BGMが効果音を邪魔しない）
- [ ] 同時に複数の効果音が再生されても音割れしない
- [ ] 長時間プレイでメモリリークしない

## 9. 将来的な拡張

### 9.1 サウンドオプション画面
- マスター音量調整
- BGM音量調整
- 効果音音量調整
- ミュートボタン

### 9.2 動的BGM
- ゲームの緊張度に応じてBGMの層を追加/削除
- スコアに応じてテンポを変化

### 9.3 空間オーディオ（2D）
- フルーツの位置に応じてパンニング
- より没入感のあるサウンド

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 承認済み
