# スイカゲーム - 高度なトピック

## 1. 開発ツール：ホットリロード

### 1.1 概要

Bevyは**標準でアセットのホットリロード機能**を提供しています。開発中にアセットファイル（画像、音声、テキストなど）を編集・保存すると、ゲームを再起動せずに即座に変更が反映されます。

**対応アセットタイプ**:
- ✅ 画像ファイル（PNG, JPG, WebP, etc.）
- ✅ 音声ファイル（OGG, WAV, MP3, FLAC）
- ✅ テキストファイル
- ✅ カスタムアセット（RON, JSON, etc.）

**動作条件**:
- `debug`ビルドで自動有効化（`cargo run`）
- `release`ビルドでは自動無効化（`cargo run --release`）
- `assets/`ディレクトリ内のファイル変更を監視

### 1.2 基本的な使い方

#### セットアップ不要

Bevyの`DefaultPlugins`を使用するだけで、ホットリロードが自動的に有効になります。

```rust
// app/suika-game/src/main.rs
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // ホットリロード自動有効化！
        .add_plugins(GameCorePlugin)
        .add_plugins(GameUIPlugin)
        .add_plugins(GameAudioPlugin)
        .add_plugins(GameAssetsPlugin)
        .run();
}
```

#### アセットの読み込み

通常通り`AssetServer`でアセットを読み込むだけで、ホットリロードに対応します。

```rust
// app/assets/src/lib.rs
use bevy::prelude::*;
use suika_game_core::fruit::FruitType;
use std::collections::HashMap;

#[derive(Resource)]
pub struct FruitAssets {
    pub sprites: HashMap<FruitType, Handle<Image>>,
}

fn load_fruit_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut sprites = HashMap::new();

    // 画像を読み込み（ファイル変更時に自動リロード）
    sprites.insert(FruitType::Cherry, asset_server.load("sprites/cherry.png"));
    sprites.insert(FruitType::Strawberry, asset_server.load("sprites/strawberry.png"));
    sprites.insert(FruitType::Grape, asset_server.load("sprites/grape.png"));
    // ... 他のフルーツ

    commands.insert_resource(FruitAssets { sprites });
    info!("✅ Fruit sprites loaded with hot-reload enabled");
}
```

### 1.3 動作の仕組み

1. **ファイル監視**: Bevyが内部で`notify`クレートを使用してファイルシステムを監視
2. **変更検出**: `assets/`ディレクトリ内のファイル変更を検出
3. **自動リロード**: 変更されたアセットを再読み込み
4. **ハンドル更新**: `Handle<T>`が参照する内容を自動更新
5. **画面反映**: 次のフレームで新しいアセットが自動的に表示される

```
ファイル保存
    ↓
[notify] ファイル変更検出
    ↓
[AssetServer] アセット再読み込み
    ↓
[Handle<Image>] 内容を自動更新
    ↓
画面に即座に反映！
```

### 1.4 実践例

#### 例1: スプライト編集のワークフロー

1. ゲームを起動（`cargo run`）
2. 画像編集ソフトで`assets/sprites/cherry.png`を開く
3. スプライトを編集して保存
4. **ゲーム画面で即座に変更が反映される！**

#### 例2: 音声ファイルの差し替え

1. ゲーム実行中
2. `assets/sounds/sfx/merge.ogg`を別のファイルで上書き
3. 次にマージ効果音が再生されるとき、新しい音声が使われる

#### 例3: 開発中の調整

```rust
// フルーツのサイズ調整用の定数ファイル
// assets/config/fruit_sizes.ron
(
    cherry: 20.0,
    strawberry: 25.0,
    grape: 30.0,
    // ...
)
```

```rust
// カスタムアセットローダーを実装すれば、
// RONファイルもホットリロード対応可能
#[derive(Asset, TypePath, Deserialize)]
pub struct FruitSizeConfig {
    pub cherry: f32,
    pub strawberry: f32,
    // ...
}

fn load_config(
    asset_server: Res<AssetServer>,
) {
    let config: Handle<FruitSizeConfig> = asset_server.load("config/fruit_sizes.ron");
    // ファイル変更時に自動リロード
}
```

### 1.5 読み込み状態の確認（オプション）

アセットの読み込み状態やリロード状態を監視することも可能です。

