use lazy_static::lazy_static;
use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};

use crate::prelude::*;

lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

pub struct TheCodeHighlighter {
    syntax: &'static SyntaxReference,
    theme: &'static Theme,
}

impl Default for TheCodeHighlighter {
    fn default() -> Self {
        Self {
            syntax: SYNTAX_SET.find_syntax_plain_text(),
            theme: &THEME_SET.themes["Solarized (light)"],
        }
    }
}

pub trait TheCodeHighlighterTrait: Send {
    fn set_syntax_by_name(&mut self, name: &str);
    fn set_theme(&mut self, theme: &str);

    fn background(&self) -> Option<TheColor>;
    fn selection_background(&self) -> Option<TheColor>;

    fn highlight_line(&self, line: &str) -> Vec<(TheColor, usize)>;
}

impl TheCodeHighlighterTrait for TheCodeHighlighter {
    fn set_syntax_by_name(&mut self, name: &str) {
        if let Some(syntax) = SYNTAX_SET.find_syntax_by_name(name) {
            self.syntax = syntax;
        }
    }

    fn set_theme(&mut self, theme: &str) {
        if let Some(theme) = THEME_SET.themes.get(theme) {
            self.theme = theme;
        }
    }

    fn background(&self) -> Option<TheColor> {
        self.theme
            .settings
            .background
            .map(|color| TheColor::from_u8(color.r, color.g, color.b, color.a))
    }

    fn selection_background(&self) -> Option<TheColor> {
        self.theme
            .settings
            .selection
            .map(|color| TheColor::from_u8(color.r, color.g, color.b, color.a))
    }

    fn highlight_line(&self, line: &str) -> Vec<(TheColor, usize)> {
        let mut h = HighlightLines::new(self.syntax, self.theme);
        h.highlight_line(line, &SYNTAX_SET)
            .map(|ranges| {
                ranges
                    .iter()
                    .map(|(style, token)| {
                        (
                            TheColor::from_u8(
                                style.foreground.r,
                                style.foreground.g,
                                style.foreground.b,
                                style.foreground.a,
                            ),
                            token.len(),
                        )
                    })
                    .collect::<Vec<(TheColor, usize)>>()
            })
            .unwrap_or(vec![(TheColor::default(), line.len())])
    }
}
