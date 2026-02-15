# Phase 2: ゲーム状態管理とリソース

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 2-3時間
**完了日**: -
**依存関係**: Phase 1

### 目的
ゲームのコアデータ構造を定義し、状態管理システムとハイスコア永続化機能を実装する。

### スコープ
- フルーツタイプの定義と各種パラメータ設定
- ゲーム状態管理リソースの実装
- ハイスコアの保存/読み込み機能
- アプリケーション状態（State）の定義
- コンポーネントと定数の定義

## 前提条件

- Phase 1が完了している
- Cargo.tomlに必要な依存関係が追加されている（serde, serde_json等）

## タスクリスト

### タスク 2.1: フルーツ型とパラメータ定義

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-2, core

**説明**:
FruitType enumと各フルーツのパラメータ（サイズ、質量、得点等）を定義する。

**受け入れ基準**:
- [ ] `app/core/src/fruit.rs` が作成されている
- [ ] `FruitType` enum に11種類のフルーツが定義されている
- [ ] `FruitParams` 構造体が定義されている（radius, mass, restitution, friction, points）
- [ ] `FruitType::next()` メソッドが実装されている（次の進化段階を返す）
- [ ] `FruitType::parameters()` メソッドが実装されている
- [ ] `FruitType::spawnable_fruits()` メソッドが実装されている（柿以下の5種類）
- [ ] `FruitType::placeholder_color()` メソッドが実装されている
- [ ] ユニットテストが追加されている

**実装ガイド**:
```rust
// app/core/src/fruit.rs
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum FruitType {
    Cherry,      // サクランボ (最小)
    Strawberry,  // イチゴ
    Grape,       // ブドウ
    Dekopon,     // デコポン
    Persimmon,   // 柿
    Apple,       // リンゴ
    Pear,        // 梨
    Peach,       // 桃
    Pineapple,   // パイナップル
    Melon,       // メロン
    Watermelon,  // スイカ (最大)
}

pub struct FruitParams {
    pub radius: f32,
    pub mass: f32,
    pub restitution: f32,
    pub friction: f32,
    pub points: u32,
}

impl FruitType {
    pub fn next(&self) -> Option<FruitType> {
        // スイカの次はNone（最大）
    }

    pub fn parameters(&self) -> FruitParams {
        // 各フルーツのパラメータを返す
    }

    pub fn spawnable_fruits() -> [FruitType; 5] {
        // サクランボ〜柿までの5種類
    }

    pub fn placeholder_color(&self) -> Color {
        // 各フルーツの色を返す
    }
}

#[cfg(test)]
mod tests {
    // FruitType::next() のテスト
    // spawnable_fruits() のテスト
}
```

**関連ドキュメント**:
- [01_specification.md - セクション2.1](../01_specification.md)

---

### タスク 2.2: ゲーム定数の定義

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-2, core

**説明**:
ゲーム全体で使用する定数（物理パラメータ、コンテナサイズ等）を定義する。

**受け入れ基準**:
- [ ] `app/core/src/constants.rs` が作成されている
- [ ] 物理定数が定義されている（GRAVITY, CONTAINER_WIDTH, CONTAINER_HEIGHT等）
- [ ] コンボシステム定数が定義されている（COMBO_WINDOW = 2.0）
- [ ] ゲームオーバー定数が定義されている（GAME_OVER_TIMER = 3.0）
- [ ] フルーツパラメータテーブルが定義されている

**実装ガイド**:
```rust
// app/core/src/constants.rs
use bevy::prelude::*;

// 物理パラメータ
pub const GRAVITY: f32 = -980.0;  // ピクセル/秒^2
pub const CONTAINER_WIDTH: f32 = 600.0;
pub const CONTAINER_HEIGHT: f32 = 800.0;
pub const WALL_THICKNESS: f32 = 20.0;
pub const BOUNDARY_LINE_Y: f32 = 300.0;

// コンボシステム
pub const COMBO_WINDOW: f32 = 2.0;  // 秒
pub const COMBO_MAX: u32 = 10;

// ゲームオーバー
pub const GAME_OVER_TIMER: f32 = 3.0;  // 秒

// フルーツパラメータ（サクランボ〜スイカ）
// 半径: 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0, 110.0, 120.0
// 得点: 10, 20, 40, 80, 160, 320, 640, 1280, 2560, 5120, 10240
```