```rust
fn monitor_asset_reload(
    asset_server: Res<AssetServer>,
    fruit_assets: Res<FruitAssets>,
) {
    for (fruit_type, handle) in &fruit_assets.sprites {
        match asset_server.get_load_state(handle) {
            Some(LoadState::Loaded) => {
                // 読み込み完了（初回またはリロード完了）
                debug!("{:?} sprite loaded", fruit_type);
            }
            Some(LoadState::Loading) => {
                // 読み込み中（リロード中）
                debug!("{:?} sprite loading...", fruit_type);
            }
            Some(LoadState::Failed(err)) => {
                // 読み込み失敗
                error!("Failed to load {:?}: {:?}", fruit_type, err);
            }
            None => {}
        }
    }
}
```

### 1.6 ホットリロードの制御

#### 開発ビルドで無効化

```rust
use bevy::asset::AssetPlugin;

App::new()
    .add_plugins(DefaultPlugins.set(AssetPlugin {
        watch_for_changes_override: Some(false), // 強制無効化
        ..default()
    }))
```

#### リリースビルド

リリースビルドでは自動的に無効化されます。

```bash
# ホットリロード無効（パフォーマンス最適化）
cargo run --release
```

### 1.7 注意事項

#### ファイル保存のタイミング

一部のエディタ（例: Photoshop）は、保存時に一時ファイルを作成してからリネームします。この場合、ホットリロードが2回トリガーされる可能性がありますが、Bevyは内部でデバウンス処理を行っているため、通常は問題ありません。

#### 大量のアセット変更

多数のファイルを一度に変更すると、リロードが連続で発生し、一時的にフレームレートが低下する可能性があります。

#### プラットフォーム依存性

- **Windows**: 正常動作
- **macOS**: 正常動作
- **Linux**: 正常動作（一部ファイルシステムで設定が必要な場合あり）
- **WASM**: ホットリロード非対応（ブラウザ環境のため）

### 1.8 トラブルシューティング

#### ホットリロードが動作しない場合

1. **debugビルドか確認**
   ```bash
   # OK: ホットリロード有効
   cargo run

   # NG: ホットリロード無効
   cargo run --release
   ```

2. **ファイルパスの確認**
   ```rust
   // OK: assets/からの相対パス
   asset_server.load("sprites/cherry.png")

   // NG: 絶対パス（ホットリロード未対応）
   asset_server.load("/full/path/to/cherry.png")
   ```

3. **ファイル監視上限（Linux）**
   ```bash
   # inotifyの監視上限を増やす
   sudo sysctl fs.inotify.max_user_watches=524288
   ```

### 1.9 開発ワークフロー例

#### グラフィックアーティストとの協業

1. アーティストが`assets/sprites/`にスプライトを配置
2. ゲームを起動したまま、アーティストがスプライトを編集
3. 保存するとゲーム内で即座に確認可能
4. 調整を繰り返し、最適なビジュアルを決定

#### サウンドデザイナーとの協業

1. `assets/sounds/`に音声ファイルを配置
2. ゲーム実行中に効果音を差し替え
3. ゲーム内で即座に新しい音を確認
4. 音量やピッチの調整を繰り返す

### 1.10 まとめ

✅ **Bevyのホットリロードは設定不要で使える**
✅ **開発効率が大幅に向上する**
✅ **画像・音声・テキスト全てに対応**
✅ **チームでの協業にも最適**

スイカゲームの開発では、この機能をフル活用してアセットの調整を効率的に行いましょう。

---

## 2. WASM（WebAssembly）対応

### 2.1 概要

Bevyゲームは Wasm にコンパイルしてブラウザで実行できます。これにより、インストール不要でブラウザから直接プレイ可能なゲームを配信できます。

### 2.2 ビルド手順

#### 前提条件

```bash
# wasmターゲットの追加
rustup target add wasm32-unknown-unknown

# wasm-bindgen-cliのインストール
cargo install wasm-bindgen-cli

# （オプション）ローカルサーバー用
cargo install basic-http-server
```

#### ビルドコマンド

```bash
# Wasmビルド
cargo build --release --target wasm32-unknown-unknown

# wasm-bindgenで JavaScriptバインディング生成
wasm-bindgen --out-dir ./web \
  --target web \
  ./target/wasm32-unknown-unknown/release/suika-game.wasm

# アセットをwebディレクトリにコピー
cp -r assets web/
```

