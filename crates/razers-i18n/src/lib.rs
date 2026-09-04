// SPDX-License-Identifier: GPL-2.0-or-later

//! Offline localization for application text, never for wire identifiers.
//!
//! 离线中英文翻译。协议标识、USB ID、源代码符号和原始诊断保持原样。
//! English source messages serve as gettext-style keys. Catalogs are embedded,
//! checked for key and placeholder parity, and never fetched at runtime.

use std::{collections::BTreeMap, sync::OnceLock};

/// Supported display language / 支持的显示语言。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Locale {
    #[default]
    En,
    ZhCn,
}

/// Persisted user preference; Auto is resolved against the current system.
/// 保存的用户选择；Auto 每次按当前系统语言解析。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Language {
    #[default]
    Auto,
    English,
    SimplifiedChinese,
}

impl Language {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Self::Auto),
            "en" | "en-US" | "en-GB" => Some(Self::English),
            "zh-CN" | "zh" | "zh-Hans" => Some(Self::SimplifiedChinese),
            _ => None,
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::English => "en",
            Self::SimplifiedChinese => "zh-CN",
        }
    }

    pub fn resolve(self) -> Locale {
        match self {
            Self::Auto => Locale::system(),
            Self::English => Locale::En,
            Self::SimplifiedChinese => Locale::ZhCn,
        }
    }
}

impl Locale {
    /// Normalize OS locale forms such as zh_CN.UTF-8 or zh-Hans-CN.
    ///
    /// 规范化系统区域标识，例如 zh_CN.UTF-8 或 zh-Hans-CN。
    pub fn from_tag(tag: &str) -> Self {
        if tag
            .split(['-', '_', '.', '@'])
            .next()
            .is_some_and(|part| part.eq_ignore_ascii_case("zh"))
        {
            Self::ZhCn
        } else {
            Self::En
        }
    }

    /// RAZERS_LANG > LC_ALL > LC_MESSAGES > LANG > native OS locale > English.
    ///
    /// 优先级依次为 RAZERS_LANG、LC_ALL、LC_MESSAGES、LANG、原生系统语言、英文。
    pub fn system() -> Self {
        for name in ["RAZERS_LANG", "LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(value) = std::env::var(name) {
                if !value.is_empty() && value != "auto" {
                    return Self::from_tag(&value);
                }
            }
        }
        sys_locale::get_locale().map_or(Self::En, |tag| Self::from_tag(&tag))
    }

    /// Translate a complete message, with English fallback for newer keys.
    ///
    /// 翻译完整消息；未知的新键回退为英文。
    pub fn text(self, key: &str) -> &str {
        catalog(self).get(key).map(String::as_str).unwrap_or(key)
    }

    /// Substitute preformatted positional arguments once (no recursive expansion).
    /// 参数只替换一次，不会把设备名称中的花括号解释为模板。
    pub fn format(self, key: &str, arguments: &[String]) -> String {
        let template = self.text(key);
        let mut output = String::new();
        let mut rest = template;
        while let Some(start) = rest.find('{') {
            output.push_str(&rest[..start]);
            let tail = &rest[start + 1..];
            if let Some(end) = tail.find('}') {
                if let Ok(index) = tail[..end].parse::<usize>() {
                    if let Some(argument) = arguments.get(index) {
                        output.push_str(argument);
                        rest = &tail[end + 1..];
                        continue;
                    }
                }
            }
            output.push('{');
            rest = tail;
        }
        output.push_str(rest);
        output
    }
}

/// Extract the shared --lang option without changing process environment.
///
/// 提取共用的 --lang 参数，不修改进程环境。
pub fn language_args(args: Vec<String>) -> Result<(Option<Language>, Vec<String>), String> {
    let mut language = None;
    let mut remaining = Vec::new();
    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        let value = if argument == "--lang" {
            Some(args.next().ok_or("--lang expects auto, en, or zh-CN")?)
        } else {
            argument.strip_prefix("--lang=").map(str::to_owned)
        };
        if let Some(value) = value {
            language = Some(Language::parse(&value).ok_or("--lang expects auto, en, or zh-CN")?);
        } else {
            remaining.push(argument);
        }
    }
    Ok((language, remaining))
}

fn catalog(locale: Locale) -> &'static BTreeMap<String, String> {
    static EN: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    static ZH: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    let (cell, source) = match locale {
        Locale::En => (&EN, include_str!("../locales/en.json")),
        Locale::ZhCn => (&ZH, include_str!("../locales/zh-CN.json")),
    };
    cell.get_or_init(|| {
        serde_json::from_str(source).expect("validated embedded translation catalog")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placeholders(text: &str) -> Vec<usize> {
        let mut result = text
            .split('{')
            .skip(1)
            .filter_map(|part| part.split('}').next()?.parse().ok())
            .collect::<Vec<_>>();
        result.sort_unstable();
        result
    }

    #[test]
    fn catalogs_have_identical_keys_and_placeholders() {
        let en = catalog(Locale::En);
        let zh = catalog(Locale::ZhCn);
        assert!(!en.is_empty());
        assert_eq!(en.keys().collect::<Vec<_>>(), zh.keys().collect::<Vec<_>>());
        for (key, value) in en {
            assert!(!zh[key].trim().is_empty(), "empty translation: {key}");
            assert_eq!(
                placeholders(value),
                placeholders(&zh[key]),
                "placeholder mismatch: {key}"
            );
        }
    }

    #[test]
    fn normalizes_locales_and_falls_back_to_english() {
        for tag in ["zh_CN.UTF-8", "zh-Hans-CN", "ZH-cn", "zh_TW"] {
            assert_eq!(Locale::from_tag(tag), Locale::ZhCn);
        }
        for tag in ["C", "POSIX", "en_US.UTF-8", "fr-FR", ""] {
            assert_eq!(Locale::from_tag(tag), Locale::En);
        }
        assert_eq!(Locale::ZhCn.text("future message"), "future message");
    }

    #[test]
    fn formatting_never_reinterprets_argument_contents() {
        assert_eq!(
            Locale::En.format("{0}: {1}", &["{1}".into(), "设备".into()]),
            "{1}: 设备"
        );
    }

    #[test]
    fn parses_shared_language_options_and_rejects_mistakes() {
        let (language, args) = language_args(vec!["help".into(), "--lang=zh-CN".into()]).unwrap();
        assert_eq!(language, Some(Language::SimplifiedChinese));
        assert_eq!(args, ["help"]);
        assert!(language_args(vec!["--lang".into()]).is_err());
        assert!(language_args(vec!["--lang=fr".into()]).is_err());
    }
}
