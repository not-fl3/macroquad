use crate::types::Color;

#[derive(Debug, Clone)]
pub struct Style {
    pub margin: f32,
    pub title_height: f32,

    pub scroll_width: f32,
    pub scroll_multiplier: f32,

    pub window_border_focused: Color,
    pub window_border_inactive: Color,

    pub window_background_focused: Color,
    pub window_background_inactive: Color,

    pub editbox_background_focused: Color,
    pub editbox_background_inactive: Color,

    pub scrollbar_background_focused_clicked: Color,
    pub scrollbar_background_focused_hovered: Color,
    pub scrollbar_background_focused: Color,
    pub scrollbar_background_focused_inactive: Color,

    pub inactive_title: Color,
    pub focused_title: Color,

    pub focused_text: Color,
    pub inactive_text: Color,

    pub margin_button: f32,
    pub button_background_focused: Color,
    pub button_background_focused_hovered: Color,
    pub button_background_focused_clicked: Color,
    pub button_background_inactive: Color,

    pub group_border_focused_hovered: Color,
    pub group_border_focused: Color,
    pub group_border_focused_highlight: Color,
    pub group_border_inactive_hovered: Color,
    pub group_border_inactive: Color,

    pub editbox_cursor_focused: Color,
    pub editbox_cursor_focused_inactive: Color,
    pub editbox_cursor_inactive: Color,

    pub separator_focused: Color,
    pub separator_inactive: Color,

    pub slider_bar_focused: Color,
    pub slider_bar_hovered: Color,
    pub slider_bar_inactive: Color,

    pub selection_background_focused: Color,
    pub selection_background_inactive: Color,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            margin: 2.0,
            title_height: 14.0,
            scroll_width: 10.0,
            scroll_multiplier: 3.,
            window_border_focused: Color::from_rgb(68, 68, 68),
            window_border_inactive: Color::from_rgba(102, 102, 102, 127),
            window_background_focused: Color::from_rgba(238, 238, 238, 255),
            window_background_inactive: Color::from_rgba(238, 238, 238, 128),
            editbox_background_focused: Color::from_rgba(200, 200, 200, 255),
            editbox_background_inactive: Color::from_rgba(200, 200, 200, 128),
            scrollbar_background_focused_clicked: Color::from_rgba(170, 170, 170, 235),
            scrollbar_background_focused_hovered: Color::from_rgba(180, 180, 180, 235),
            scrollbar_background_focused: Color::from_rgba(204, 204, 204, 235),
            scrollbar_background_focused_inactive: Color::from_rgba(204, 204, 204, 128),
            inactive_title: Color::from_rgba(102, 102, 102, 128),
            focused_title: Color::from_rgba(0, 0, 0, 255),
            focused_text: Color::from_rgba(0, 0, 0, 255),
            inactive_text: Color::from_rgba(102, 102, 102, 127),
            margin_button: 3.,
            button_background_focused: Color::from_rgba(204, 204, 204, 235),
            button_background_focused_hovered: Color::from_rgba(170, 170, 170, 235),
            button_background_focused_clicked: Color::from_rgba(187, 187, 187, 255),
            button_background_inactive: Color::from_rgba(204, 204, 204, 127),
            group_border_focused_hovered: Color::from_rgba(34, 153, 34, 68),
            group_border_focused: Color::from_rgba(34, 34, 34, 68),
            group_border_focused_highlight: Color::from_rgba(34, 34, 255, 255),
            group_border_inactive_hovered: Color::from_rgba(17, 136, 17, 34),
            group_border_inactive: Color::from_rgba(17, 17, 17, 34),

            editbox_cursor_focused: Color::from_rgba(0, 0, 0, 235),
            editbox_cursor_focused_inactive: Color::from_rgba(220, 220, 220, 235),
            editbox_cursor_inactive: Color::from_rgba(50, 50, 50, 34),

            separator_focused: Color::from_rgba(180, 180, 180, 235),
            separator_inactive: Color::from_rgba(180, 180, 180, 134),

            slider_bar_focused: Color::from_rgba(100, 100, 100, 255),
            slider_bar_hovered: Color::from_rgba(220, 220, 220, 255),
            slider_bar_inactive: Color::from_rgba(100, 100, 100, 34),

	    selection_background_focused: Color::from_rgba(100, 100, 100, 100),
	    selection_background_inactive: Color::from_rgba(100, 100, 100, 36),
        }
    }
}

impl Style {
    pub fn window_border(&self, focused: bool) -> Color {
        if focused {
            self.window_border_focused
        } else {
            self.window_border_inactive
        }
    }

    pub fn background(&self, focused: bool) -> Color {
        if focused {
            self.window_background_focused
        } else {
            self.window_background_inactive
        }
    }

    pub fn editbox_background(&self, focused: bool) -> Color {
        if focused {
            self.editbox_background_focused
        } else {
            self.editbox_background_inactive
        }
    }

    pub fn scroll_bar_handle(&self, focused: bool, hovered: bool, clicked: bool) -> Color {
        if focused {
            if clicked {
                self.scrollbar_background_focused_clicked
            } else if hovered {
                self.scrollbar_background_focused_hovered
            } else {
                self.scrollbar_background_focused
            }
        } else {
            self.scrollbar_background_focused_inactive
        }
    }

    pub fn title(&self, focused: bool) -> Color {
        if focused {
            self.focused_title
        } else {
            self.inactive_title
        }
    }

    pub fn text(&self, focused: bool) -> Color {
        if focused {
            self.focused_text
        } else {
            self.inactive_text
        }
    }

    pub fn button_background(&self, focused: bool, hovered: bool, clicked: bool) -> Color {
        if focused {
            if clicked {
                self.button_background_focused_clicked
            } else if hovered {
                self.button_background_focused_hovered
            } else {
                self.button_background_focused
            }
        } else {
            self.button_background_inactive
        }
    }

    pub fn drag_border(&self, focused: bool, hovered: bool, highlight: bool) -> Color {
        if focused {
            if hovered {
                self.group_border_focused_hovered
            } else {
                if highlight {
                    self.group_border_focused_highlight
                } else {
                    self.group_border_focused
                }
            }
        } else {
            if hovered {
                self.group_border_inactive_hovered
            } else {
                self.group_border_inactive
            }
        }
    }

    pub fn tabbar_background(
        &self,
        focused: bool,
        selected: bool,
        hovered: bool,
        clicked: bool,
    ) -> Color {
        if focused {
            if clicked {
                self.button_background_focused_clicked
            } else if hovered {
                self.button_background_focused
            } else {
                if selected {
                    self.button_background_focused_hovered
                } else {
                    self.button_background_inactive
                }
            }
        } else {
            self.button_background_inactive
        }
    }

    pub fn editbox_cursor(&self, focused: bool, input_focus: bool) -> Color {
        if focused && input_focus {
            self.editbox_cursor_focused
        } else if input_focus {
            self.editbox_cursor_focused_inactive
        } else {
            self.editbox_cursor_inactive
        }
    }

    pub fn separator(&self, focused: bool) -> Color {
        if focused {
            self.separator_focused
        } else {
            self.separator_inactive
        }
    }

    pub fn slider_bar(&self, focused: bool, hovered: bool) -> Color {
        if focused && hovered {
            self.slider_bar_hovered
        } else if focused {
            self.slider_bar_focused
        } else {
            self.slider_bar_inactive
        }
    }

    pub fn selection_background(&self, focused: bool) -> Color {
	if focused {
	    self.selection_background_focused
	} else {
	    self.selection_background_inactive
	}
    }
}