#### 最適化

```toml
# Cargo.toml - Wasm用の最適化設定
[profile.wasm-release]
inherits = "release"
opt-level = "z"          # サイズ最適化
lto = true               # リンク時最適化
codegen-units = 1        # 単一コードユニット
strip = true             # シンボル削除
panic = "abort"          # パニック時アボート
```

```bash
# 最適化ビルド
cargo build --profile wasm-release --target wasm32-unknown-unknown
```

### 2.3 HTMLテンプレート

```html
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>スイカゲーム - Suika Game</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        }
        canvas {
            outline: none;
            border: 2px solid white;
            border-radius: 8px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.3);
        }
    </style>
</head>
<body>
    <script type="module">
        import init from './suika-game.js';
        init();
    </script>
</body>
</html>
```

### 2.4 Wasm固有の注意点

#### ファイルシステムアクセス

```rust
// ハイスコアの保存は localStorage を使用
#[cfg(target_arch = "wasm32")]
fn save_highscore(score: u32) {
    use web_sys::window;
    if let Some(storage) = window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("suika_highscore", &score.to_string());
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_highscore(score: u32) {
    // ファイルに保存
    std::fs::write("save/highscore.txt", score.to_string()).ok();
}
```

#### アセットパス

```rust
// Wasmではアセットパスが異なる可能性がある
#[cfg(target_arch = "wasm32")]
const ASSET_PREFIX: &str = "assets/";

#[cfg(not(target_arch = "wasm32"))]
const ASSET_PREFIX: &str = "assets/";
```

#### パフォーマンス考慮事項

- **物理演算**: Wasmは native より遅いため、物理パラメータの調整が必要な場合がある
- **フレームレート**: 60fps を目標とするが、低スペック環境では30fpsにフォールバック
- **メモリ**: Wasmのメモリ制限（通常2GB）に注意

### 2.5 デプロイ

#### GitHub Pages

```bash
# gh-pagesブランチにデプロイ
git checkout --orphan gh-pages
git add web/*
git commit -m "Deploy to GitHub Pages"
git push origin gh-pages
```

#### Itch.io

1. `web/` ディレクトリをZIP圧縮
2. Itch.ioにアップロード
3. 「This file will be played in the browser」を選択

---

## 3. アクセシビリティ機能

### 3.1 カラーブラインドモード

フルーツを色だけでなく、模様やアイコンでも区別可能にする。

```rust
#[derive(Resource)]
pub struct AccessibilitySettings {
    pub colorblind_mode: ColorblindMode,
    pub high_contrast: bool,
    pub font_scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorblindMode {
    Normal,
    Protanopia,   // 赤色弱
    Deuteranopia, // 緑色弱
    Tritanopia,   // 青色弱
}

// フルーツスプライトに模様を追加
fn apply_colorblind_mode(
    mode: ColorblindMode,
    fruit_type: FruitType,
) -> PatternOverlay {
    match mode {
        ColorblindMode::Normal => PatternOverlay::None,
        _ => match fruit_type {
            FruitType::Cherry => PatternOverlay::Dots,
            FruitType::Strawberry => PatternOverlay::Stripes,
            FruitType::Grape => PatternOverlay::Grid,
            // ...
        }
    }
}
```

### 3.2 音声フィードバック

視覚に頼らないゲームプレイのサポート。

```rust
// 重要なイベントで音声案内
fn announce_event(
    event: GameEvent,
    audio: &AudioChannel,
    tts: &TextToSpeech, // オプション: TTS統合
) {
    match event {
        GameEvent::FruitMerged(fruit_type) => {
            audio.play(merge_sound);
            #[cfg(feature = "tts")]
            tts.speak(format!("{:?} merged!", fruit_type));
        }
        GameEvent::GameOver => {
            audio.play(gameover_sound);
            #[cfg(feature = "tts")]
            tts.speak("Game Over");
        }
    }
}
```

### 3.3 キーボード操作の充実

マウス不要での完全操作。