**関連ドキュメント**:
- [01_specification.md - セクション2](../01_specification.md)
- [03_physics_rendering.md - セクション2](../03_physics_rendering.md)

---

### タスク 2.3: ゲーム状態リソースの実装

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-2, core

**説明**:
ゲームの状態を管理するリソースを実装する。

**受け入れ基準**:
- [ ] `app/core/src/resources.rs` が作成されている
- [ ] `GameState` リソースが定義されている
- [ ] `ComboTimer` リソースが定義されている
- [ ] `GameOverTimer` リソースが定義されている
- [ ] `NextFruitType` リソースが定義されている
- [ ] 各リソースにDefault実装がある

**実装ガイド**:
```rust
// app/core/src/resources.rs
use bevy::prelude::*;
use crate::fruit::FruitType;

#[derive(Resource, Default)]
pub struct GameState {
    pub score: u32,
    pub highscore: u32,
    pub elapsed_time: f32,
}

#[derive(Resource)]
pub struct ComboTimer {
    pub time_since_last_merge: f32,
    pub combo_window: f32,  // 2.0秒
    pub current_combo: u32,
}

impl Default for ComboTimer {
    fn default() -> Self {
        Self {
            time_since_last_merge: f32::MAX,
            combo_window: crate::constants::COMBO_WINDOW,
            current_combo: 1,
        }
    }
}

#[derive(Resource)]
pub struct GameOverTimer {
    pub time_over_boundary: f32,
    pub warning_threshold: f32,  // 3.0秒
    pub is_warning: bool,
}

impl Default for GameOverTimer {
    fn default() -> Self {
        Self {
            time_over_boundary: 0.0,
            warning_threshold: crate::constants::GAME_OVER_TIMER,
            is_warning: false,
        }
    }
}

#[derive(Resource)]
pub struct NextFruitType(pub FruitType);

impl Default for NextFruitType {
    fn default() -> Self {
        Self(FruitType::Cherry)
    }
}
```

**関連ドキュメント**:
- [02_architecture.md - セクション3](../02_architecture.md)

---

### タスク 2.4: ハイスコア永続化機能の実装

**優先度**: P1
**推定工数**: 1時間
**ラベル**: task, phase-2, core

**説明**:
ハイスコアをJSONファイルとして保存/読み込みする機能を実装する。

**受け入れ基準**:
- [ ] `app/core/src/persistence.rs` が作成されている
- [ ] `HighscoreData` 構造体が定義されている（serde対応）
- [ ] `save_highscore()` 関数が実装されている
- [ ] `load_highscore()` 関数が実装されている
- [ ] エラーハンドリングが適切に実装されている
- [ ] `save/` ディレクトリが自動作成される
- [ ] ユニットテストが追加されている

**実装ガイド**:
```rust
// app/core/src/persistence.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Default)]
pub struct HighscoreData {
    pub highscore: u32,
}

const SAVE_DIR: &str = "save";
const HIGHSCORE_FILE: &str = "save/highscore.json";

pub fn save_highscore(data: &HighscoreData) -> Result<(), Box<dyn std::error::Error>> {
    // save/ディレクトリ作成
    fs::create_dir_all(SAVE_DIR)?;

    // JSONシリアライズ
    let json = serde_json::to_string_pretty(data)?;

    // ファイル書き込み
    fs::write(HIGHSCORE_FILE, json)?;

    Ok(())
}

pub fn load_highscore() -> HighscoreData {
    if !Path::new(HIGHSCORE_FILE).exists() {
        return HighscoreData::default();
    }

    match fs::read_to_string(HIGHSCORE_FILE) {
        Ok(json) => {
            serde_json::from_str(&json).unwrap_or_default()
        }
        Err(_) => HighscoreData::default(),
    }
}

#[cfg(test)]
mod tests {
    // save/loadのテスト
}
```

**関連ドキュメント**:
- [02_architecture.md - セクション4](../02_architecture.md)

---

### タスク 2.5: アプリケーション状態の定義

**優先度**: P0
**推定工数**: 0.25時間
**ラベル**: task, phase-2, core

**説明**:
Bevyのステート機能を使用して、アプリケーション状態（タイトル、プレイ中、ポーズ、ゲームオーバー）を定義する。

**受け入れ基準**:
- [ ] `app/core/src/states.rs` が作成されている
- [ ] `AppState` enum が定義されている（Title, Playing, Paused, GameOver）
- [ ] `States` トレイトが実装されている
- [ ] デフォルトが Title になっている

