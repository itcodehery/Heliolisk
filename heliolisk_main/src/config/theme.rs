use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreset {
    TokyoNight,
    CatppuccinMocha,
    GruvboxDark,
    Nord,
    SolarizedDark,
}

impl ThemePreset {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().replace(['-', '_', ' '], "").as_str() {
            "tokyonight" | "tokyo" => Some(ThemePreset::TokyoNight),
            "catppuccin" | "catppuccinmocha" | "mocha" => Some(ThemePreset::CatppuccinMocha),
            "gruvbox" | "gruvboxdark" => Some(ThemePreset::GruvboxDark),
            "nord" => Some(ThemePreset::Nord),
            "solarized" | "solarizeddark" => Some(ThemePreset::SolarizedDark),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub bg: Color,
    pub fg: Color,
    pub border: Color,
    pub border_focused: Color,
    pub status_bg: Color,
    pub status_fg: Color,
    pub error_bg: Color,
    pub error_fg: Color,
    pub nav_mode: Color,
    pub edit_mode: Color,
    pub select_mode: Color,
    pub command_mode: Color,
    pub cursor_line_bg: Color,
    pub line_number: Color,
    pub line_number_active: Color,
    pub explorer_dir: Color,
    pub explorer_file: Color,
    pub explorer_selected_bg: Color,
    pub explorer_selected_fg: Color,
    // Syntax tokens
    pub keyword: Color,
    pub function: Color,
    pub string: Color,
    pub number: Color,
    pub comment: Color,
    pub type_name: Color,
    pub operator: Color,
}

impl Theme {
    pub fn preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::TokyoNight => Self::tokyo_night(),
            ThemePreset::CatppuccinMocha => Self::catppuccin_mocha(),
            ThemePreset::GruvboxDark => Self::gruvbox_dark(),
            ThemePreset::Nord => Self::nord(),
            ThemePreset::SolarizedDark => Self::solarized_dark(),
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night".to_string(),
            bg: Color::Rgb(26, 27, 38),
            fg: Color::Rgb(192, 202, 245),
            border: Color::Rgb(59, 66, 97),
            border_focused: Color::Rgb(122, 162, 247),
            status_bg: Color::Rgb(22, 22, 30),
            status_fg: Color::Rgb(169, 177, 214),
            error_bg: Color::Rgb(247, 118, 142),
            error_fg: Color::Rgb(26, 27, 38),
            nav_mode: Color::Rgb(122, 162, 247),
            edit_mode: Color::Rgb(158, 206, 106),
            select_mode: Color::Rgb(224, 175, 104),
            command_mode: Color::Rgb(247, 118, 142),
            cursor_line_bg: Color::Rgb(41, 46, 66),
            line_number: Color::Rgb(86, 95, 137),
            line_number_active: Color::Rgb(122, 162, 247),
            explorer_dir: Color::Rgb(122, 162, 247),
            explorer_file: Color::Rgb(192, 202, 245),
            explorer_selected_bg: Color::Rgb(41, 46, 66),
            explorer_selected_fg: Color::Rgb(125, 207, 255),
            keyword: Color::Rgb(187, 154, 247),
            function: Color::Rgb(122, 162, 247),
            string: Color::Rgb(158, 206, 106),
            number: Color::Rgb(255, 158, 100),
            comment: Color::Rgb(86, 95, 137),
            type_name: Color::Rgb(42, 195, 222),
            operator: Color::Rgb(137, 221, 255),
        }
    }

    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "Catppuccin Mocha".to_string(),
            bg: Color::Rgb(30, 30, 46),
            fg: Color::Rgb(205, 214, 244),
            border: Color::Rgb(69, 71, 90),
            border_focused: Color::Rgb(203, 166, 247),
            status_bg: Color::Rgb(24, 24, 37),
            status_fg: Color::Rgb(186, 194, 222),
            error_bg: Color::Rgb(243, 139, 168),
            error_fg: Color::Rgb(17, 17, 27),
            nav_mode: Color::Rgb(137, 180, 250),
            edit_mode: Color::Rgb(166, 227, 161),
            select_mode: Color::Rgb(249, 226, 175),
            command_mode: Color::Rgb(243, 139, 168),
            cursor_line_bg: Color::Rgb(49, 50, 68),
            line_number: Color::Rgb(108, 112, 134),
            line_number_active: Color::Rgb(203, 166, 247),
            explorer_dir: Color::Rgb(137, 180, 250),
            explorer_file: Color::Rgb(205, 214, 244),
            explorer_selected_bg: Color::Rgb(49, 50, 68),
            explorer_selected_fg: Color::Rgb(180, 190, 254),
            keyword: Color::Rgb(203, 166, 247),
            function: Color::Rgb(137, 180, 250),
            string: Color::Rgb(166, 227, 161),
            number: Color::Rgb(250, 179, 135),
            comment: Color::Rgb(108, 112, 134),
            type_name: Color::Rgb(249, 226, 175),
            operator: Color::Rgb(148, 226, 213),
        }
    }

    pub fn gruvbox_dark() -> Self {
        Self {
            name: "Gruvbox Dark".to_string(),
            bg: Color::Rgb(40, 40, 40),
            fg: Color::Rgb(235, 219, 178),
            border: Color::Rgb(80, 73, 69),
            border_focused: Color::Rgb(215, 153, 33),
            status_bg: Color::Rgb(29, 32, 33),
            status_fg: Color::Rgb(168, 153, 132),
            error_bg: Color::Rgb(204, 36, 29),
            error_fg: Color::Rgb(235, 219, 178),
            nav_mode: Color::Rgb(131, 165, 152),
            edit_mode: Color::Rgb(184, 187, 38),
            select_mode: Color::Rgb(250, 189, 47),
            command_mode: Color::Rgb(251, 73, 52),
            cursor_line_bg: Color::Rgb(60, 56, 54),
            line_number: Color::Rgb(124, 111, 100),
            line_number_active: Color::Rgb(250, 189, 47),
            explorer_dir: Color::Rgb(131, 165, 152),
            explorer_file: Color::Rgb(235, 219, 178),
            explorer_selected_bg: Color::Rgb(60, 56, 54),
            explorer_selected_fg: Color::Rgb(250, 189, 47),
            keyword: Color::Rgb(251, 73, 52),
            function: Color::Rgb(184, 187, 38),
            string: Color::Rgb(184, 187, 38),
            number: Color::Rgb(211, 134, 155),
            comment: Color::Rgb(146, 131, 116),
            type_name: Color::Rgb(250, 189, 47),
            operator: Color::Rgb(254, 128, 25),
        }
    }

    pub fn nord() -> Self {
        Self {
            name: "Nord".to_string(),
            bg: Color::Rgb(46, 52, 64),
            fg: Color::Rgb(236, 239, 244),
            border: Color::Rgb(76, 86, 106),
            border_focused: Color::Rgb(136, 192, 208),
            status_bg: Color::Rgb(36, 41, 51),
            status_fg: Color::Rgb(216, 222, 233),
            error_bg: Color::Rgb(191, 97, 106),
            error_fg: Color::Rgb(46, 52, 64),
            nav_mode: Color::Rgb(129, 161, 193),
            edit_mode: Color::Rgb(163, 190, 140),
            select_mode: Color::Rgb(235, 203, 139),
            command_mode: Color::Rgb(191, 97, 106),
            cursor_line_bg: Color::Rgb(59, 66, 82),
            line_number: Color::Rgb(76, 86, 106),
            line_number_active: Color::Rgb(136, 192, 208),
            explorer_dir: Color::Rgb(136, 192, 208),
            explorer_file: Color::Rgb(229, 233, 240),
            explorer_selected_bg: Color::Rgb(59, 66, 82),
            explorer_selected_fg: Color::Rgb(143, 188, 187),
            keyword: Color::Rgb(129, 161, 193),
            function: Color::Rgb(136, 192, 208),
            string: Color::Rgb(163, 190, 140),
            number: Color::Rgb(180, 142, 173),
            comment: Color::Rgb(97, 110, 136),
            type_name: Color::Rgb(143, 188, 187),
            operator: Color::Rgb(129, 161, 193),
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark".to_string(),
            bg: Color::Rgb(0, 43, 54),
            fg: Color::Rgb(131, 148, 150),
            border: Color::Rgb(7, 54, 66),
            border_focused: Color::Rgb(38, 139, 210),
            status_bg: Color::Rgb(7, 54, 66),
            status_fg: Color::Rgb(147, 161, 161),
            error_bg: Color::Rgb(220, 50, 47),
            error_fg: Color::Rgb(0, 43, 54),
            nav_mode: Color::Rgb(38, 139, 210),
            edit_mode: Color::Rgb(133, 153, 0),
            select_mode: Color::Rgb(181, 137, 0),
            command_mode: Color::Rgb(220, 50, 47),
            cursor_line_bg: Color::Rgb(7, 54, 66),
            line_number: Color::Rgb(88, 110, 117),
            line_number_active: Color::Rgb(38, 139, 210),
            explorer_dir: Color::Rgb(38, 139, 210),
            explorer_file: Color::Rgb(131, 148, 150),
            explorer_selected_bg: Color::Rgb(7, 54, 66),
            explorer_selected_fg: Color::Rgb(42, 161, 152),
            keyword: Color::Rgb(133, 153, 0),
            function: Color::Rgb(38, 139, 210),
            string: Color::Rgb(42, 161, 152),
            number: Color::Rgb(211, 54, 130),
            comment: Color::Rgb(88, 110, 117),
            type_name: Color::Rgb(181, 137, 0),
            operator: Color::Rgb(108, 113, 196),
        }
    }
}

impl From<ThemePreset> for Theme {
    fn from(preset: ThemePreset) -> Self {
        Self::preset(preset)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::tokyo_night()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_presets_resolution() {
        let presets = [
            ("tokyo-night", "Tokyo Night"),
            ("catppuccin-mocha", "Catppuccin Mocha"),
            ("gruvbox-dark", "Gruvbox Dark"),
            ("nord", "Nord"),
            ("solarized-dark", "Solarized Dark"),
        ];

        for (slug, expected_name) in presets {
            let preset = ThemePreset::from_name(slug).expect("Failed to match preset");
            let theme: Theme = preset.into();
            assert_eq!(theme.name, expected_name);
        }

        assert!(ThemePreset::from_name("non-existent-theme").is_none());
    }
}