```rust
fn keyboard_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut spawn_position: ResMut<NextFruitPosition>,
) {
    // 左右移動
    if keyboard.pressed(KeyCode::ArrowLeft) {
        spawn_position.x -= 2.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        spawn_position.x += 2.0;
    }

    // 高速移動（Shiftキー）
    let speed_multiplier = if keyboard.pressed(KeyCode::ShiftLeft) {
        5.0
    } else {
        1.0
    };

    // スペースキーまたはEnterでドロップ
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter) {
        // フルーツをドロップ
    }

    // ESCでポーズ
    if keyboard.just_pressed(KeyCode::Escape) {
        // ポーズメニュー
    }

    // Tabでフォーカス移動（UI要素）
    if keyboard.just_pressed(KeyCode::Tab) {
        // 次のUI要素にフォーカス
    }
}
```

### 3.4 フォントサイズ調整

```rust
#[derive(Resource)]
pub struct FontSettings {
    pub scale: f32, // 1.0 = 通常, 1.5 = 150%, etc.
}

fn apply_font_scale(
    settings: Res<FontSettings>,
    mut text_query: Query<&mut TextFont>,
) {
    for mut font in text_query.iter_mut() {
        font.font_size = BASE_FONT_SIZE * settings.scale;
    }
}
```

### 3.5 UI コントラスト設定

```rust
fn apply_high_contrast(
    settings: Res<AccessibilitySettings>,
    mut background_query: Query<&mut BackgroundColor, With<UiElement>>,
) {
    if settings.high_contrast {
        for mut bg in background_query.iter_mut() {
            bg.0 = Color::srgb(0.0, 0.0, 0.0); // 黒背景
        }
    }
}
```

---

## 4. パーティクルプーリングシステム

### 4.1 概要

パーティクルの生成と削除を繰り返すとメモリアロケーションが頻発します。プーリングシステムで再利用することでパフォーマンスを向上させます。

### 4.2 実装

```rust
#[derive(Resource)]
pub struct ParticlePool {
    pub available: Vec<Entity>,
    pub max_size: usize,
    pub total_created: usize,
}

impl Default for ParticlePool {
    fn default() -> Self {
        Self {
            available: Vec::with_capacity(100),
            max_size: 100,
            total_created: 0,
        }
    }
}

// パーティクルをプールから取得またはスポーン
fn spawn_particle_from_pool(
    pool: &mut ParticlePool,
    commands: &mut Commands,
    position: Vec2,
) -> Entity {
    if let Some(entity) = pool.available.pop() {
        // プールから再利用
        entity
    } else if pool.total_created < pool.max_size {
        // 新規作成
        pool.total_created += 1;
        commands.spawn((
            Particle::default(),
            Transform::from_translation(position.extend(0.0)),
            Visibility::default(),
        )).id()
    } else {
        // プールが満杯の場合は最古のパーティクルを再利用
        warn!("Particle pool exhausted");
        pool.available.first().copied().unwrap_or(Entity::PLACEHOLDER)
    }
}

// パーティクルの寿命が切れたらプールに返却
fn return_particles_to_pool(
    mut pool: ResMut<ParticlePool>,
    mut commands: Commands,
    particle_query: Query<(Entity, &Particle)>,
) {
    for (entity, particle) in particle_query.iter() {
        if particle.current_time >= particle.lifetime {
            // 非表示にしてプールに返却
            commands.entity(entity).insert(Visibility::Hidden);
            pool.available.push(entity);
        }
    }
}

// パーティクルを再アクティブ化
fn activate_particle(
    commands: &mut Commands,
    entity: Entity,
    position: Vec2,
    velocity: Vec2,
    color: Color,
) {
    commands.entity(entity).insert((
        Transform::from_translation(position.extend(0.0)),
        Particle {
            velocity,
            color,
            lifetime: 1.0,
            current_time: 0.0,
        },
        Visibility::Visible,
    ));
}
```

### 4.3 パーティクルシステムの統合

```rust
fn spawn_merge_particles(
    mut commands: Commands,
    mut pool: ResMut<ParticlePool>,
    mut merge_events: EventReader<FruitMergeEvent>,
) {
    for event in merge_events.read() {
        // 合体位置から放射状にパーティクルを生成
        for i in 0..20 {
            let angle = (i as f32 / 20.0) * std::f32::consts::TAU;
            let velocity = Vec2::new(
                angle.cos() * 100.0,
                angle.sin() * 100.0,
            );

            let entity = spawn_particle_from_pool(
                &mut pool,
                &mut commands,
                event.position,
            );

            activate_particle(
                &mut commands,
                entity,
                event.position,
                velocity,
                Color::srgb(1.0, 1.0, 0.0),
            );
        }
    }
}
```

