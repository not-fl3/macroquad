use crate::{
    color::Color,
    math::RectOffset,
    text::{
        atlas::{Atlas, SpriteKey},
        Font,
    },
    texture::Image,
    ui::ElementState,
    Error,
};

use std::sync::{Arc, Mutex};

pub struct StyleBuilder {
    atlas: Arc<Mutex<Atlas>>,
    font: Arc<Mutex<Font>>,
    font_size: u16,
    text_color: Color,
    text_color_hovered: Color,
    text_color_clicked: Color,
    background_margin: Option<RectOffset>,
    margin: Option<RectOffset>,
    background: Option<Image>,
    background_hovered: Option<Image>,
    background_clicked: Option<Image>,
    color: Color,
    color_inactive: Option<Color>,
    color_hovered: Color,
    color_selected: Color,
    color_selected_hovered: Color,
    color_clicked: Color,
    reverse_background_z: bool,
}

impl StyleBuilder {
    pub(crate) fn new(default_font: Arc<Mutex<Font>>, atlas: Arc<Mutex<Atlas>>) -> StyleBuilder {
        StyleBuilder {
            atlas,
            font: default_font,
            font_size: 16,
            text_color: Color::from_rgba(0, 0, 0, 255),
            text_color_hovered: Color::from_rgba(0, 0, 0, 255),
            text_color_clicked: Color::from_rgba(0, 0, 0, 255),
            color: Color::from_rgba(255, 255, 255, 255),
            color_hovered: Color::from_rgba(255, 255, 255, 255),
            color_clicked: Color::from_rgba(255, 255, 255, 255),
            color_selected: Color::from_rgba(255, 255, 255, 255),
            color_selected_hovered: Color::from_rgba(255, 255, 255, 255),
            color_inactive: None,
            background: None,
            background_margin: None,
            margin: None,
            background_hovered: None,
            background_clicked: None,
            reverse_background_z: false,
        }
    }

    pub fn font(self, ttf_bytes: &[u8]) -> Result<StyleBuilder, Error> {
        let font = Font::load_from_bytes(self.atlas.clone(), ttf_bytes)?;

        Ok(StyleBuilder {
            font: Arc::new(Mutex::new(font)),
            ..self
        })
    }

    pub fn background(self, background: Image) -> StyleBuilder {
        StyleBuilder {
            background: Some(background),
            ..self
        }
    }

    pub fn margin(self, margin: RectOffset) -> StyleBuilder {
        StyleBuilder {
            margin: Some(margin),
            ..self
        }
    }

    pub fn background_margin(self, margin: RectOffset) -> StyleBuilder {
        StyleBuilder {
            background_margin: Some(margin),
            ..self
        }
    }

    pub fn background_hovered(self, background_hovered: Image) -> StyleBuilder {
        StyleBuilder {
            background_hovered: Some(background_hovered),
            ..self
        }
    }

    pub fn background_clicked(self, background_clicked: Image) -> StyleBuilder {
        StyleBuilder {
            background_clicked: Some(background_clicked),
            ..self
        }
    }

    pub fn text_color(self, color: Color) -> StyleBuilder {
        StyleBuilder {
            text_color: color,
            ..self
        }
    }

    pub fn text_color_hovered(self, color_hovered: Color) -> StyleBuilder {
        StyleBuilder {
            text_color_hovered: color_hovered,
            ..self
        }
    }

    pub fn text_color_clicked(self, color_clicked: Color) -> StyleBuilder {
        StyleBuilder {
            text_color_clicked: color_clicked,
            ..self
        }
    }

    pub fn font_size(self, font_size: u16) -> StyleBuilder {
        StyleBuilder { font_size, ..self }
    }

    pub fn color(self, color: Color) -> StyleBuilder {
        StyleBuilder { color, ..self }
    }

    pub fn color_hovered(self, color_hovered: Color) -> StyleBuilder {
        StyleBuilder {
            color_hovered,
            ..self
        }
    }

    pub fn color_clicked(self, color_clicked: Color) -> StyleBuilder {
        StyleBuilder {
            color_clicked,
            ..self
        }
    }

    pub fn color_selected(self, color_selected: Color) -> StyleBuilder {
        StyleBuilder {
            color_selected: color_selected,
            ..self
        }
    }

    pub fn color_selected_hovered(self, color_selected_hovered: Color) -> StyleBuilder {
        StyleBuilder {
            color_selected_hovered: color_selected_hovered,
            ..self
        }
    }

    pub fn color_inactive(self, color_inactive: Color) -> StyleBuilder {
        StyleBuilder {
            color_inactive: Some(color_inactive),
            ..self
        }
    }

    pub fn reverse_background_z(self, reverse_background_z: bool) -> StyleBuilder {
        StyleBuilder {
            reverse_background_z,
            ..self
        }
    }

