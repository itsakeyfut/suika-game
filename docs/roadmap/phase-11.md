# Phase 11: ピクセルアート統合（後日）

## フェーズ概要

**ステータス**: 🔲 未着手（オプション）
**推定工数**: 8-12時間
**完了日**: -
**依存関係**: Phase 10

### 目的
プレースホルダーグラフィックを自作のピクセルアートスプライトに置き換え、視覚的な完成度を高める。

### スコープ
- フルーツスプライトの作成（11種類、200x200px）
- 背景画像の作成（オプション）
- UIエレメントのピクセルアート化
- スプライトシートの作成と統合
- アニメーションフレームの追加（オプション）
- ホットリロードを活用した効率的な作業フロー

## 前提条件

- Phase 10が完了している
- ゲームが完成状態である
- ピクセルアートツール（Aseprite、Pixelorama等）が準備されている

## タスクリスト

### タスク 11.1: フルーツスプライトの作成

**優先度**: P0
**推定工数**: 6時間
**ラベル**: task, phase-11, art

**説明**:
11種類のフルーツのピクセルアートスプライトを作成する。

**受け入れ基準**:
- [ ] 11種類すべてのフルーツスプライトが作成されている
- [ ] 各スプライトのサイズが200x200pxである
- [ ] 円形のシルエットを維持している
- [ ] 各フルーツが識別しやすいデザインである
- [ ] カラーパレットが統一されている
- [ ] PNG形式で保存されている（透過背景）

**実装ガイド**:

