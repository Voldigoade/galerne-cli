use galerne::storage::schema::SaveState;
use galerne::storage::stats::format_career_stats;
use galerne::ui::i18n::{format_message, has_translation, t, Locale, ALL_KEYS, ALL_LOCALES};

#[test]
fn test_locale_from_code_and_as_code() {
    assert_eq!(Locale::from_code("en"), Some(Locale::En));
    assert_eq!(Locale::from_code("fr"), Some(Locale::Fr));
    assert_eq!(Locale::from_code("es"), Some(Locale::Es));
    assert_eq!(Locale::from_code("zh-CN"), Some(Locale::ZhCn));
    assert_eq!(Locale::from_code("zh"), Some(Locale::ZhCn));
    assert_eq!(Locale::from_code("unknown"), None);

    assert_eq!(Locale::En.as_code(), "en");
    assert_eq!(Locale::Fr.as_code(), "fr");
    assert_eq!(Locale::Es.as_code(), "es");
    assert_eq!(Locale::ZhCn.as_code(), "zh-CN");
}

#[test]
fn test_all_keys_translated_in_all_locales() {
    for &key in ALL_KEYS {
        let en_val = t(key, Locale::En);
        assert!(
            !en_val.is_empty(),
            "English translation for '{}' must not be empty",
            key
        );

        for &locale in ALL_LOCALES {
            assert!(
                has_translation(key, locale),
                "Translation table is missing '{}' in locale {:?}",
                key,
                locale
            );
            let val = t(key, locale);
            assert!(
                !val.is_empty(),
                "Translation for '{}' in locale {:?} must not be empty",
                key,
                locale
            );
            assert_ne!(
                val, key,
                "Translation for '{}' in locale {:?} must not be raw key",
                key, locale
            );
        }
    }
}

#[test]
fn test_career_stats_use_selected_locale() {
    let state = SaveState::default();
    let french = format_career_stats(&state, Locale::Fr);
    let spanish = format_career_stats(&state, Locale::Es);
    let chinese = format_career_stats(&state, Locale::ZhCn);

    assert!(french.contains("Temps passé dans le jeu"));
    assert!(spanish.contains("Tiempo en el juego"));
    assert!(chinese.contains("游戏时间"));
    assert!(!french.contains("Total Game Time"));
    assert!(!spanish.contains("Total Game Time"));
    assert!(!chinese.contains("Total Game Time"));
}

#[test]
fn active_run_summary_reports_traversal_state_without_survivor_score() {
    for &locale in ALL_LOCALES {
        let summary = format_message(
            "active_run_summary",
            locale,
            &[
                ("distance", "120".to_owned()),
                ("speed", "18".to_owned()),
                ("shards", "4".to_owned()),
                ("shields", "1".to_owned()),
            ],
        );

        assert!(!summary.contains("{score}"));
        assert!(!summary.contains("Score"));
        assert!(!summary.contains("Puntos"));
        assert!(!summary.contains("得分"));
        assert!(summary.contains("120"));
        assert!(summary.contains("4"));
    }
}

#[test]
fn test_checkpoint_event_uses_game_locale() {
    for &locale in ALL_LOCALES {
        let mut game = galerne::game::runner::state::RunnerState::new(120, 30, 0x1234_5678);
        game.set_locale(locale);

        for _ in 0..600 {
            game.player.invulnerable_timer = 10.0;
            game.update(0.033, None);
            if game.last_checkpoint_id == 1 {
                break;
            }
        }

        let expected = galerne::ui::i18n::format_message(
            "checkpoint_message",
            locale,
            &[("name", t("cp_shanty_outpost", locale).to_owned())],
        );
        assert!(
            game.floating_indicators
                .iter()
                .any(|indicator| indicator.text == expected),
            "checkpoint message must use locale {:?}",
            locale
        );
    }
}

#[test]
fn test_locale_detection_fallback() {
    // With no env or unknown lang, fallback is English
    std::env::set_var("GALERNE_LANG", "invalid_locale_xyz");
    let detected = Locale::detect();
    assert_eq!(detected, Locale::En);

    std::env::set_var("GALERNE_LANG", "fr_FR.UTF-8");
    let detected_fr = Locale::detect();
    assert_eq!(detected_fr, Locale::Fr);

    std::env::set_var("GALERNE_LANG", "es_ES.UTF-8");
    let detected_es = Locale::detect();
    assert_eq!(detected_es, Locale::Es);

    std::env::set_var("GALERNE_LANG", "zh_CN.UTF-8");
    let detected_zh = Locale::detect();
    assert_eq!(detected_zh, Locale::ZhCn);

    std::env::remove_var("GALERNE_LANG");
}