---

## 5. アセットローディング状態追跡

### 5.1 ローディング状態管理

```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AssetLoadState {
    #[default]
    Loading,
    Ready,
    Failed,
}

#[derive(Resource)]
pub struct AssetsLoading {
    pub total: usize,
    pub loaded: usize,
    pub failed: Vec<String>,
}

impl AssetsLoading {
    pub fn progress(&self) -> f32 {
        if self.total == 0 {
            return 1.0;
        }
        self.loaded as f32 / self.total as f32
    }
}
```

### 5.2 アセット読み込みの追跡

```rust
fn track_asset_loading(
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
    mut next_state: ResMut<NextState<AssetLoadState>>,
    fruit_assets: Res<FruitAssets>,
    audio_assets: Res<AudioAssets>,
) {
    let mut loaded = 0;
    let mut failed = Vec::new();

    // フルーツスプライトの確認
    for (fruit_type, handle) in &fruit_assets.sprites {
        match asset_server.get_load_state(handle) {
            Some(LoadState::Loaded) => loaded += 1,
            Some(LoadState::Failed(_)) => {
                failed.push(format!("Sprite: {:?}", fruit_type));
            }
            _ => {}
        }
    }

    // オーディオアセットの確認
    for handle in [
        &audio_assets.bgm_title,
        &audio_assets.bgm_game,
        &audio_assets.sfx_merge,
        // ...
    ] {
        match asset_server.get_load_state(handle) {
            Some(LoadState::Loaded) => loaded += 1,
            Some(LoadState::Failed(_)) => {
                failed.push("Audio asset".to_string());
            }
            _ => {}
        }
    }

    loading.loaded = loaded;
    loading.failed = failed;

    // 状態遷移
    if loaded == loading.total {
        next_state.set(AssetLoadState::Ready);
    } else if !loading.failed.is_empty() && loaded + loading.failed.len() == loading.total {
        error!("Failed to load assets: {:?}", loading.failed);
        next_state.set(AssetLoadState::Failed);
    }
}
```

### 5.3 ローディング画面UI

```rust
fn setup_loading_screen(
    mut commands: Commands,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        LoadingScreen,
    )).with_children(|parent| {
        // "Loading..." テキスト
        parent.spawn((
            Text::new("Loading..."),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));

        // プログレスバー
        parent.spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Px(20.0),
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
        )).with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
                ProgressBar,
            ));
        });
    });
}

fn update_loading_progress(
    loading: Res<AssetsLoading>,
    mut progress_query: Query<&mut Node, With<ProgressBar>>,
) {
    for mut style in progress_query.iter_mut() {
        style.width = Val::Percent(loading.progress() * 100.0);
    }
}
```

---

## 6. テストパターンと戦略

### 6.1 単体テスト

