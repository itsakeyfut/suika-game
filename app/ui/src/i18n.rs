//! Internationalisation: in-code string table.
//!
//! All user-visible strings should pass through [`t`] so that switching
//! [`Language`] at runtime immediately changes the UI text on the next
//! screen rebuild.

use suika_game_core::resources::settings::Language;

/// Returns the localised string for the given key and language.
///
/// Falls back to `key` itself when a translation is missing, so new keys
/// remain visible during development rather than silently showing empty text.
///
/// All call sites must pass a string literal so that the returned reference
/// can be `'static` even for unknown keys.
pub fn t(key: &'static str, lang: Language) -> &'static str {
    match (key, lang) {
        // ── Title screen ──────────────────────────────────────────────────
        ("game_title", Language::Japanese) => "スイカゲーム",
        ("game_title", Language::English) => "Suika Game",
        ("btn_start", Language::Japanese) => "スタート",
        ("btn_start", Language::English) => "Start",
        ("btn_settings", Language::Japanese) => "設定",
        ("btn_settings", Language::English) => "Settings",
        ("btn_how_to_play", Language::Japanese) => "遊び方",
        ("btn_how_to_play", Language::English) => "Guide",
        ("highscore", Language::Japanese) => "ハイスコア",
        ("highscore", Language::English) => "Best Score",

        // ── Settings screen ───────────────────────────────────────────────
        ("settings_title", Language::Japanese) => "設定",
        ("settings_title", Language::English) => "Settings",
        ("label_bgm", Language::Japanese) => "BGM音量",
        ("label_bgm", Language::English) => "BGM Volume",
        ("label_sfx", Language::Japanese) => "SE音量",
        ("label_sfx", Language::English) => "SFX Volume",
        ("label_effects", Language::Japanese) => "エフェクト",
        ("label_effects", Language::English) => "Effects",
        ("label_language", Language::Japanese) => "言語",
        ("label_language", Language::English) => "Language",
        ("value_on", Language::Japanese) => "ON",
        ("value_on", Language::English) => "ON",
        ("value_off", Language::Japanese) => "OFF",
        ("value_off", Language::English) => "OFF",
        ("lang_japanese", Language::Japanese) => "日本語",
        ("lang_japanese", Language::English) => "Japanese",
        ("lang_english", Language::Japanese) => "English",
        ("lang_english", Language::English) => "English",
        ("btn_back", Language::Japanese) => "もどる",
        ("btn_back", Language::English) => "Back",
        ("btn_quit", Language::Japanese) => "終了",
        ("btn_quit", Language::English) => "Quit",

        // ── How to play screen ────────────────────────────────────────────
        ("how_to_play_title", Language::Japanese) => "遊び方",
        ("how_to_play_title", Language::English) => "How to Play",
        ("htp_drop_title", Language::Japanese) => "フルーツを落とす",
        ("htp_drop_title", Language::English) => "Drop Fruits",
        ("htp_drop_body", Language::Japanese) => {
            "左右キー / マウスで移動\nクリック / スペースで落下"
        }
        ("htp_drop_body", Language::English) => "Move: Arrow keys / Mouse\nDrop: Click / Space",
        ("htp_merge_title", Language::Japanese) => "同じフルーツが合体",
        ("htp_merge_title", Language::English) => "Same Fruits Merge",
        ("htp_merge_body", Language::Japanese) => "隣接する同種フルーツが自動合体\nスコア獲得！",
        ("htp_merge_body", Language::English) => "Adjacent same fruits auto-merge\nEarn score!",
        ("htp_evolve_title", Language::Japanese) => "スイカを目指せ",
        ("htp_evolve_title", Language::English) => "Evolve to Watermelon",
        ("htp_evolve_body", Language::Japanese) => {
            "合体するたびに大きなフルーツに進化\nスイカが最大！"
        }
        ("htp_evolve_body", Language::English) => {
            "Fruits evolve on each merge\nWatermelon is the biggest!"
        }
        ("htp_gameover_title", Language::Japanese) => "ゲームオーバー",
        ("htp_gameover_title", Language::English) => "Game Over",
        ("htp_gameover_body", Language::Japanese) => "フルーツが境界ラインを超えたら終了",
        ("htp_gameover_body", Language::English) => "Game ends when fruits exceed the boundary",

        // ── Pause menu ────────────────────────────────────────────────────
        ("pause_title", Language::Japanese) => "ポーズ",
        ("pause_title", Language::English) => "PAUSED",
        ("btn_resume", Language::Japanese) => "再開",
        ("btn_resume", Language::English) => "Resume",
        ("btn_title", Language::Japanese) => "タイトルへ",
        ("btn_title", Language::English) => "To Title",

        // ── Game-over screen ──────────────────────────────────────────────
        ("score", Language::Japanese) => "スコア",
        ("score", Language::English) => "Score",
        ("new_record", Language::Japanese) => "NEW RECORD!",
        ("new_record", Language::English) => "NEW RECORD!",
        ("elapsed_time", Language::Japanese) => "プレイ時間",
        ("elapsed_time", Language::English) => "Play Time",
        ("btn_retry", Language::Japanese) => "もう一度",
        ("btn_retry", Language::English) => "Retry",

        // ── HUD (in-game overlay) ─────────────────────────────────────────
        ("hud_best_score", Language::Japanese) => "ベストスコア",
        ("hud_best_score", Language::English) => "Best Score",
        ("hud_score", Language::Japanese) => "スコア",
        ("hud_score", Language::English) => "Score",
        ("hud_next", Language::Japanese) => "ネクスト",
        ("hud_next", Language::English) => "Next",

        // ── Fallback ──────────────────────────────────────────────────────
        _ => key,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_key_japanese() {
        assert_eq!(t("btn_start", Language::Japanese), "スタート");
    }

    #[test]
    fn test_known_key_english() {
        assert_eq!(t("btn_start", Language::English), "Start");
    }

    #[test]
    fn test_unknown_key_returns_key_itself() {
        // All call sites pass string literals, so the key IS 'static.
        assert_eq!(t("nonexistent_key", Language::Japanese), "nonexistent_key");
        assert_eq!(t("nonexistent_key", Language::English), "nonexistent_key");
    }

    #[test]
    fn test_all_screen_keys_non_empty() {
        let keys = [
            "game_title",
            "btn_start",
            "btn_settings",
            "btn_how_to_play",
            "settings_title",
            "label_bgm",
            "label_sfx",
            "label_effects",
            "label_language",
            "btn_back",
            "how_to_play_title",
            "htp_drop_title",
            "htp_merge_title",
            "htp_evolve_title",
            "htp_gameover_title",
        ];
        for key in &keys {
            assert!(
                !t(key, Language::Japanese).is_empty(),
                "JP key empty: {key}"
            );
            assert!(!t(key, Language::English).is_empty(), "EN key empty: {key}");
        }
    }

    #[test]
    fn test_languages_differ_for_distinguishable_keys() {
        assert_ne!(
            t("btn_start", Language::Japanese),
            t("btn_start", Language::English)
        );
        assert_ne!(
            t("game_title", Language::Japanese),
            t("game_title", Language::English)
        );
    }
}