**推奨ツール**:
- [Aseprite](https://www.aseprite.org/) - 有料だがプロ仕様
- [Pixelorama](https://orama-interactive.itch.io/pixelorama) - 無料のオープンソース
- [LibreSprite](https://libresprite.github.io/) - 無料、Asepriteのフォーク

**アートスタイル**:
- ピクセルアート、16x16〜32x32基準で200x200pxにスケールアップ
- アウトラインあり（黒または濃い色）
- 簡素化されたディテール（シンプルで識別しやすい）
- 明るく彩度の高い色（ゲームの楽しい雰囲気）

**各フルーツのデザイン要件**:
| フルーツ | 色 | 特徴 |
|---------|-----|------|
| サクランボ | 赤 | 2つの実と茎 |
| イチゴ | 赤〜ピンク | 三角形、種のドット |
| ブドウ | 紫 | 複数の粒 |
| デコポン | オレンジ | 上部に突起 |
| 柿 | オレンジ | ヘタ付き |
| リンゴ | 赤 | 葉っぱ付き |
| 梨 | 黄緑 | 楕円形 |
| 桃 | ピンク | ハート型 |
| パイナップル | 黄 | 葉っぱと格子模様 |
| メロン | 緑 | 網目模様 |
| スイカ | 緑×赤 | 縞模様 |

**ワークフロー**:
```bash
# 1. Asepriteで32x32の基準サイズで作成
# 2. ディテールを追加
# 3. 200x200pxにスケールアップ（Nearest Neighbor）
# 4. PNG形式でエクスポート（透過背景）
# 5. assets/sprites/ に保存

# ファイル命名規則:
# cherry.png, strawberry.png, grape.png, dekopon.png, persimmon.png
# apple.png, pear.png, peach.png, pineapple.png, melon.png, watermelon.png
```

**関連ドキュメント**:
- [01_specification.md - セクション8.1](../01_specification.md)

---

### タスク 11.2: スプライトシートの作成

**優先度**: P1
**推定工数**: 1時間
**ラベル**: task, phase-11, art

**説明**:
個別のスプライトを1枚のスプライトシートにまとめ、読み込みを効率化する。

**受け入れ基準**:
- [ ] スプライトシートが作成されている（fruits.png）
- [ ] 各スプライトが200x200pxのグリッドに配置されている
- [ ] スプライトシート定義ファイルが作成されている（fruits.ron）
- [ ] Bevyで読み込める形式になっている

**実装ガイド**:
```bash
# TextureAtlas用のスプライトシート作成
# 11フルーツ × 200x200px = 2200x200px（横並び）
# または 600x800px（3列4行、1枠空き）

# Aseprite: File > Export Sprite Sheet
# - Layout: By Rows / By Columns
# - Padding: 0px
# - Output: fruits.png

# または手動で配置（GIMPやPhotoshop）
```

```rust
// assets/sprites/fruits.ron
(
    texture_path: "sprites/fruits.png",
    tile_size: (200.0, 200.0),
    columns: 11,
    rows: 1,
    // フルーツの順番: Cherry, Strawberry, Grape, Dekopon, Persimmon,
    //                Apple, Pear, Peach, Pineapple, Melon, Watermelon
)
```

**関連ドキュメント**:
- [10_advanced_topics.md - セクション3](../10_advanced_topics.md)

---

### タスク 11.3: スプライトローダーの実装

**優先度**: P0
**推定工数**: 1.5時間
**ラベル**: task, phase-11, code

**説明**:
スプライトシートを読み込み、フルーツごとにテクスチャを割り当てるシステムを実装する。

**受け入れ基準**:
- [ ] `app/assets/src/sprites.rs` が作成されている
- [ ] `FruitSprites` リソースが定義されている
- [ ] `load_fruit_sprites` システムが実装されている
- [ ] 各FruitTypeに対応するTextureAtlasIndexが設定されている
- [ ] spawn_fruit関数がスプライトを使用するよう更新されている

**実装ガイド**:
```rust
// app/assets/src/sprites.rs
use bevy::prelude::*;

#[derive(Resource)]
pub struct FruitSprites {
    pub texture_atlas: Handle<TextureAtlasLayout>,
    pub texture: Handle<Image>,
}

pub fn load_fruit_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("sprites/fruits.png");

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(200, 200),
        11,
        1,
        None,
        None,
    );

    let texture_atlas = texture_atlas_layouts.add(layout);

    commands.insert_resource(FruitSprites {
        texture_atlas,
        texture,
    });
}

// FruitTypeにインデックスを返すメソッド追加
impl FruitType {
    pub fn sprite_index(&self) -> usize {
        match self {
            FruitType::Cherry => 0,
            FruitType::Strawberry => 1,
            FruitType::Grape => 2,
            FruitType::Dekopon => 3,
            FruitType::Persimmon => 4,
            FruitType::Apple => 5,
            FruitType::Pear => 6,
            FruitType::Peach => 7,
            FruitType::Pineapple => 8,
            FruitType::Melon => 9,
            FruitType::Watermelon => 10,
        }
    }
}

// spawn_fruit関数を更新
pub fn spawn_fruit(
    commands: &mut Commands,
    fruit_type: FruitType,
    position: Vec2,
    fruit_sprites: &FruitSprites,
) {
    let params = fruit_type.parameters();

    commands.spawn((
        Fruit {
            fruit_type,
            points: params.points,
        },
        SpriteBundle {
            texture: fruit_sprites.texture.clone(),
            transform: Transform::from_xyz(position.x, position.y, 0.0),
            ..default()
        },
        TextureAtlas {
            layout: fruit_sprites.texture_atlas.clone(),
            index: fruit_type.sprite_index(),
        },
        RigidBody::Dynamic,
        Collider::ball(params.radius),
        // ... その他の物理コンポーネント
    ));
}
```

**関連ドキュメント**:
- [10_advanced_topics.md - セクション3](../10_advanced_topics.md)

---

### タスク 11.4: 背景画像の作成と統合（オプション）

**優先度**: P2
**推定工数**: 2時間
**ラベル**: task, phase-11, art

**説明**:
ゲーム背景のピクセルアート画像を作成し、統合する。

**受け入れ基準**:
- [ ] 背景画像が作成されている（1280x720px推奨）
- [ ] 背景がゲームを邪魔しない（コントラスト低め）
- [ ] 背景がゲームの雰囲気に合っている
- [ ] 背景が正しく表示される

**実装ガイド**:
```rust
// 背景のスポーン
pub fn setup_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(SpriteBundle {
        texture: asset_server.load("sprites/background.png"),
        transform: Transform::from_xyz(0.0, 0.0, -100.0),
        ..default()
    });
}
```

**デザイン要件**:
- シンプルなグラデーションまたはパターン
- 彩度を抑えた色（ゲームエリアとのコントラスト）
- タイル可能なパターン（オプション）
- ピクセルアート調

**関連ドキュメント**:
- [03_physics_rendering.md - セクション4](../03_physics_rendering.md)

---

### タスク 11.5: UIエレメントのピクセルアート化（オプション）

**優先度**: P2
**推定工数**: 2時間
**ラベル**: task, phase-11, art

**説明**:
ボタン、枠、アイコンなどのUIエレメントをピクセルアートで作成する。

**受け入れ基準**:
- [ ] ボタン画像が作成されている（Normal, Hover, Pressed）
- [ ] 次のフルーツプレビュー枠が作成されている
- [ ] その他のUIパーツが作成されている
- [ ] ピクセルフォントが用意されている（オプション）

**実装ガイド**:
```rust
// 9-sliceボタンの使用（拡大縮小可能）
pub fn spawn_button_with_sprite(
    parent: &mut ChildBuilder,
    text: &str,
    action: ButtonAction,
    asset_server: &AssetServer,
) {
    parent.spawn((
        Button,
        ImageNode {
            image: asset_server.load("ui/button_normal.png"),
            ..default()
        },
        Node {
            width: Val::Px(240.0),
            height: Val::Px(80.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        MenuButton { action },
    )).with_children(|parent| {
        parent.spawn((
            Text::new(text),
            TextFont {
                font: asset_server.load("fonts/PixelFont.ttf"),
                font_size: 32.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
        ));
    });
}
```

**ピクセルフォント**:
- [Press Start 2P](https://fonts.google.com/specimen/Press+Start+2P) - 無料
- [VT323](https://fonts.google.com/specimen/VT323) - 無料
- [Silkscreen](https://fonts.google.com/specimen/Silkscreen) - 無料

**関連ドキュメント**:
- [04_ui_ux.md](../04_ui_ux.md)

---

### タスク 11.6: アニメーションフレームの追加（オプション）

**優先度**: P2
**推定工数**: 3時間
**ラベル**: task, phase-11, art

**説明**:
フルーツに簡単なアニメーション（アイドル、揺れ等）を追加する。

**受け入れ基準**:
- [ ] 各フルーツに2〜4フレームのアニメーションがある
- [ ] アニメーションがループ再生される
- [ ] アニメーションが自然である
- [ ] パフォーマンスに影響がない

**実装ガイド**:
```rust
#[derive(Component)]
pub struct FruitAnimation {
    pub timer: Timer,
    pub frames: Vec<usize>,
    pub current_frame: usize,
}

pub fn animate_fruits(
    mut query: Query<(&mut TextureAtlas, &mut FruitAnimation)>,
    time: Res<Time>,
) {
    for (mut atlas, mut anim) in query.iter_mut() {
        anim.timer.tick(time.delta());

        if anim.timer.just_finished() {
            anim.current_frame = (anim.current_frame + 1) % anim.frames.len();
            atlas.index = anim.frames[anim.current_frame];
        }
    }
}
```

**アニメーション例**:
- アイドルアニメーション（2フレーム、軽く揺れる）
- 合体時のスパークエフェクト（4フレーム）
- 表情の変化（オプション、2フレーム）

**関連ドキュメント**:
- [03_physics_rendering.md - セクション5](../03_physics_rendering.md)

---

### タスク 11.7: ホットリロードを活用したイテレーション

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-11, workflow

**説明**:
Bevyのホットリロード機能を活用して、アートアセットを効率的に調整する。

**受け入れ基準**:
- [ ] アセットファイルを変更すると自動的にゲームに反映される
- [ ] ゲーム再起動なしでスプライトを確認できる
- [ ] イテレーションが高速化されている

**実装ガイド**:
```rust
// Bevyのホットリロードはデフォルトで有効
// AssetServerが自動的にファイル変更を検知

// 作業フロー:
// 1. cargo run でゲームを起動
// 2. Asepriteでスプライトを編集
// 3. Export (Ctrl+E) で上書き保存
// 4. ゲーム画面で自動的に更新される
// 5. 問題があれば2に戻る

// tips: デュアルモニターがあると効率的
// - モニター1: Aseprite
// - モニター2: ゲーム画面
```

**ワークフローのヒント**:
- スプライトを編集 → エクスポート → ゲームで確認
- カラーパレットを最初に確定させる
- 1つのフルーツを完成させてから次に進む
- 定期的にゲーム内で全体のバランスを確認

**関連ドキュメント**:
- [10_advanced_topics.md - セクション4](../10_advanced_topics.md)

---

### タスク 11.8: 最終調整とポリッシュ

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-11, polish

**説明**:
すべてのアートアセットを統合し、最終調整を行う。

**受け入れ基準**:
- [ ] すべてのプレースホルダーがピクセルアートに置き換わっている
- [ ] スプライトのサイズが適切である
- [ ] カラーバランスが統一されている
- [ ] アートスタイルが一貫している
- [ ] ゲーム全体の視覚的完成度が高い

**チェックリスト**:
- [ ] 11種類すべてのフルーツがピクセルアート
- [ ] UIボタンがピクセルアート（オプション）
- [ ] 背景がピクセルアート（オプション）
- [ ] フォントがピクセルフォント（オプション）
- [ ] 境界線のデザインが改善されている（オプション）
- [ ] コンテナ（箱）のデザインが改善されている（オプション）

**最終調整項目**:
- スプライトのサイズ微調整（物理コライダーとの整合性）
- 色の微調整（ゲーム画面全体のバランス）
- アニメーション速度の調整（オプション）
- エフェクトとの調和確認

**関連ドキュメント**:
- [01_specification.md - セクション8](../01_specification.md)

---

## フェーズ検証

### 検証項目

- [ ] すべてのタスクが完了している
- [ ] `cargo build --workspace` が成功する
- [ ] `cargo run` でゲームが起動する
- [ ] すべてのフルーツがピクセルアートで表示される
- [ ] スプライトが正しく読み込まれている
- [ ] フルーツが識別しやすい
- [ ] アートスタイルが統一されている
- [ ] 背景やUIがゲームを邪魔しない
- [ ] パフォーマンスに問題がない
- [ ] ホットリロードが機能している

### 検証手順

```bash
# ゲーム実行
cargo run

# 確認項目:
# 1. すべてのフルーツがピクセルアートで表示される
# 2. 各フルーツが識別しやすい
# 3. カラーバランスが良い
# 4. スプライトサイズが適切
# 5. アニメーションがスムーズ（実装している場合）
# 6. 背景がゲームを邪魔しない
# 7. UIが統一されたデザイン
# 8. 全体的な視覚的完成度が高い

# ホットリロードテスト:
# 1. ゲームを起動したまま
# 2. スプライトファイルを編集
# 3. 保存
# 4. ゲーム内で自動的に更新されることを確認
```

## 完了条件

- [ ] すべてのタスクが完了している
- [ ] すべての検証項目が合格している
- [ ] プレースホルダーグラフィックがすべて置き換わっている
- [ ] アートスタイルが一貫している
- [ ] ゲームが視覚的に魅力的である
- [ ] パフォーマンスに問題がない

## 次のフェーズ

Phase 11完了 → **プロジェクト完成！**

さらなる拡張（タイムアタックモード、オンラインランキング等）は別プロジェクトとして検討。

## 備考

- Phase 11はオプションフェーズであり、必須ではない
- プレースホルダーでも十分にゲームは楽しめる
- ピクセルアート作成は時間がかかるため、急がず丁寧に
- アートスキルがない場合、フリー素材の利用も検討（ライセンス確認必須）
- Asepriteは有料だが、長期的には投資価値がある
- 無料の代替ツール（Pixelorama、LibreSprite）も十分に使える
- ホットリロードを活用して効率的に作業する
- 最初から完璧を目指さず、反復的に改善する

**アート作成のヒント**:
- 参考画像を集める（実物の写真、他のゲームのスプライト）
- シンプルなシルエットから始める
- カラーパレットを制限する（4〜6色程度）
- アウトラインを使って識別しやすくする
- 小さいサイズ（16x16, 32x32）で作成してスケールアップ
- 定期的にゲーム内で確認する
- フィードバックを得る（友人、コミュニティ）

**リソース**:
- [Lospec](https://lospec.com/) - ピクセルアートチュートリアルとパレット
- [itch.io](https://itch.io/game-assets/free/tag-pixel-art) - 無料ピクセルアート素材
- [OpenGameArt](https://opengameart.org/) - 無料ゲームアセット
- [Pixel Art Tutorial](https://blog.studiominiboss.com/pixelart) - 学習リソース

---

**バージョン**: 1.0
**最終更新**: 2026-02-15
**ステータス**: 未着手（オプション）
