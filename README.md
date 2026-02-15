# スイカゲーム (Suika Game Clone)

Bevyゲームエンジンを使用した「スイカゲーム」のクローン実装プロジェクトです。

## 📖 プロジェクト概要

このプロジェクトは、Rustのゲームエンジン**Bevy**と物理エンジン**Rapier2D**を使用して、人気のパズルゲーム「スイカゲーム」を再現する学習プロジェクトです。

### 特徴

- 🎮 **Bevy 0.17.3**: 安定版ECSベースゲームエンジン
- ⚙️ **Rapier2D 0.32.0**: 高性能な2D物理シミュレーション
- 🎨 **2.5D視点**: 斜め俯瞰視点による立体的な表現
- ✨ **リッチなエフェクト**: パーティクル、画面シェイク、アニメーション
- 🎵 **サウンド統合**: BGMと効果音による没入感
- 📊 **スコアシステム**: ハイスコア保存、コンボシステム
- 🔥 **ホットリロード**: 開発中のアセット自動リロード対応
- 🌐 **WASM対応**: ブラウザでも動作可能

## 🎯 ゲームルール

1. **フルーツを落とす**: 画面上部からフルーツを落として箱に積み上げる
2. **合体**: 同じ種類のフルーツ同士が接触すると合体して、次の段階のフルーツに進化
3. **目標**: 最終目標は最大のフルーツ「スイカ」を作ること
4. **ゲームオーバー**: フルーツが境界線を3.0秒超えるとゲームオーバー

### フルーツの進化チェーン（11段階）

サクランボ → イチゴ → ブドウ → デコポン → 柿 → リンゴ → 梨 → 桃 → パイナップル → メロン → **スイカ**

### コンボシステム

- 2.0秒以内の連続合体でコンボ成立
- コンボ数に応じてボーナススコア（+10%〜+50%）

## 🚀 クイックスタート

### 前提条件

- Rust (stable channel)
- Cargo

### インストールと実行

```bash
# リポジトリのクローン
git clone https://github.com/itsakeyfut/suika-game.git
cd suika-game

# 依存関係のビルド（初回は時間がかかります）
cargo build

# ゲームの実行
cargo run -p suika-game

# または justを使用（推奨）
just run
```

### Justコマンドランナー（推奨）