#### フルーツ進化ロジック

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fruit_evolution() {
        assert_eq!(
            FruitType::Cherry.next(),
            Some(FruitType::Strawberry)
        );
        assert_eq!(
            FruitType::Watermelon.next(),
            None
        );
    }

    #[test]
    fn test_fruit_points() {
        assert_eq!(FruitType::Cherry.points(), 10);
        assert_eq!(FruitType::Watermelon.points(), 500);
    }

    #[test]
    fn test_combo_bonus() {
        assert_eq!(calculate_combo_bonus(100, 0), 100);
        assert_eq!(calculate_combo_bonus(100, 2), 110);
        assert_eq!(calculate_combo_bonus(100, 3), 120);
        assert_eq!(calculate_combo_bonus(100, 5), 150);
    }
}
```

#### スコア計算ロジック

```rust
#[test]
fn test_score_calculation() {
    let mut state = GameState::default();

    // 基本得点
    add_score(&mut state, FruitType::Cherry, 0);
    assert_eq!(state.score, 10);

    // コンボボーナス
    add_score(&mut state, FruitType::Strawberry, 2);
    assert_eq!(state.score, 10 + 22); // 20 * 1.1
}
```

### 6.2 統合テスト

```rust
#[test]
fn test_fruit_merge_system() {
    let mut app = App::new();

    // 必要なプラグインを追加
    app.add_plugins(MinimalPlugins)
       .add_plugins(GameCorePlugin);

    // フルーツを2つスポーン
    let fruit1 = app.world.spawn((
        Fruit {
            fruit_type: FruitType::Cherry,
            points: 10,
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();

    let fruit2 = app.world.spawn((
        Fruit {
            fruit_type: FruitType::Cherry,
            points: 10,
        },
        Transform::from_xyz(1.0, 0.0, 0.0),
    )).id();

    // 合体イベントを発火
    app.world.send_event(FruitMergeEvent {
        entity1: fruit1,
        entity2: fruit2,
        fruit_type: FruitType::Cherry,
        next_fruit_type: FruitType::Strawberry,
        position: Vec2::new(0.5, 0.0),
        points: 10,
    });

    // システムを実行
    app.update();

    // 古いフルーツが削除されているか確認
    assert!(app.world.get_entity(fruit1).is_none());
    assert!(app.world.get_entity(fruit2).is_none());

    // 新しいフルーツが生成されているか確認
    let new_fruits: Vec<_> = app.world
        .query_filtered::<&Fruit, With<Fruit>>()
        .iter(&app.world)
        .collect();
    assert_eq!(new_fruits.len(), 1);
    assert_eq!(new_fruits[0].fruit_type, FruitType::Strawberry);
}
```

### 6.3 パフォーマンステスト

```rust
#[test]
fn test_many_fruits_performance() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(GameCorePlugin);

    // 100個のフルーツをスポーン
    for i in 0..100 {
        app.world.spawn((
            Fruit {
                fruit_type: FruitType::Cherry,
                points: 10,
            },
            Transform::from_xyz(
                (i % 10) as f32 * 10.0,
                (i / 10) as f32 * 10.0,
                0.0,
            ),
        ));
    }

    use std::time::Instant;
    let start = Instant::now();

    // 100フレーム実行
    for _ in 0..100 {
        app.update();
    }

    let duration = start.elapsed();
    let avg_frame_time = duration.as_secs_f64() / 100.0;

    // 60fps = 16.67ms/frame
    assert!(avg_frame_time < 0.0167,
        "Average frame time too high: {:.4}ms",
        avg_frame_time * 1000.0
    );
}
```

### 6.4 UI テスト

```rust
#[test]
fn test_button_interaction() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(GameUIPlugin);

    app.init_state::<AppState>();
    app.world.resource_mut::<NextState<AppState>>().set(AppState::Title);

    app.update();

    // スタートボタンを探す
    let button = app.world
        .query_filtered::<Entity, With<StartButton>>()
        .iter(&app.world)
        .next()
        .expect("Start button should exist");

    // ボタンクリックをシミュレート
    app.world.entity_mut(button).insert(Interaction::Pressed);
    app.update();

    // 状態がPlayingに遷移したか確認
    assert_eq!(
        app.world.resource::<State<AppState>>().get(),
        &AppState::Playing
    );
}
```

### 6.5 CI/CD統合

#### GitHub Actions例

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Check formatting
        run: cargo fmt -- --check

  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build
        run: cargo build --release
```

---

## 7. パフォーマンス最適化

### 7.1 プロファイリング

```bash
# flamegraphでプロファイリング
cargo install flamegraph
cargo flamegraph

# puffin を使ったフレーム単位のプロファイリング
# Cargo.toml に追加
# puffin = "0.18"
# puffin_egui = "0.25"
```

```rust
fn expensive_system() {
    puffin::profile_function!();

    puffin::profile_scope!("Inner operation");
    // 重い処理
}
```

### 7.2 最適化のヒント

1. **Queryフィルタ**: `With`, `Without` を積極的に使用
2. **Change Detection**: `Changed<T>` でコンポーネント変更時のみ処理
3. **並列化**: 依存関係のないシステムは自動並列実行
4. **Entity Recycling**: エンティティの使い回し（プーリング）
5. **Sprite Batching**: 同じテクスチャのスプライトをまとめて描画

```rust
// 悪い例: 全エンティティをチェック
fn update_positions(
    mut query: Query<&mut Transform>,
) {
    for mut transform in query.iter_mut() {
        // 全エンティティで実行
    }
}

// 良い例: フルーツだけをチェック
fn update_fruit_positions(
    mut query: Query<&mut Transform, (With<Fruit>, Changed<Transform>)>,
) {
    for mut transform in query.iter_mut() {
        // フルーツかつ変更があったもののみ
    }
}
```

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 承認済み