    pub fn build(self) -> Style {
        let mut atlas = self.atlas.lock().unwrap();

        let background = self.background.map(|image| {
            let id = atlas.new_unique_id();
            atlas.cache_sprite(id, image);
            id
        });

        let background_hovered = self.background_hovered.map(|image| {
            let id = atlas.new_unique_id();
            atlas.cache_sprite(id, image);
            id
        });

        let background_clicked = self.background_clicked.map(|image| {
            let id = atlas.new_unique_id();
            atlas.cache_sprite(id, image);
            id
        });

        Style {
            background_margin: self.background_margin,
            margin: self.margin,
            background,
            background_hovered,
            background_clicked,
            color: self.color,
            color_hovered: self.color_hovered,
            color_clicked: self.color_clicked,
            color_inactive: self.color_inactive,
            color_selected: self.color_selected,
            color_selected_hovered: self.color_selected_hovered,
            font: self.font,
            text_color: self.text_color,
            text_color_hovered: self.text_color_hovered,
            text_color_clicked: self.text_color_clicked,
            font_size: self.font_size,
            reverse_background_z: self.reverse_background_z,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Style {
    pub(crate) background: Option<SpriteKey>,
    pub(crate) background_hovered: Option<SpriteKey>,
    pub(crate) background_clicked: Option<SpriteKey>,
    pub(crate) color: Color,
    pub(crate) color_inactive: Option<Color>,
    pub(crate) color_hovered: Color,
    pub(crate) color_clicked: Color,
    pub(crate) color_selected: Color,
    pub(crate) color_selected_hovered: Color,
    /// Margins of background image
    /// Applies to background/background_hovered/background_clicked etc
    /// Part of the texture within the margin would not be scaled, which is useful
    /// for things like element borders
    pub(crate) background_margin: Option<RectOffset>,
    /// Margin that do not affect textures
    /// Useful to leave some empty space between element border and element content
    /// Maybe be negative to compensate background_margin when content should overlap the
    /// borders
    pub(crate) margin: Option<RectOffset>,
    pub(crate) font: Arc<Mutex<Font>>,
    pub(crate) text_color: Color,
    pub(crate) text_color_hovered: Color,
    pub(crate) text_color_clicked: Color,
    pub(crate) font_size: u16,
    pub(crate) reverse_background_z: bool,
}

impl Style {
    fn default(font: Arc<Mutex<Font>>) -> Style {
        Style {
            background: None,
            background_margin: None,
            margin: None,
            background_hovered: None,
            background_clicked: None,
            font,
            text_color: Color::from_rgba(0, 0, 0, 255),
            text_color_hovered: Color::from_rgba(0, 0, 0, 255),
            text_color_clicked: Color::from_rgba(0, 0, 0, 255),
            font_size: 16,
            color: Color::from_rgba(255, 255, 255, 255),
            color_hovered: Color::from_rgba(255, 255, 255, 255),
            color_clicked: Color::from_rgba(255, 255, 255, 255),
            color_selected: Color::from_rgba(255, 255, 255, 255),
            color_selected_hovered: Color::from_rgba(255, 255, 255, 255),
            color_inactive: None,
            reverse_background_z: false,
        }
    }

    pub(crate) fn border_margin(&self) -> RectOffset {
        let background_offset = self.background_margin.unwrap_or_default();
        let background = self.margin.unwrap_or_default();

        RectOffset {
            left: background_offset.left + background.left,
            right: background_offset.right + background.right,
            top: background_offset.top + background.top,
            bottom: background_offset.bottom + background.bottom,
        }
    }

    pub(crate) fn text_color(&self, element_state: ElementState) -> Color {
        let ElementState {
            focused,
            hovered,
            clicked,
            ..
        } = element_state;

        if clicked {
            self.text_color_clicked
        } else if hovered {
            self.text_color_hovered
        } else if focused {
            self.text_color
        } else {
            Color::new(
                self.text_color.r * 0.6,
                self.text_color.g * 0.6,
                self.text_color.b * 0.6,
                self.text_color.a * 0.6,
            )
        }
    }

    pub(crate) fn color(&self, element_state: ElementState) -> Color {
        let ElementState {
            clicked,
            hovered,
            focused,
            selected,
        } = element_state;

        if focused == false {
            return self.color_inactive.unwrap_or(Color::from_rgba(
                (self.color.r as f32 * 255.) as u8,
                (self.color.g as f32 * 255.) as u8,
                (self.color.b as f32 * 255.) as u8,
                (self.color.a as f32 * 255. * 0.8) as u8,
            ));
        }
        if clicked {
            return self.color_clicked;
        }
        if selected && hovered {
            return self.color_selected_hovered;
        }

        if selected {
            return self.color_selected;
        }
        if hovered {
            return self.color_hovered;
        }

        self.color
    }

    pub(crate) fn background_sprite(&self, element_state: ElementState) -> Option<SpriteKey> {
        let ElementState {
            clicked, hovered, ..
        } = element_state;

        if clicked && self.background_clicked.is_some() {
            return self.background_clicked;
        }

        if hovered && self.background_hovered.is_some() {
            return self.background_hovered;
        }

        self.background
    }
}

#[derive(Debug, Clone)]
pub struct Skin {
    pub label_style: Style,
    pub button_style: Style,
    pub tabbar_style: Style,
    pub combobox_style: Style,
    pub window_style: Style,
    pub editbox_style: Style,
    pub window_titlebar_style: Style,
    pub scrollbar_style: Style,
    pub scrollbar_handle_style: Style,
    pub checkbox_style: Style,
    pub group_style: Style,

    pub margin: f32,
    pub title_height: f32,

    pub scroll_width: f32,
    pub scroll_multiplier: f32,
}

impl Skin {
    pub(crate) fn new(atlas: Arc<Mutex<Atlas>>, default_font: Arc<Mutex<Font>>) -> Self {
        Skin {
            label_style: Style {
                margin: Some(RectOffset::new(2., 2., 2., 2.)),
                text_color: Color::from_rgba(0, 0, 0, 255),
                color_inactive: Some(Color::from_rgba(0, 0, 0, 128)),
                ..Style::default(default_font.clone())
            },
            button_style: Style {
                margin: Some(RectOffset::new(2., 2., 2., 2.)),
                color: Color::from_rgba(204, 204, 204, 235),
                color_clicked: Color::from_rgba(187, 187, 187, 255),
                color_hovered: Color::from_rgba(170, 170, 170, 235),
                text_color: Color::from_rgba(0, 0, 0, 255),
                ..Style::default(default_font.clone())
            },
            combobox_style: StyleBuilder::new(default_font.clone(), atlas.clone())
                .background_margin(RectOffset::new(1., 14., 1., 1.))
                .color_inactive(Color::from_rgba(238, 238, 238, 128))
                .text_color(Color::from_rgba(0, 0, 0, 255))
                .color(Color::from_rgba(220, 220, 220, 255))
                .background(Image {
                    width: 16,
                    height: 30,
                    bytes: include_bytes!("combobox.img").to_vec(),
                })
                .build(),
            tabbar_style: Style {
                margin: Some(RectOffset::new(2., 2., 2., 2.)),
                color: Color::from_rgba(220, 220, 220, 235),
                color_clicked: Color::from_rgba(187, 187, 187, 235),
                color_hovered: Color::from_rgba(170, 170, 170, 235),
                color_selected_hovered: Color::from_rgba(180, 180, 180, 235),
                color_selected: Color::from_rgba(204, 204, 204, 235),
                text_color: Color::from_rgba(0, 0, 0, 255),
                ..Style::default(default_font.clone())
            },
            window_style: StyleBuilder::new(default_font.clone(), atlas.clone())
                .background_margin(RectOffset::new(1., 1., 1., 1.))
                .color_inactive(Color::from_rgba(238, 238, 238, 128))
                .text_color(Color::from_rgba(0, 0, 0, 255))
                .background(Image {
                    width: 3,
                    height: 3,
                    bytes: vec![
                        68, 68, 68, 255, 68, 68, 68, 255, 68, 68, 68, 255, 68, 68, 68, 255, 238,
                        238, 238, 255, 68, 68, 68, 255, 68, 68, 68, 255, 68, 68, 68, 255, 68, 68,
                        68, 255,
                    ],
                })
                .build(),
            window_titlebar_style: Style {
                color: Color::from_rgba(68, 68, 68, 255),
                color_inactive: Some(Color::from_rgba(102, 102, 102, 127)),
                text_color: Color::from_rgba(0, 0, 0, 255),
                ..Style::default(default_font.clone())
            },
            scrollbar_style: Style {
                color: Color::from_rgba(68, 68, 68, 255),
                ..Style::default(default_font.clone())
            },
            editbox_style: Style {
                text_color: Color::from_rgba(0, 0, 0, 255),
                color_selected: Color::from_rgba(200, 200, 200, 255),
                ..Style::default(default_font.clone())
            },

            scrollbar_handle_style: Style {
                color: Color::from_rgba(204, 204, 204, 235),
                color_inactive: Some(Color::from_rgba(204, 204, 204, 128)),
                color_hovered: Color::from_rgba(180, 180, 180, 235),
                color_clicked: Color::from_rgba(170, 170, 170, 235),
                ..Style::default(default_font.clone())
            },
            checkbox_style: Style {
                text_color: Color::from_rgba(0, 0, 0, 255),
                font_size: 16,
                color: Color::from_rgba(200, 200, 200, 255),
                color_hovered: Color::from_rgba(210, 210, 210, 255),
                color_clicked: Color::from_rgba(150, 150, 150, 255),
                color_selected: Color::from_rgba(128, 128, 128, 255),
                color_selected_hovered: Color::from_rgba(140, 140, 140, 255),
                ..Style::default(default_font.clone())
            },
            group_style: Style {
                color: Color::from_rgba(34, 34, 34, 68),
                color_hovered: Color::from_rgba(34, 153, 34, 68),
                color_selected: Color::from_rgba(34, 34, 255, 255),
                color_selected_hovered: Color::from_rgba(55, 55, 55, 68),
                ..Style::default(default_font.clone())
            },

            margin: 2.0,
            title_height: 14.0,
            scroll_width: 10.0,
            scroll_multiplier: 3.,
        }
    }
}