[just](https://github.com/casey/just)をインストールすると、より簡単にコマンドを実行できます：

```bash
# Justのインストール
cargo install just

# 利用可能なコマンドを表示
just --list

# 基本コマンド
just run          # ゲーム実行
just dev          # デバッグモードで実行（詳細ログ付き）
just release      # リリースビルドで実行
just test         # 全テスト実行
just check        # フォーマット + Clippy チェック

# WASM向けビルド
just wasm-build   # WASMビルド
just wasm-serve   # ローカルサーバーで実行
```

## 🎮 操作方法

### ゲームプレイ

**キーボード:**
- **左右矢印キー**: フルーツの落下位置を調整
- **スペースキー**: フルーツを落下
- **ESCキー**: ポーズメニュー

**マウス:**
- **マウス移動**: フルーツの落下位置を調整
- **左クリック**: フルーツを落下

**UI:**
- **マウスクリック**: ボタンの選択

## 📚 ドキュメント

詳細な設計ドキュメントは `docs/` ディレクトリにあります：

1. **[ゲーム仕様書](docs/01_specification.md)** - ゲームルールと要件
2. **[技術アーキテクチャ](docs/02_architecture.md)** - ECS設計とシステム構成
3. **[物理・レンダリング](docs/03_physics_rendering.md)** - 物理エンジンと2.5D表現
4. **[UI/UX設計](docs/04_ui_ux.md)** - 画面レイアウトとインタラクション
5. **[オーディオ設計](docs/05_audio.md)** - BGMと効果音
6. **[実装計画](docs/06_implementation_plan.md)** - フェーズ別実装ガイド（Phase 1-11）
7. **[プロジェクト構造](docs/07_project_structure.md)** - ファイル構成と役割
8. **[クレートアーキテクチャ](docs/08_crate_architecture.md)** - ワークスペース構成と依存関係
9. **[クイックリファレンス](docs/09_quick_reference.md)** - コマンドと設定の早見表
10. **[高度なトピック](docs/10_advanced_topics.md)** - ホットリロード、WASM、アクセシビリティ、テスト

## 🗂️ プロジェクト構造（ワークスペース）

```
suika-game/
├── Cargo.toml          # ワークスペース定義
├── justfile            # Justコマンド定義
├── .claude/            # Claude Code設定
├── .github/            # GitHub Issue テンプレート
├── docs/               # 設計ドキュメント（全11ファイル）
├── assets/             # ゲームアセット
│   ├── sprites/        # スプライト画像
│   ├── sounds/         # BGM・効果音
│   └── fonts/          # フォント
├── app/                # 全クレート（5クレート構成）
│   ├── core/           # suika-game-core: コアゲームロジック
│   ├── ui/             # suika-game-ui: UI実装
│   ├── audio/          # suika-game-audio: オーディオ管理
│   ├── assets/         # suika-game-assets: アセット管理
│   └── suika-game/     # suika-game: メインバイナリ
└── save/               # セーブデータ
```

### クレート依存関係

```
suika-game (main)
  ├─→ suika-game-core
  ├─→ suika-game-ui ────→ suika-game-core
  ├─→ suika-game-audio ─→ suika-game-core
  └─→ suika-game-assets → suika-game-core
```

## 🛠️ 技術スタック

- **言語**: Rust (Edition 2024, Stable Channel)
- **ゲームエンジン**: [Bevy](https://bevyengine.org/) 0.17.3
- **物理エンジン**: [bevy_rapier2d](https://github.com/dimforge/bevy_rapier) 0.32.0
- **オーディオ**: [bevy_kira_audio](https://github.com/NiklasEi/bevy_kira_audio) 0.24.0
- **その他**: rand 0.10, serde 1.0

### バージョン選択の理由

安定性を重視し、実績のある安定版バージョンを採用しています：
- Bevy 0.17.3: 安定版（ホットリロード完全対応）
- Rapier 0.32.0: Bevy 0.17.3との互換性確認済み
- bevy_kira_audio 0.24.0: 最新安定版

## 📋 開発ロードマップ

現在の実装状況：

- [x] **Phase 1**: プロジェクトセットアップ ✅
  - ワークスペース構成
  - ドキュメント作成（全11ファイル）
  - 開発環境整備
- [ ] **Phase 2**: ゲーム状態管理とリソース
- [ ] **Phase 3**: 物理環境の構築
- [ ] **Phase 4**: フルーツシステムの実装
- [ ] **Phase 5**: 衝突検出と合体システム
- [ ] **Phase 6**: ゲームオーバー判定
- [ ] **Phase 7**: 基本UIの実装
- [ ] **Phase 8**: リッチなビジュアルエフェクト
- [ ] **Phase 9**: サウンド統合
- [ ] **Phase 10**: 調整とポリッシュ
- [ ] **Phase 11**: ピクセルアート統合

詳細は [実装計画書](docs/06_implementation_plan.md) を参照してください。

## 🧪 開発・テスト

### コード品質チェック

```bash
# 全テストの実行
cargo test
# または
just test

# コードフォーマット
cargo fmt --all
# または
just fmt

# Lintチェック
cargo clippy --workspace -- -D warnings
# または
just clippy

# フォーマット + Clippy（推奨）
just check
```

### 特定クレートのテスト

```bash
# 単体テスト
just unit-test suika-game-core
just unit-test suika-game-core test_fruit_merge

# 統合テスト
just integration-test suika-game-core
```

## 🔥 ホットリロード（開発機能）

Bevyの標準ホットリロード機能により、ゲーム実行中にアセットを編集すると即座に反映されます。

```bash
# デバッグモードで実行（ホットリロード有効）
cargo run
# または
just dev

# アセットを編集
# → assets/sprites/cherry.png を編集して保存
# → ゲーム内で即座に反映！
```

詳細は [docs/10_advanced_topics.md セクション1](docs/10_advanced_topics.md) を参照。

## 🌐 WASM対応

ブラウザで動作するWebAssemblyビルドに対応しています。

```bash
# WASM向けビルド
just wasm-build

# ローカルサーバーで実行
just wasm-serve
# → http://localhost:8000 でアクセス
```

詳細は [docs/10_advanced_topics.md セクション2](docs/10_advanced_topics.md) を参照。

## 🎨 アセットについて

現在はプレースホルダー（単色の円）を使用しています。将来的に自作のピクセルアートスプライトに置き換える予定です（Phase 11）。

### プレースホルダーアセット

- **フルーツ**: 各種類ごとに異なる色の円形スプライト
- **BGM/効果音**: フリー素材またはシンセサイザーで生成予定
- **フォント**: システムフォントまたはフリーフォント

## 🤝 コントリビューション

このプロジェクトは学習目的のため、現時点ではコントリビューションは受け付けていません。

バグ報告や機能提案は [GitHub Issues](https://github.com/itsakeyfut/suika-game/issues) でお願いします。

## 📝 ライセンス

MIT License

Copyright (c) 2026 itsakeyfut

詳細は [LICENSE](LICENSE) ファイルを参照してください。

## 🔗 参考リンク

### 公式ドキュメント

- [Bevy 公式サイト](https://bevyengine.org/)
- [Bevy Rapier2D](https://github.com/dimforge/bevy_rapier)
- [bevy_kira_audio](https://github.com/NiklasEi/bevy_kira_audio)

### スイカゲーム関連

- [スイカゲーム - Wikipedia](https://ja.wikipedia.org/wiki/%E3%82%B9%E3%82%A4%E3%82%AB%E3%82%B2%E3%83%BC%E3%83%A0)
- [スイカゲーム完全攻略](https://news.denfaminicogamer.jp/kikakuthetower/231031f)

### 開発ツール

- [Just コマンドランナー](https://github.com/casey/just)
- [Rust公式サイト](https://www.rust-lang.org/)
