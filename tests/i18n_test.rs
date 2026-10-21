use galerne::ui::i18n::{t, Locale, ALL_KEYS, ALL_LOCALES};

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
