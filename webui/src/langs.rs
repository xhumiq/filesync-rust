use lazy_static::lazy_static;
use leptos_i18n::I18nContext;
use crate::i18n::{use_i18n, I18nKeys, Locale};
use std::collections::HashMap;

lazy_static! {
  pub static ref MONTHS: HashMap<&'static Locale, [&'static str; 12]> = {
    let mut m: HashMap<&'static Locale, [&'static str; 12]> = HashMap::new();
    m.insert(&Locale::en, [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December"
    ]);
    m.insert(&Locale::fr, [
        "janvier", "février", "mars", "avril", "mai", "juin",
        "juillet", "août", "septembre", "octobre", "novembre", "décembre"
    ]);
    m.insert(&Locale::zh, [
        "一月", "二月", "三月", "四月", "五月", "六月",
        "七月", "八月", "九月", "十月", "十一月", "十二月"
    ]);
    m
  };

}

/// Get all 12 month names for a given language code.
/// Falls back to English if language not found.
pub fn month_names(lang: Locale) -> [&'static str; 12] {
    MONTHS.get(&lang).copied().unwrap_or(MONTHS[&Locale::en])
}

pub fn format_date(lang: Locale, date: &chrono::NaiveDate) -> String {
    return match lang {
        Locale::en => date.format("%A, %B %e, %Y").to_string(),
        Locale::fr => date.format("%A %e %B %Y").to_string(),
        Locale::zh => date.format("%Y年%m月%d日 %A").to_string()
            .replace("Monday", "星期一")
            .replace("Tuesday", "星期二")
            .replace("Wednesday", "星期三")
            .replace("Thursday", "星期四")
            .replace("Friday", "星期五")
            .replace("Saturday", "星期六")
            .replace("Sunday", "星期日")
    }
}

/// Get a single month name (1-based index)
pub fn month_name(lang: Locale, month: usize) -> Option<&'static str> {
    if !(1..=12).contains(&month) {
        return None;
    }
    Some(month_names(lang)[month - 1])
}

/// Get the current language code from localStorage or browser language
pub fn get_locale() -> (I18nContext<Locale, I18nKeys>, Locale) {
    let i18n = use_i18n();
    let mut loc = Locale::en;
    if let Some(window) = web_sys::window() {
        // First check localStorage
        let mut locale = if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(locale_str)) = storage.get_item("locale") {
                match locale_str.as_str() {
                    "zh" => Some(Locale::zh),
                    "en" => Some(Locale::en),
                    "fr" => Some(Locale::fr),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        // If not in localStorage, check browser language
        if locale.is_none() {
            match window.navigator().language() {
                Some(lang) => {
                    if lang.starts_with("zh") {
                        loc = Locale::zh;
                    } else if lang.starts_with("fr") {
                        loc = Locale::fr;
                    }
                }
                None => {}
            }
        }else {
            loc = locale.unwrap();
        }

        // Set the locale
        i18n.set_locale(loc);
    }
    (i18n, loc)
}

pub fn toggle_locale(i18n:I18nContext<Locale, I18nKeys>, local_text: &str) ->  Locale {
    let mut loc = Locale::en;
    if let Some(window) = web_sys::window() {
        let mut new_locale = match local_text {
            "zh" => Some(Locale::zh),
            "fr" => Some(Locale::fr),
            "en" => Some(Locale::en),
            _ => None
        };
        if new_locale==None{
            // First check localStorage
            let current_locale = i18n.get_locale();
            new_locale = match current_locale {
                Locale::zh => Some(Locale::en),
                Locale::fr => Some(Locale::zh),
                _ => Some(Locale::fr)
            };
        }
        loc = new_locale.unwrap();
        i18n.set_locale(loc);

        // Save to localStorage
        if let Ok(Some(storage)) = window.local_storage() {
            let locale_str = match loc {
                Locale::en => "en",
                Locale::zh => "zh",
                Locale::fr => "fr",
            };
            let _ = storage.set_item("locale", locale_str);
        }
    }
    loc
}
