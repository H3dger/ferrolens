use clap::ValueEnum;
use ratatui::style::Color;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum ThemeName {
    #[default]
    Default,
    Catppuccin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThemePalette {
    pub table_header: Color,
    pub header_hues: [Color; 7],
    pub status_bar: Color,
    pub background: Color,
    pub top_bar_bg: Color,
    pub detail_bg: Color,
    pub bottom_bar_bg: Color,
    pub accent: Color,
    pub text: Color,
    pub muted: Color,
    pub border: Color,
    pub selected_row_bg: Color,
    pub selected_row_fg: Color,
    pub focused_header_bg: Color,
    pub focused_header_fg: Color,
    pub focused_column_fg: Color,
    pub current_cell_bg: Color,
    pub current_cell_fg: Color,
    pub filtered_header_fg: Color,
    pub sorted_header_fg: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

impl ThemePalette {
    pub fn from_theme(theme: ThemeName) -> Self {
        match theme {
            ThemeName::Default => Self {
                table_header: Color::Gray,
                header_hues: [
                    Color::Gray,
                    Color::Blue,
                    Color::Cyan,
                    Color::Green,
                    Color::Magenta,
                    Color::Yellow,
                    Color::White,
                ],
                status_bar: Color::DarkGray,
                background: Color::Reset,
                top_bar_bg: Color::Reset,
                detail_bg: Color::Reset,
                bottom_bar_bg: Color::Reset,
                accent: Color::Magenta,
                text: Color::White,
                muted: Color::Gray,
                border: Color::DarkGray,
                selected_row_bg: Color::Rgb(32, 36, 44),
                selected_row_fg: Color::White,
                focused_header_bg: Color::Rgb(44, 52, 64),
                focused_header_fg: Color::White,
                focused_column_fg: Color::Magenta,
                current_cell_bg: Color::Rgb(58, 80, 110),
                current_cell_fg: Color::White,
                filtered_header_fg: Color::Yellow,
                sorted_header_fg: Color::Cyan,
                success: Color::Green,
                warning: Color::Rgb(255, 175, 95),
                error: Color::Red,
            },
            ThemeName::Catppuccin => {
                let palette = catppuccin::PALETTE.mocha.colors;
                Self {
                    table_header: to_ratatui_color(palette.subtext1),
                    header_hues: [
                        to_ratatui_color(palette.rosewater),
                        to_ratatui_color(palette.flamingo),
                        to_ratatui_color(palette.pink),
                        to_ratatui_color(palette.mauve),
                        to_ratatui_color(palette.lavender),
                        to_ratatui_color(palette.sapphire),
                        to_ratatui_color(palette.green),
                    ],
                    status_bar: to_ratatui_color(palette.subtext0),
                    background: to_ratatui_color(palette.base),
                    top_bar_bg: Color::Reset,
                    detail_bg: Color::Reset,
                    bottom_bar_bg: Color::Reset,
                    accent: to_ratatui_color(palette.mauve),
                    text: to_ratatui_color(palette.text),
                    muted: to_ratatui_color(palette.overlay0),
                    border: to_ratatui_color(palette.surface1),
                    selected_row_bg: to_ratatui_color(palette.surface0),
                    selected_row_fg: to_ratatui_color(palette.text),
                    focused_header_bg: to_ratatui_color(palette.surface0),
                    focused_header_fg: to_ratatui_color(palette.lavender),
                    focused_column_fg: to_ratatui_color(palette.lavender),
                    current_cell_bg: to_ratatui_color(palette.surface1),
                    current_cell_fg: to_ratatui_color(palette.lavender),
                    filtered_header_fg: to_ratatui_color(palette.yellow),
                    sorted_header_fg: to_ratatui_color(palette.teal),
                    success: to_ratatui_color(palette.green),
                    warning: to_ratatui_color(palette.peach),
                    error: to_ratatui_color(palette.red),
                }
            }
        }
    }
}

impl ThemeName {
    pub fn as_str(self) -> &'static str {
        match self {
            ThemeName::Default => "default",
            ThemeName::Catppuccin => "catppuccin",
        }
    }
}

fn to_ratatui_color(color: catppuccin::Color) -> Color {
    Color::Rgb(color.rgb.r, color.rgb.g, color.rgb.b)
}
