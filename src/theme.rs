use serde::{Deserialize, Serialize};
use std::fmt;

/// Terminal color usable in a picker theme.
///
/// Named colors follow crossterm's naming, where plain names (`yellow`, `cyan`)
/// are the bright ANSI variants and `dark_*` names are the normal ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum ThemeColor {
    /// The terminal's default foreground/background color.
    Reset,
    /// 256-color palette index; 0-15 follow the user's terminal palette.
    Ansi(u8),
    Rgb {
        r: u8,
        g: u8,
        b: u8,
    },
}

/// Color names mapped to their ANSI palette index.
const NAMED_COLORS: &[(&str, u8)] = &[
    ("black", 0),
    ("dark_red", 1),
    ("dark_green", 2),
    ("dark_yellow", 3),
    ("dark_blue", 4),
    ("dark_magenta", 5),
    ("dark_cyan", 6),
    ("grey", 7),
    ("dark_grey", 8),
    ("red", 9),
    ("green", 10),
    ("yellow", 11),
    ("blue", 12),
    ("magenta", 13),
    ("cyan", 14),
    ("white", 15),
];

impl ThemeColor {
    /// Parses a color name, `#rrggbb` hex code, or 0-255 palette index.
    ///
    /// Names are case-insensitive and accept `-`, `_`, or no separator
    /// (`dark_grey`, `dark-grey`, `darkgrey`), plus `gray` spellings.
    pub fn parse(value: &str) -> Result<Self, String> {
        let trimmed = value.trim();
        if let Some(hex) = trimmed.strip_prefix('#') {
            return parse_hex(hex).ok_or_else(|| format!("invalid hex color {value:?}"));
        }
        if let Ok(index) = trimmed.parse::<u8>() {
            return Ok(Self::Ansi(index));
        }

        let normalized = trimmed
            .to_ascii_lowercase()
            .replace('-', "_")
            .replace("gray", "grey");
        if matches!(normalized.as_str(), "reset" | "default" | "none") {
            return Ok(Self::Reset);
        }
        let normalized = match normalized.strip_prefix("dark") {
            Some(rest) if !rest.starts_with('_') => format!("dark_{rest}"),
            _ => normalized,
        };
        NAMED_COLORS
            .iter()
            .find(|(name, _)| *name == normalized)
            .map(|(_, index)| Self::Ansi(*index))
            .ok_or_else(|| format!("unknown color {value:?}"))
    }
}

fn parse_hex(hex: &str) -> Option<ThemeColor> {
    if hex.len() != 6 || !hex.is_ascii() {
        return None;
    }
    let channel = |range| u8::from_str_radix(&hex[range], 16).ok();
    Some(ThemeColor::Rgb {
        r: channel(0..2)?,
        g: channel(2..4)?,
        b: channel(4..6)?,
    })
}

impl TryFrom<String> for ThemeColor {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<ThemeColor> for String {
    fn from(color: ThemeColor) -> Self {
        color.to_string()
    }
}

impl fmt::Display for ThemeColor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reset => formatter.write_str("reset"),
            Self::Ansi(index) => match NAMED_COLORS.iter().find(|(_, i)| i == index) {
                Some((name, _)) => formatter.write_str(name),
                None => write!(formatter, "{index}"),
            },
            Self::Rgb { r, g, b } => write!(formatter, "#{r:02x}{g:02x}{b:02x}"),
        }
    }
}

/// Visual style of one picker text role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextStyle {
    pub fg: ThemeColor,
    pub bg: ThemeColor,
    pub bold: bool,
    pub dim: bool,
}

/// Picker colors for each [`RenderStyle`](crate::model::RenderStyle) role.
///
/// Resolved from the global config before the picker pane launches and carried
/// in the picker snapshot; defaults reproduce the original hardcoded look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PickerTheme {
    /// Pane text that is not part of any match.
    pub unmatched: TextStyle,
    /// Matched text following its hint.
    #[serde(rename = "match")]
    pub matched: TextStyle,
    /// Hint badge characters still to be typed to select a match.
    pub hint: TextStyle,
    /// Hint characters already typed; by default greyed out on the hint background.
    pub hint_typed: TextStyle,
    /// Status line showing the mode, typed keys and how to cancel.
    pub status: TextStyle,
}

impl PickerTheme {
    /// Default `hint_typed` style, derived from the hint so the badge keeps its background.
    pub fn default_hint_typed(hint: &TextStyle) -> TextStyle {
        TextStyle {
            fg: ThemeColor::Ansi(8),
            bg: hint.bg,
            bold: false,
            dim: false,
        }
    }
}

impl Default for PickerTheme {
    fn default() -> Self {
        let hint = TextStyle {
            fg: ThemeColor::Ansi(0),
            bg: ThemeColor::Ansi(14),
            bold: true,
            dim: false,
        };
        Self {
            unmatched: TextStyle {
                fg: ThemeColor::Ansi(8),
                bg: ThemeColor::Reset,
                bold: false,
                dim: true,
            },
            matched: TextStyle {
                fg: ThemeColor::Ansi(11),
                bg: ThemeColor::Reset,
                bold: false,
                dim: false,
            },
            hint_typed: Self::default_hint_typed(&hint),
            hint,
            status: TextStyle {
                fg: ThemeColor::Ansi(0),
                bg: ThemeColor::Ansi(7),
                bold: false,
                dim: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_names_case_and_separator_insensitively() {
        for value in [
            "dark_grey",
            "dark-grey",
            "DarkGrey",
            "darkgray",
            "Dark-Gray",
        ] {
            assert_eq!(ThemeColor::parse(value), Ok(ThemeColor::Ansi(8)), "{value}");
        }
        assert_eq!(ThemeColor::parse("yellow"), Ok(ThemeColor::Ansi(11)));
        assert_eq!(ThemeColor::parse("gray"), Ok(ThemeColor::Ansi(7)));
        assert_eq!(ThemeColor::parse("default"), Ok(ThemeColor::Reset));
    }

    #[test]
    fn parses_hex_and_palette_index() {
        assert_eq!(
            ThemeColor::parse("#FF8800"),
            Ok(ThemeColor::Rgb {
                r: 255,
                g: 136,
                b: 0
            })
        );
        assert_eq!(ThemeColor::parse("208"), Ok(ThemeColor::Ansi(208)));
    }

    #[test]
    fn rejects_invalid_colors() {
        for value in ["#fff", "#gggggg", "256", "purple", "darkpurple", ""] {
            assert!(ThemeColor::parse(value).is_err(), "{value}");
        }
    }

    #[test]
    fn display_round_trips_through_parse() {
        for color in [
            ThemeColor::Reset,
            ThemeColor::Ansi(3),
            ThemeColor::Ansi(200),
            ThemeColor::Rgb { r: 1, g: 2, b: 3 },
        ] {
            assert_eq!(ThemeColor::parse(&color.to_string()), Ok(color));
        }
    }
}
