use crate::config::Config;

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsAction {
    None,
    FontIncreased,
    FontDecreased,
    Exit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsItem {
    FontSize,
    MarginTop,
    MarginBottom,
    MarginLeft,
    MarginRight,
    Exit,
}

impl SettingsItem {
    pub fn label(&self, config: &Config) -> String {
        match self {
            SettingsItem::FontSize     => format!("Font Size:      {:.0}pt", config.font_size),
            SettingsItem::MarginTop    => format!("Margin Top:     {}px",    config.margin_top),
            SettingsItem::MarginBottom => format!("Margin Bottom:  {}px",    config.margin_bottom),
            SettingsItem::MarginLeft   => format!("Margin Left:    {}px",    config.margin_left),
            SettingsItem::MarginRight  => format!("Margin Right:   {}px",    config.margin_right),
            SettingsItem::Exit         => "Save and Exit".to_string(),
        }
    }

    pub fn all() -> Vec<SettingsItem> {
        vec![
            SettingsItem::FontSize,
            SettingsItem::MarginTop,
            SettingsItem::MarginBottom,
            SettingsItem::MarginLeft,
            SettingsItem::MarginRight,
            SettingsItem::Exit,
        ]
    }
}

pub struct Settings {
    pub items:    Vec<SettingsItem>,
    pub selected: usize,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            items:    SettingsItem::all(),
            selected: 0,
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.items.len() {
            self.selected += 1;
        }
    }

    /// Increase selected setting value
    pub fn increase(&mut self, config: &mut Config) -> SettingsAction {
        match self.items[self.selected] {
            SettingsItem::FontSize => {
                config.increase_font();
                SettingsAction::FontIncreased
            }
            SettingsItem::MarginTop => {
                if config.margin_top < 200 {
                    config.margin_top += 5;
                }
                SettingsAction::None
            }
            SettingsItem::MarginBottom => {
                if config.margin_bottom < 200 {
                    config.margin_bottom += 5;
                }
                SettingsAction::None
            }
            SettingsItem::MarginLeft => {
                if config.margin_left < 200 {
                    config.margin_left += 5;
                }
                SettingsAction::None
            }
            SettingsItem::MarginRight => {
                if config.margin_right < 200 {
                    config.margin_right += 5;
                }
                SettingsAction::None
            }
            SettingsItem::Exit => SettingsAction::Exit,
        }
    }

    /// Decrease selected setting value
    pub fn decrease(&mut self, config: &mut Config) -> SettingsAction {
        match self.items[self.selected] {
            SettingsItem::FontSize => {
                config.decrease_font();
                SettingsAction::FontDecreased
            }
            SettingsItem::MarginTop => {
                if config.margin_top > 10 {
                    config.margin_top -= 5;
                }
                SettingsAction::None
            }
            SettingsItem::MarginBottom => {
                if config.margin_bottom > 10 {
                    config.margin_bottom -= 5;
                }
                SettingsAction::None
            }
            SettingsItem::MarginLeft => {
                if config.margin_left > 10 {
                    config.margin_left -= 5;
                }
                SettingsAction::None
            }
            SettingsItem::MarginRight => {
                if config.margin_right > 10 {
                    config.margin_right -= 5;
                }
                SettingsAction::None
            }
            SettingsItem::Exit => SettingsAction::Exit,
        }
    }

    pub fn select(&mut self, config: &mut Config) -> SettingsAction {
        match self.items[self.selected] {
            SettingsItem::Exit => SettingsAction::Exit,
            _                  => SettingsAction::None,
        }
    }

    /// Get labels for all items for rendering
    pub fn render_items(&self, config: &Config) -> Vec<(String, bool)> {
        self.items
            .iter()
            .enumerate()
            .map(|(i, item)| (item.label(config), i == self.selected))
            .collect()
    }
}