**実装ガイド**:
```rust
// app/core/src/states.rs
use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,
    Playing,
    Paused,
    GameOver,
}
```

**関連ドキュメント**:
- [02_architecture.md - セクション2](../02_architecture.md)

---

### タスク 2.6: コンポーネントの定義

**優先度**: P0
**推定工数**: 0.25時間
**ラベル**: task, phase-2, core

**説明**:
ゲーム内で使用するコンポーネントを定義する。

**受け入れ基準**:
- [ ] `app/core/src/components.rs` が作成されている
- [ ] `Fruit` コンポーネントが定義されている
- [ ] `Container` マーカーコンポーネントが定義されている
- [ ] `BoundaryLine` マーカーコンポーネントが定義されている
- [ ] 必要に応じて他のコンポーネントも定義されている

**実装ガイド**:
```rust
// app/core/src/components.rs
use bevy::prelude::*;
use crate::fruit::FruitType;

#[derive(Component)]
pub struct Fruit {
    pub fruit_type: FruitType,
    pub points: u32,
}

#[derive(Component)]
pub struct Container;

#[derive(Component)]
pub struct BoundaryLine;

#[derive(Component)]
pub struct NextFruitPreview;
```

**関連ドキュメント**:
- [02_architecture.md - セクション2](../02_architecture.md)

---

### タスク 2.7: lib.rsとmod構成の整備

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-2, core

**説明**:
app/core/src/lib.rsを作成し、各モジュールをエクスポートする。

**受け入れ基準**:
- [ ] `app/core/src/lib.rs` が作成されている
- [ ] すべてのモジュールが適切にエクスポートされている
- [ ] 公開APIが整理されている
- [ ] `cargo build -p suika-game-core` が成功する

**実装ガイド**:
```rust
// app/core/src/lib.rs
pub mod components;
pub mod constants;
pub mod fruit;
pub mod persistence;
pub mod resources;
pub mod states;

// 再エクスポート
pub use components::*;
pub use constants::*;
pub use fruit::*;
pub use persistence::*;
pub use resources::*;
pub use states::*;
```

**関連ドキュメント**:
- [08_crate_architecture.md](../08_crate_architecture.md)

---

### タスク 2.8: メインバイナリへの統合

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-2, integration

**説明**:
app/suika-game/src/main.rsを更新し、Phase 2で実装したリソースと状態を統合する。

**受け入れ基準**:
- [ ] `app/suika-game/Cargo.toml` に suika-game-core 依存関係が追加されている
- [ ] main.rsでリソースが初期化されている
- [ ] AppState が設定されている
- [ ] ハイスコアが起動時にロードされている
- [ ] `cargo run` でビルドが成功する

**実装ガイド**:
```rust
// app/suika-game/src/main.rs
use bevy::prelude::*;
use suika_game_core::*;

fn main() {
    // ハイスコアをロード
    let highscore_data = load_highscore();

    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .insert_resource(GameState {
            score: 0,
            highscore: highscore_data.highscore,
            elapsed_time: 0.0,
        })
        .insert_resource(ComboTimer::default())
        .insert_resource(GameOverTimer::default())
        .insert_resource(NextFruitType::default())
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
- [ ] `cargo test --workspace` が成功する
- [ ] `cargo clippy --workspace` が警告なしで成功する
- [ ] ハイスコアの保存/読み込みが動作する
- [ ] GameStateリソースが正しく初期化されている
- [ ] FruitTypeのすべてのメソッドが動作する

### 検証手順

```bash
# ビルドチェック
cargo build --workspace

# テスト実行
cargo test --workspace

# Clippy チェック
cargo clippy --workspace -- -D warnings

# または Justコマンド使用
just check
just test
```

## 完了条件

- [ ] すべてのタスクが完了している
- [ ] すべての検証項目が合格している
- [ ] ドキュメントが更新されている（必要な場合）
- [ ] コードがフォーマットされている（`just fmt`）

## 次のフェーズ

Phase 2完了 → 次は **Phase 3: 物理環境の構築** に進む

## 備考

- Phase 2ではまだゲームウィンドウは表示されない
- データ構造とリソース管理の基盤を構築するフェーズ
- ユニットテストを充実させることで、後のフェーズでの問題を早期発見できる
- ハイスコアファイルは `save/highscore.json` に保存される（.gitignoreで除外）

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 未着手
